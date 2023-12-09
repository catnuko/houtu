use bevy::{
    ecs::{
        entity,
        query::ROQueryItem,
        system::{lifetimeless::SRes, SystemParamItem},
    },
    math::{DMat4, DVec3},
    prelude::*,
    render::{
        extract_component::{ExtractComponent, ExtractComponentPlugin},
        main_graph::node::CAMERA_DRIVER,
        render_asset::RenderAssets,
        render_resource::*,
        renderer::{RenderDevice, RenderQueue},
        view::NoFrustumCulling,
        Extract, RenderApp, RenderSet,
    },
};

use super::{
    node_atlas::ShaderDefines, node_atlas_render::GpuNodeAtlas, terrain_data::TerrainConfig,
    terrain_render_pipeline::TerrainRenderPipeline,
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
#[derive(Component)]
pub struct TerrainBindGroup {
    pub bind_group: BindGroup,
    pub shader_defines: ShaderDefines,
}
pub fn extract_terrain_config(
    mut command: Commands,
    device: Res<RenderDevice>,
    mut images: ResMut<RenderAssets<Image>>,
    query: Extract<Query<(Entity, &TerrainConfig)>>,
) {
    for (entity, terrain_config) in query.iter() {
        let terrain_config_uniform: TerrainConfigUniform = terrain_config.into();
        //TODO 提取时除了复制，尽量少搞其它逻辑
        let texture = terrain_config.create(&device);
        let gpu_node_atlas = GpuNodeAtlas {
            attachments: terrain_config.attachments.clone(),
            array_texture: texture,
            texture_size: terrain_config.get_array_texture_size(),
            quantization_bits12: terrain_config.quantization_bits12,
            has_web_mercator_t: terrain_config.has_web_mercator_t,
        };
        command
            .get_or_spawn(entity)
            .insert((terrain_config_uniform, gpu_node_atlas));
    }
}
pub fn prepare_terrain(
    mut commands: Commands,
    images: Res<RenderAssets<Image>>,
    mut queue: ResMut<RenderQueue>,
    mut query: Query<(Entity, &TerrainConfigUniform, &mut GpuNodeAtlas)>,
    render_device: Res<RenderDevice>,
    pipeline: Res<TerrainRenderPipeline>,
) {
    let mut command_encoder =
        render_device.create_command_encoder(&CommandEncoderDescriptor::default());
    for (entity, terrain_config_uniform, mut gpu_node_atlas) in &mut query {
        gpu_node_atlas.update(&mut command_encoder, &mut queue, &images);
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
            &pipeline.layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(
                        &gpu_node_atlas.create_texture_view(),
                    ),
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
        commands.entity(entity).insert(TerrainBindGroup {
            bind_group: bind_group,
            shader_defines: shader_defines,
        });
    }
    queue.submit(vec![command_encoder.finish()]);
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
