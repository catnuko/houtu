use bevy::{
    asset::LoadState,
    core_pipeline::core_3d::Opaque3d,
    pbr::MeshPipelineKey,
    prelude::*,
    render::{
        render_asset::RenderAssets,
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase},
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::ExtractedView,
        Extract, RenderApp, RenderSet,
    },
};

use super::{
    terrain_bundle::TerrainConfig,
    terrain_pipeline::{DrawTerrain, ShaderDefines, TerrainPipeline, TerrainPipelineKey},
};

#[derive(Clone, Default, Component, ShaderType)]
pub struct TerrainConfigUniform {
    pub minimum_height: f32,
    pub maximum_height: f32,
    pub center_3d: Vec3,
    pub scale_and_bias: Mat4,
    pub mvp: Mat4,
}
impl From<&TerrainConfig> for TerrainConfigUniform {
    fn from(config: &TerrainConfig) -> Self {
        Self {
            minimum_height: config.minimum_height,
            maximum_height: config.maximum_height,
            center_3d: config.center_3d,
            scale_and_bias: config.scale_and_bias,
            mvp: config.mvp,
        }
    }
}

#[derive(Clone)]
pub struct TerrainAttachment {
    pub handle: Handle<Image>,
    pub translation_and_scale: Vec4,
    pub coordinate_rectangle: Vec4,
    pub web_mercator_t: f32,
    pub alpha: f32,
    pub day_alpha: f32,
    pub night_alpha: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub hue: f32,
    pub saturation: f32,
    pub one_over_gamma: f32,
    pub width: u32,
    pub height: u32,
}
#[derive(Component)]
pub struct GpuNodeAtlas {
    pub attachments: Vec<TerrainAttachment>,
    pub array_texture: Texture,
    pub texture_size: UVec3,
    pub quantization_bits12: bool,
    pub has_web_mercator_t: bool,
}

impl GpuNodeAtlas {
    pub fn create_texture_view(&self) -> TextureView {
        self.array_texture.create_view(&TextureViewDescriptor {
            label: Some("array_texture_view"),
            dimension: Some(TextureViewDimension::D2Array),
            array_layer_count: Some(self.texture_size.z),
            base_array_layer: 0,
            ..Default::default()
        })
    }
}
#[derive(Component)]
pub struct TerrainBindGroup {
    pub bind_group: BindGroup,
    // pub shader_defines: ShaderDefines,
    pub pipeline_id: CachedRenderPipelineId,
}
pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, finish_loading_attachment_from_disk);
    }
    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Opaque3d, DrawTerrain>()
                .init_resource::<TerrainPipeline>()
                .init_resource::<SpecializedMeshPipelines<TerrainPipeline>>()
                .add_systems(ExtractSchedule, extract_terrain_config)
                .add_systems(
                    bevy::render::Render,
                    (
                        prepare_terrain.in_set(RenderSet::Queue),
                        queue_terrain.after(RenderSet::Prepare),
                    ),
                );
        }
    }
}
fn finish_loading_attachment_from_disk(
    mut images: ResMut<Assets<Image>>,
    mut terrain_query: Query<&mut TerrainConfig>,
    server: Res<AssetServer>,
) {
    for config in terrain_query.iter_mut() {
        for attachment in &config.attachments {
            let state = server.get_load_state(&attachment.handle);
            match state {
                Some(LoadState::Failed) => {
                    info!("Image loading failure")
                }
                Some(LoadState::Loaded) => {
                    let image = images.get_mut(&attachment.handle).unwrap();
                    image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
                }
                _ => {}
            }
        }
    }
}
fn extract_terrain_config(
    mut command: Commands,
    device: Res<RenderDevice>,
    queue: Res<RenderQueue>,
    query: Extract<Query<(Entity, &TerrainConfig, &Handle<Mesh>)>>,
) {
    for (entity, terrain_config, handle) in query.iter() {
        let terrain_config_uniform: TerrainConfigUniform = terrain_config.into();
        let texture = terrain_config.create(&device, &queue);
        let gpu_node_atlas = GpuNodeAtlas {
            attachments: terrain_config.attachments.clone(),
            array_texture: texture,
            texture_size: terrain_config.get_array_texture_size(),
            quantization_bits12: terrain_config.quantization_bits12,
            has_web_mercator_t: terrain_config.has_web_mercator_t,
        };
        command.spawn((terrain_config_uniform, gpu_node_atlas, handle.clone()));
    }
}

