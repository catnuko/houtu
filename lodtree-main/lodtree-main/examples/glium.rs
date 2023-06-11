use glium::index::PrimitiveType;
use glium::{glutin, implement_vertex, program, uniform, Surface};

use lodtree::coords::QuadVec;
use lodtree::*;

// the chunk struct for the tree
struct Chunk {
    visible: bool,
    cache_state: i32, // 0 is new, 1 is merged, 2 is cached, 3 is both
    selected: bool,
    in_bounds: bool,
}

fn main() {
    // start the glium event loop
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new().with_title("Quadtree demo");
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // make a vertex buffer
    // we'll reuse it as we only need to draw one quad multiple times anyway
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            // only need a 2d position
            position: [f32; 2],
        }

        implement_vertex!(Vertex, position);

        glium::VertexBuffer::new(
            &display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                },
            ],
        )
        .unwrap()
    };

    // and the index buffer to form the triangle
    let index_buffer = glium::IndexBuffer::new(
        &display,
        PrimitiveType::TrianglesList,
        &[0 as u16, 1, 2, 1, 2, 3],
    )
    .unwrap();

    // and get the shaders
    let program = program!(&display,
    140 => {
        vertex: "
			#version 140

			uniform vec2 offset;
			uniform float scale;

			in vec2 position;

			void main() {

				vec2 local_position = position * scale + 0.005;
				local_position.x = min(local_position.x, scale) - 0.0025;
				local_position.y = min(local_position.y, scale) - 0.0025;

				gl_Position = vec4(local_position + (offset + scale * 0.5) * 2.0 - 1.0, 0.0, 1.0);
			}
		",

        fragment: "
			#version 140

			uniform int state;
			uniform int selected;
			uniform int in_bounds;

			void main() {

				if (state == 0) gl_FragColor = vec4(0.2, 0.2, 0.2, 1.0); // new, white
				if (state == 1) gl_FragColor = vec4(0.0, 0.2, 0.0, 1.0); // merged, green
				if (state == 2) gl_FragColor = vec4(0.2, 0.0, 0.0, 1.0); // from cache, red
				if (state == 3) gl_FragColor = vec4(0.4, 0.2, 0.0, 1.0); // both, yellow

				if (selected != 0) gl_FragColor = vec4(0.0, 0.2, 0.2, 1.0); // selected, so blue
				if (in_bounds != 0) gl_FragColor = vec4(0.0, 0.2, 0.2, 1.0); // in bounds, so purple

			}
		"
    }
    )
    .unwrap();

    let draw = move |mouse_pos: (f64, f64),
                     tree: &mut Tree<Chunk, QuadVec>,
                     display: &glium::Display| {
        // update the tree
        // adding chunks to their respective position, and also set them visible when adding
        if tree.prepare_update(
            &[QuadVec::from_float_coords(
                mouse_pos.0,
                1.0 - mouse_pos.1,
                6,
            )],
            2,
            |_position| Chunk {
                visible: true,
                cache_state: 0,
                selected: false,
                in_bounds: false,
            },
        ) {
            // position should already have been set, so we can just change the visibility
            for chunk in tree.iter_chunks_to_activate_mut() {
                chunk.visible = true;
                chunk.cache_state |= 1;
            }

            for chunk in tree.iter_chunks_to_deactivate_mut() {
                chunk.visible = false;
            }

            // and make chunks that are cached visible
            for chunk in tree.iter_chunks_to_remove_mut() {
                chunk.cache_state = 2;
            }

            // do the update
            tree.do_update();

            // and clean
            tree.complete_update();
        }

        // go over all chunks in the tree and set them to not be selected
        for chunk in tree.iter_chunks_mut() {
            chunk.selected = false;
        }

        // and select the chunk at the mouse position
        if let Some(chunk) = tree.get_chunk_from_position_mut(QuadVec::from_float_coords(
            mouse_pos.0,
            1.0 - mouse_pos.1,
            6,
        )) {
            chunk.selected = true;
        }

        // and select a number of chunks in a region when the mouse buttons are selected

        // and, Redraw!
        let mut target = display.draw();
        target.clear_color(0.6, 0.6, 0.6, 1.0);

        // go over all chunks, iterator version
        for (chunk, position) in tree.iter_chunks_and_positions() {
            if chunk.visible {
                // draw it if it's visible
                // here we get the chunk position and size
                let uniforms = uniform! {
                    offset: [position.get_float_coords().0 as f32, position.get_float_coords().1 as f32],
                    scale: position.get_size() as f32,
                    state: chunk.cache_state,
                    selected: chunk.selected as i32,
                };

                // draw it with glium
                target
                    .draw(
                        &vertex_buffer,
                        &index_buffer,
                        &program,
                        &uniforms,
                        &Default::default(),
                    )
                    .unwrap();
            }
        }

        target.finish().unwrap();
    };

    // set up the tree
    let mut tree = Tree::<Chunk, QuadVec>::new(64);

    draw((0.5, 0.5), &mut tree, &display);

    // the mouse cursor position
    let mut mouse_pos = (0.5, 0.5);

    // last time redraw was done
    let mut last_redraw = std::time::Instant::now();

    // run the main loop
    event_loop.run(move |event, _, control_flow| {
        *control_flow = match event {
            glutin::event::Event::RedrawRequested(_) => {
                // and draw, if enough time elapses
                if last_redraw.elapsed().as_millis() > 16 {
                    draw(mouse_pos, &mut tree, &display);
                    last_redraw = std::time::Instant::now();
                }

                glutin::event_loop::ControlFlow::Wait
            }
            glutin::event::Event::WindowEvent { event, .. } => match event {
                // stop if the window is closed
                glutin::event::WindowEvent::CloseRequested => glutin::event_loop::ControlFlow::Exit,
                glutin::event::WindowEvent::CursorMoved { position, .. } => {
                    // get the mouse position
                    mouse_pos = (
                        position.x / display.get_framebuffer_dimensions().0 as f64,
                        position.y / display.get_framebuffer_dimensions().1 as f64,
                    );

                    // request a redraw
                    display.gl_window().window().request_redraw();

                    glutin::event_loop::ControlFlow::Wait
                }
                _ => glutin::event_loop::ControlFlow::Wait,
            },
            _ => glutin::event_loop::ControlFlow::Wait,
        }
    });
}
