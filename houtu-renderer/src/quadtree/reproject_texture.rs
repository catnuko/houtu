use std::{borrow::Cow, num::NonZeroU32};

use async_channel::{Receiver, Sender};
use bevy::{
    core::cast_slice,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        Extract, RenderApp,
    },
    utils::HashMap,
};
use pollster::FutureExt;
use std::mem;

use super::{imagery_layer::ImageryLayerId, imagery_storage::ImageryKey, tile_key::TileKey};

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let queue = ReprojectTextureTaskQueue::default();
        app.insert_resource(queue);
        app.add_system(receive_images);
        app.add_event::<FinishReprojectTexture>();

        let render_queue = ReprojectTextureTaskQueue {
            map: HashMap::new(),
        };
        let render_app = app.sub_app_mut(RenderApp);
        render_app.add_system(extract_reproject_texture_task_queue.in_schedule(ExtractSchedule));
        render_app.init_resource::<ReprojectTexturePipeline>();
        let mut render_graph = render_app.world.resource_mut::<RenderGraph>();
        render_graph.add_node("reproject_texture", ReprojectTextureNode::default());
        render_graph.add_node_edge(
            "reproject_texture",
            bevy::render::main_graph::node::CAMERA_DRIVER,
        );
        render_app.insert_resource(render_queue);
    }
}
pub struct FinishReprojectTexture {
    pub imagery_key: ImageryKey,
}
pub fn receive_images(
    mut render_world_queue: ResMut<ReprojectTextureTaskQueue>,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
    mut evt_writer: EventWriter<FinishReprojectTexture>,
) {
    for task in render_world_queue.map.iter() {
        async {
            let buffer_slice = task.1.output_buffer.slice(..);

            // NOTE: We have to create the mapping THEN device.poll() before await
            // the future. Otherwise the application will freeze.
            let (tx, rx) = futures_intrusive::channel::shared::oneshot_channel();
            // let (tx,rx) = async_channel::bounded(1);
            buffer_slice.map_async(MapMode::Read, move |result| {
                tx.send(result).unwrap();
            });
            render_device.poll(wgpu::Maintain::Wait);
            rx.receive().await.unwrap().unwrap();
            if let Some(mut image) = images.get_mut(&task.1.image) {
                image.data = buffer_slice.get_mapped_range().to_vec();
            }

            task.1.output_buffer.unmap();

            evt_writer.send(FinishReprojectTexture {
                imagery_key: task.1.imagery_key,
            });

            // bevy::log::info!("task finish {:?}",task.1.imagery_key);
        }
        .block_on();
    }
}

#[derive(Resource)]
pub struct ReprojectTextureTaskQueue {
    map: HashMap<ImageryKey, ReprojectTextureTask>,
}
impl Default for ReprojectTextureTaskQueue {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}
impl ReprojectTextureTaskQueue {
    pub fn count(&self) -> usize {
        self.map.len()
    }
    pub fn get(&self, key: &ImageryKey) -> Option<&ReprojectTextureTask> {
        self.map.get(key)
    }
    pub fn remove(&mut self, key: &ImageryKey) -> Option<ReprojectTextureTask> {
        self.map.remove(key)
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, v: ReprojectTextureTask) {
        self.map.insert(v.imagery_key, v);
    }
    pub fn clear(&mut self) {
        self.map.clear()
    }
}
fn extract_reproject_texture_task_queue(
    main_world_queue: Extract<Res<ReprojectTextureTaskQueue>>,
    mut render_world_queue: ResMut<ReprojectTextureTaskQueue>,
) {
    render_world_queue.clear();
    // bevy::log::info!("task queue length is {}", main_world_queue.count());
    main_world_queue
        .map
        .iter()
        .for_each(|x| render_world_queue.push(x.1.clone()))
}
#[derive(Clone, PartialEq, Eq)]
pub enum ReprojectTextureTaskState {
    Start = 0,
    Drawd = 1,
    Copied = 2,
}
pub struct ReprojectTextureTask {
    // pub key: TileKey,
    pub image: Handle<Image>,
    pub output_texture: Handle<Image>,
    pub output_buffer: Buffer,
    pub webmercartor_buffer: Buffer,
    pub index_buffer: Buffer,
    pub uniform_buffer: Buffer,
    pub imagery_key: ImageryKey,
    pub state: ReprojectTextureTaskState,
}
impl Clone for ReprojectTextureTask {
    fn clone(&self) -> Self {
        Self {
            image: self.image.clone(),
            output_texture: self.output_texture.clone(),
            webmercartor_buffer: self.webmercartor_buffer.clone(),
            index_buffer: self.index_buffer.clone(),
            output_buffer: self.output_buffer.clone(),
            uniform_buffer: self.uniform_buffer.clone(),
            imagery_key: self.imagery_key.clone(),
            state: self.state.clone(),
        }
    }
}
#[derive(Copy, Clone, Debug, Default, ShaderType)]
pub struct ParamsUniforms {
    pub viewport_orthographic: Mat4,
    pub texture_dimensions: UVec2,
}
pub(crate) const UNIFORM_BUFFER_SIZE: BufferAddress =
    mem::size_of::<ParamsUniforms>() as BufferAddress;
#[derive(Resource)]
pub struct ReprojectTexturePipeline {
    texture_bind_group_layout: BindGroupLayout,
    pipeline: CachedRenderPipelineId,
    vertex_buffer: Buffer,
}