/// Queses all terrain entities for rendering via the terrain pipeline.
fn queue_terrain(
    draw_functions: Res<DrawFunctions<Opaque3d>>,
    mut view_query: Query<(&ExtractedView, &mut RenderPhase<Opaque3d>)>,
    terrain_query: Query<(Entity, &TerrainBindGroup)>,
) {
    let draw_function = draw_functions.read().get_id::<DrawTerrain>().unwrap();
    for (view, mut opaque_phase) in view_query.iter_mut() {
        for (entity, terrain_bind_group) in terrain_query.iter() {
            opaque_phase.add(Opaque3d {
                entity,
                pipeline: terrain_bind_group.pipeline_id,
                draw_function,
                distance: f32::MIN, // draw terrain first
                batch_range: 0..1,
                dynamic_offset: None,
            });
        }
    }
    println!("terrain_query,{}", terrain_query.iter().len());
}

fn prepare_terrain(
    mut commands: Commands,
    images: Res<RenderAssets<Image>>,
    mut queue: ResMut<RenderQueue>,
    mut query: Query<(
        Entity,
        &TerrainConfigUniform,
        &mut GpuNodeAtlas,
        &Handle<Mesh>,
    )>,
    render_device: Res<RenderDevice>,
    terrain_pipeline: Res<TerrainPipeline>,
    pipeline_cache: Res<PipelineCache>,
    render_meshes: Res<RenderAssets<Mesh>>,
    mut pipelines: ResMut<SpecializedMeshPipelines<TerrainPipeline>>,
) {
    let mut command_encoder =
        render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    for (entity, terrain_config_uniform, mut gpu_node_atlas, mesh_handle) in &mut query {
        update(&mut gpu_node_atlas, &mut command_encoder, &images);
        let mut buffer = encase::UniformBuffer::new(Vec::new());
        buffer.write(terrain_config_uniform).unwrap();
        let vertex_uniform_buffer = render_device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("vertex_uniform_buffer"),
            usage: BufferUsages::UNIFORM,
            contents: &buffer.into_inner(),
        });
        let sampler = render_device.create_sampler(&SamplerDescriptor::default());
        let (uniform_buffer, state_uniform_buffer, shader_defines) =
            other_uniform(&gpu_node_atlas, &render_device);
        let bind_group = render_device.create_bind_group(
            Some("terrain_material"),
            &terrain_pipeline.layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&gpu_node_atlas.create_texture_view()),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: state_uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 4,
                    resource: vertex_uniform_buffer.as_entire_binding(),
                },
            ],
        );
        let key = TerrainPipelineKey {
            shader_defines: shader_defines,
        };
        if let Some(mesh) = render_meshes.get(mesh_handle) {
            let pipeline_id =
                pipelines.specialize(&pipeline_cache, &terrain_pipeline, key, &mesh.layout);
            let pipeline_id = match pipeline_id {
                Ok(id) => id,
                Err(err) => {
                    error!("{}", err);
                    return;
                }
            };
            commands.entity(entity).insert(TerrainBindGroup {
                bind_group: bind_group,
                pipeline_id,
            });
        }
    }
    queue.submit(vec![command_encoder.finish()]);
}

