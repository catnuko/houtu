use std::borrow::Cow;

use async_channel::{Receiver, Sender};
use bevy::{
    core::cast_slice,
    prelude::*,
    render::{
        extract_resource::{ExtractResource, ExtractResourcePlugin},
        render_asset::RenderAssets,
        render_graph::{self, RenderGraph},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        texture::BevyDefault,
        Extract, RenderApp, RenderSet,
    },
    utils::HashMap,
};
use houtu_scene::Rectangle;

use super::{tile_quad_tree::IndicesAndEdgesCacheArc, TileKey};
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let queue = ReprojectTextureTaskQueue::default();
        let render_queue = ReprojectTextureTaskQueue {
            map: HashMap::new(),
            status_channel: queue.clone_status_channel(),
        };
        app.insert_resource(queue);
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

#[derive(Resource)]
pub struct ReprojectTextureTaskQueue {
    map: HashMap<TileKey, ReprojectTextureTask>,
    pub status_channel: (Sender<TileKey>, Receiver<TileKey>),
}
impl Default for ReprojectTextureTaskQueue {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
            status_channel: async_channel::unbounded(),
        }
    }
}
impl ReprojectTextureTaskQueue {
    pub fn count(&self) -> usize {
        self.map.len()
    }
    pub fn get(&self, key: &TileKey) -> Option<&ReprojectTextureTask> {
        self.map.get(key)
    }
    pub fn new() -> Self {
        Self::default()
    }
    pub fn push(&mut self, v: ReprojectTextureTask) {
        self.map.insert(v.key, v);
    }
    pub fn clear(&mut self) {
        self.map.clear()
    }
    pub fn clone_status_channel(&self) -> (Sender<TileKey>, Receiver<TileKey>) {
        self.status_channel.clone()
    }
}
fn extract_reproject_texture_task_queue(
    main_world_queue: Extract<Res<ReprojectTextureTaskQueue>>,
    mut render_world_queue: ResMut<ReprojectTextureTaskQueue>,
) {
    render_world_queue.clear();
    main_world_queue
        .map
        .iter()
        .for_each(|x| render_world_queue.push(x.1.clone()))
}
pub struct ReprojectTextureTask {
    pub key: TileKey,
    pub image: Handle<Image>,
    pub output_texture: Handle<Image>,
    pub webmercartor_buffer: Buffer,
}
impl Clone for ReprojectTextureTask {
    fn clone(&self) -> Self {
        Self {
            image: self.image.clone(),
            key: self.key.clone(),
            output_texture: self.output_texture.clone(),
            webmercartor_buffer: self.webmercartor_buffer.clone(),
        }
    }
}
#[derive(Resource)]
pub struct ReprojectTexturePipeline {
    texture_bind_group_layout: BindGroupLayout,
    pipeline: CachedRenderPipelineId,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl FromWorld for ReprojectTexturePipeline {
    fn from_world(world: &mut World) -> Self {
        let texture_bind_group_layout =
            world
                .resource::<RenderDevice>()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: StorageTextureAccess::ReadWrite,
                            format: TextureFormat::Rgba8Unorm,
                            view_dimension: TextureViewDimension::D2,
                        },
                        count: None,
                    }],
                });
        let shader = world
            .resource::<AssetServer>()
            .load("reproject_webmecator.wgsl");
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = pipeline_cache.queue_render_pipeline(RenderPipelineDescriptor {
            label: Some(Cow::from("reproject_texture")),
            layout: vec![texture_bind_group_layout.clone()],
            push_constant_ranges: Vec::new(),
            vertex: VertexState {
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("vertex_main"),
                buffers: vec![],
            },
            fragment: Some(FragmentState {
                shader: shader.clone(),
                shader_defs: vec![],
                entry_point: Cow::from("fragment_main"),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
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
        let render_queue = world.resource::<RenderQueue>();
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
            label: Some("indices_buffer"),
            contents: cast_slice(&positions),
            usage: BufferUsages::VERTEX,
        });
        let indicesAndEdgesCache = world.resource::<IndicesAndEdgesCacheArc>();
        let indices = indicesAndEdgesCache
            .0
            .clone()
            .lock()
            .unwrap()
            .getRegularGridIndices(2, 64);
        let index_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("indices_buffer"),
            contents: cast_slice(&indices),
            usage: BufferUsages::VERTEX,
        });

        ReprojectTexturePipeline {
            texture_bind_group_layout,
            pipeline,
            index_buffer,
            vertex_buffer,
        }
    }
}
#[derive(Default)]
struct ReprojectTextureNode;
impl render_graph::Node for ReprojectTextureNode {
    fn run(
        &self,
        graph: &mut render_graph::RenderGraphContext,
        render_context: &mut bevy::render::renderer::RenderContext,
        world: &World,
    ) -> Result<(), render_graph::NodeRunError> {
        let pipeline_cache = world.resource::<PipelineCache>();
        let pipeline = world.resource::<ReprojectTexturePipeline>();
        let task_queue = world.resource::<ReprojectTextureTaskQueue>();
        let gpu_images = world.resource::<RenderAssets<Image>>();
        let render_pipeline = pipeline_cache
            .get_render_pipeline(pipeline.pipeline)
            .expect("reproject texture pipeline");

        for task in task_queue.map.iter() {
            let view = &gpu_images.get(&task.1.image).expect("task.image");
            let output_texture = &gpu_images
                .get(&task.1.output_texture)
                .expect("task.output_texture");
            let bind_group =
                render_context
                    .render_device()
                    .create_bind_group(&BindGroupDescriptor {
                        label: None,
                        layout: &pipeline.texture_bind_group_layout,
                        entries: &[BindGroupEntry {
                            binding: 0,
                            resource: BindingResource::TextureView(&view.texture_view),
                        }],
                    });
            let mut render_pass =
                render_context
                    .command_encoder()
                    .begin_render_pass(&RenderPassDescriptor {
                        label: "reproject_texture".into(),
                        color_attachments: &[Some(RenderPassColorAttachment {
                            view: &output_texture.texture_view,
                            resolve_target: None,
                            ops: Operations::default(),
                        })],
                        depth_stencil_attachment: None,
                    });
            render_pass.set_pipeline(render_pipeline);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_vertex_buffer(0, *pipeline.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, *task.1.webmercartor_buffer.slice(..));
            render_pass.set_index_buffer(*pipeline.index_buffer.slice(..), IndexFormat::Uint32);
            render_pass.draw(0..3, 0..1);
        }
        Ok(())
    }
}