impl FromWorld for ReprojectTexturePipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                sample_type: TextureSampleType::Float { filterable: true },
                                view_dimension: TextureViewDimension::D2,
                                multisampled: false,
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::VERTEX_FRAGMENT,
                            ty: BindingType::Buffer {
                                ty: BufferBindingType::Uniform,
                                has_dynamic_offset: false,
                                min_binding_size: BufferSize::new(UNIFORM_BUFFER_SIZE),
                            },
                            count: None,
                        },
                    ],
                });
        let shader = world
            .resource::<AssetServer>()
            .load("reproject_webmecator.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: None,
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            vertex: VertexState {
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("vertex_main"),
                buffers: vec![
                    VertexBufferLayout {
                        array_stride: 0,
                        step_mode: VertexStepMode::Vertex,
                        attributes: vec![VertexAttribute {
                            shader_location: 0,
                            format: VertexFormat::Float32x4,
                            offset: 0,
                        }],
                    },
                    VertexBufferLayout {
                        array_stride: 0,
                        step_mode: VertexStepMode::Vertex,
                        attributes: vec![VertexAttribute {
                            shader_location: 1,
                            format: VertexFormat::Float32x2,
                            offset: 0,
                        }],
                    },
                ],
            },
            fragment: Some(FragmentState {
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("fragment_main"),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::Rgba8UnormSrgb,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
            },
            multisample: MultisampleState::default(),
            depth_stencil: None,
        });

        let render_device = world.resource::<RenderDevice>();
        // let render_queue = world.resource::<RenderQueue>();
        let mut positions = vec![0.0; 2 * 64 * 2];
        let mut index = 0;
        for j in 0..64 {
            let y = j as f32 / 63.0;
            positions[index] = 0.0;
            index += 1;
            positions[index] = y;
            index += 1;
            positions[index] = 1.0;
            index += 1;
            positions[index] = y;
            index += 1;
        }
        let vertex_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: cast_slice(&positions),
            usage: BufferUsages::COPY_DST | BufferUsages::VERTEX,
        });

        ReprojectTexturePipeline {
            texture_bind_group_layout,
            pipeline,
            vertex_buffer,
        }
    }
}
enum State {
    Loading,
    Update,
}
struct ReprojectTextureNode {
    state: State,
}
impl Default for ReprojectTextureNode {
    fn default() -> Self {
        Self {
            state: State::Loading,
        }
    }
}
impl render_graph::Node for ReprojectTextureNode {
    fn update(&mut self, world: &mut World) {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ReprojectTexturePipeline>();
        match self.state {
            State::Loading => {
                if let CachedPipelineState::Ok(_) =
                    pipeline_cache.get_render_pipeline_state(pipeline.pipeline)
                {
                    self.state = State::Update;
                }
            }
            State::Update => {}
        }
    }
    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        if matches!(self.state, State::Loading) {
            return Ok(());
        }
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ReprojectTexturePipeline>();
        let task_queue = world.resource::<ReprojectTextureTaskQueue>();
        let gpu_images = world.resource::<RenderAssets<Image>>();
        let render_pipeline = pipeline_cache
            .get_render_pipeline(pipeline.pipeline)
            .expect("need a reproject texture pipeline");

        // bevy::log::info!("task queue length is {}", task_queue.count());
        for task in task_queue.map.iter() {
            let (input_image, output_texture) = if let (Some(v), Some(v2)) = (
                gpu_images.get(&task.1.image),
                gpu_images.get(&task.1.output_texture),
            ) {
                (v, v2)
            } else {
                continue;
            };

            // bevy::log::info!("reprojecting texture for tile {:?}", task.1.key);
            let bind_group =
                render_context
                    .render_device()
                    .create_bind_group(&BindGroupDescriptor {
                        label: None,

                        layout: &pipeline.texture_bind_group_layout,
                        entries: &[
                            BindGroupEntry {
                                binding: 0,
                                resource: BindingResource::TextureView(&input_image.texture_view),
                            },
                            BindGroupEntry {
                                binding: 1,
                                resource: BindingResource::Sampler(&input_image.sampler),
                            },
                            BindGroupEntry {
                                binding: 2,
                                resource: BindingResource::Buffer(BufferBinding {
                                    buffer: &task.1.uniform_buffer,
                                    offset: 0,
                                    size: BufferSize::new(UNIFORM_BUFFER_SIZE),
                                }),
                            },
                        ],
                    });

            {
                let mut render_pass =
                    render_context
                        .command_encoder()
                        .begin_render_pass(&RenderPassDescriptor {
                            label: "reproject_texture_pass".into(),
                            color_attachments: &[Some(RenderPassColorAttachment {
                                view: &output_texture.texture_view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: wgpu::LoadOp::Clear(wgpu::Color {
                                        r: 0.1,
                                        g: 0.2,
                                        
                                        b: 0.3,
                                        a: 1.0,
                                    }),
                                    store: true,
                                },
                            })],
                            depth_stencil_attachment: None,
                        });
                render_pass.set_pipeline(render_pipeline);
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.set_vertex_buffer(0, *pipeline.vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, *task.1.webmercartor_buffer.slice(..));
                render_pass.set_index_buffer(*task.1.index_buffer.slice(..), IndexFormat::Uint32);
                render_pass.draw(0..3, 0..1);
            }
            let output_texture_size = output_texture.texture.size();
            let u32_size = std::mem::size_of::<u32>() as u32;
            render_context.command_encoder().copy_texture_to_buffer(
                ImageCopyTexture {
                    aspect: TextureAspect::All,
                    texture: &output_texture.texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                },
                ImageCopyBuffer {
                    buffer: &task.1.output_buffer,
                    layout: ImageDataLayout {
                        offset: 0,
                        bytes_per_row: NonZeroU32::new(u32_size * output_texture_size.width),
                        rows_per_image: NonZeroU32::new(output_texture_size.height),
                    },
                },
                output_texture_size,
            );
        }
        Ok(())
    }
}