fn update(
    gpu_node_atlas: &mut GpuNodeAtlas,
    command_encoder: &mut CommandEncoder,
    images: &RenderAssets<Image>,
) {
    for (index, attachment) in gpu_node_atlas.attachments.iter().enumerate() {
        let index = index as u32;
        if let Some(atlas_attachment) = images.get(&attachment.handle) {
            command_encoder.copy_texture_to_texture(
                ImageCopyTexture {
                    texture: &atlas_attachment.texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                ImageCopyTexture {
                    texture: &gpu_node_atlas.array_texture,
                    mip_level: 0,
                    origin: Origin3d {
                        x: 0,
                        y: 0,
                        z: index,
                    },
                    aspect: TextureAspect::All,
                },
                Extent3d {
                    width: atlas_attachment.texture.width(),
                    height: atlas_attachment.texture.height(),
                    depth_or_array_layers: 1,
                },
            );
            // info!("copy over")
        } else {
            error!("Something went wrong, attachment is not available!")
        }
    }
}

fn other_uniform(
    gpu_node_atlas: &GpuNodeAtlas,
    render_device: &RenderDevice,
) -> (Buffer, Buffer, ShaderDefines) {
    let mut buffer_data = vec![];
    let mut apply_brightness = false;
    let mut apply_contrast = false;
    let mut apply_hue = false;
    let mut apply_saturation = false;
    let mut apply_gamma = false;
    let mut apply_alpha = false;
    let mut apply_day_night_alpha = false;
    let mut apply_split = false;
    let mut apply_cutout = false;
    let mut apply_color_to_alpha = false; //
    let mut apply_quantization_bits12 = gpu_node_atlas.quantization_bits12;
    let mut apply_webmercator_t = gpu_node_atlas.has_web_mercator_t; //
    for attachment in &gpu_node_atlas.attachments {
        let translation_and_scale = attachment.translation_and_scale;
        buffer_data.push(translation_and_scale.x);
        buffer_data.push(translation_and_scale.y);
        buffer_data.push(translation_and_scale.z);
        buffer_data.push(translation_and_scale.w);
        let coordinate_rectangle = attachment.coordinate_rectangle;
        buffer_data.push(coordinate_rectangle.x);
        buffer_data.push(coordinate_rectangle.y);
        buffer_data.push(coordinate_rectangle.z);
        buffer_data.push(coordinate_rectangle.w);
        buffer_data.push(attachment.web_mercator_t);

        buffer_data.push(attachment.alpha);
        apply_alpha = apply_alpha || attachment.alpha != 1.0;

        buffer_data.push(attachment.night_alpha);
        apply_day_night_alpha = apply_day_night_alpha || attachment.night_alpha != 1.0;

        // 耗费了四天时间，查这个渲染的bug，原来是这里多传个f32
        // 导致传入的数据比着色器中的TerrainMaterialUniform多了4个字节，以至于第二次及其之后的循环的数据都不对
        // TODO alpha暂时在着色器中用不到，先不管
        // buffer_data.push(attachment.day_alpha);
        apply_day_night_alpha = apply_day_night_alpha || attachment.day_alpha != 1.0; //

        buffer_data.push(attachment.brightness);
        apply_brightness = apply_brightness || attachment.brightness != 1.0;

        buffer_data.push(attachment.contrast);
        apply_contrast = apply_contrast || attachment.contrast != 1.0; //

        buffer_data.push(attachment.hue);
        apply_hue = apply_hue || attachment.hue != 1.0; //sdfsfsdf

        buffer_data.push(attachment.saturation);
        apply_saturation = apply_saturation || attachment.saturation != 1.0;

        buffer_data.push(attachment.one_over_gamma);
        apply_gamma = apply_gamma || attachment.one_over_gamma != 1.0;
    }
    let uniform_buffer = render_device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
        label: Some("uniform_buffer"),
        contents: bytemuck::cast_slice(&buffer_data), //
        usage: wgpu::BufferUsages::STORAGE,           //
    });
    let state_uniform_buffer_data = vec![gpu_node_atlas.attachments.len() as i32];

    let state_uniform_buffer =
        render_device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
            label: Some("state_uniform_buffer"), //
            contents: bytemuck::cast_slice(&state_uniform_buffer_data),
            usage: wgpu::BufferUsages::UNIFORM,
        });
    let shader_defines = ShaderDefines {
        apply_brightness,
        apply_contrast,
        apply_hue,
        apply_saturation,
        apply_gamma,
        apply_alpha,
        apply_day_night_alpha,
        apply_split,
        apply_cutout,
        apply_color_to_alpha,
        apply_quantization_bits12,
        apply_webmercator_t,
    };
    return (uniform_buffer, state_uniform_buffer, shader_defines);
}
