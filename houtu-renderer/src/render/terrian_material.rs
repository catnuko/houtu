use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    prelude::*,
    reflect::TypeUuid,
    render::{
        extract_component::ExtractComponent,
        mesh::{MeshVertexAttribute, MeshVertexBufferLayout},
        render_asset::{self, RenderAssets},
        render_resource::{
            self, AsBindGroup, AsBindGroupError, BindGroupLayout, PreparedBindGroup,
            RenderPipelineDescriptor, Sampler, ShaderRef, ShaderType, SpecializedMeshPipelineError,
        },
        renderer::{self, RenderDevice},
        texture::FallbackImage,
    },
};
use wgpu::{
    BindGroupEntry, BindGroupLayoutEntry, BufferBinding, Extent3d, ShaderStages, TextureDescriptor,
    VertexFormat,
};
/// A marker component used to identify a terrain entity.
#[derive(Clone, Copy, Component, ExtractComponent)]
pub struct Terrain;
#[derive(Default, TypeUuid, Debug, Clone, Resource)]
#[uuid = "886f4558-1621-492a-856e-ea1dbc9902d9"]
pub struct TerrainMeshMaterial {
    pub textures: Vec<Handle<Image>>,
    pub texture_width: u32,
    pub texture_height: u32,
    pub translation_and_scale: Vec<Vec4>,
    pub coordinate_rectangle: Vec<Vec4>,
    pub use_web_mercator_t: Vec<f32>,
    pub alpha: Vec<f32>,
    pub night_alpha: Vec<f32>,
    pub day_alpha: Vec<f32>,
    pub brightness: Vec<f32>,
    pub contrast: Vec<f32>,
    pub hue: Vec<f32>,
    pub saturation: Vec<f32>,
    pub one_over_gamma: Vec<f32>,
}
impl Material for TerrainMeshMaterial {
    fn fragment_shader() -> ShaderRef {
        "terrain_material.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "terrain_material.wgsl".into()
    }
    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        _descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        // descriptor.primitive.polygon_mode = PolygonMode::Line;
        let data = _key.bind_group_data;
        let mut shader_defines = vec![];
        if data.apply_brightness {
            shader_defines.push("APPLY_BRIGHTNESS");
        }
        if data.apply_contrast {
            shader_defines.push("APPLY_CONTRAST");
        }
        if data.apply_hue {
            shader_defines.push("APPLY_HUE");
        }
        if data.apply_saturation {
            shader_defines.push("APPLY_SATURATION");
        }
        if data.apply_gamma {
            shader_defines.push("APPLY_GAMMA");
        }
        if data.apply_alpha {
            shader_defines.push("APPLY_ALPHA");
        }
        if data.apply_day_night_alpha {
            shader_defines.push("APPLY_DAY_NIGHT_ALPHA");
        }
        let attribute_real = _layout
            .get_layout(&[
                Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
                Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
                ATTRIBUTE_WEB_MERCATOR_T.at_shader_location(2),
            ])
            .unwrap();
        _descriptor.vertex.buffers = vec![attribute_real];
        Ok(())
    }
}
const MAX_TEXTURE_COUNT: usize = 16;
#[derive(ShaderType, Default)]
struct TerrainMeshMaterialUniform {
    translation_and_scale: Vec4,
    coordinate_rectangle: Vec4,
    use_web_mercator_t: f32,
    alpha: f32,
    night_alpha: f32,
    day_alpha: f32,
    brightness: f32,
    contrast: f32,
    hue: f32,
    saturation: f32,
    one_over_gamma: f32,
}
#[derive(ShaderType, Default)]
pub struct TerrainMeshMaterialUniformList {
    #[size(runtime)]
    data: Vec<TerrainMeshMaterialUniform>,
}
const MAX_TEXTURE: u32 = 5;
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct ShaderDefines {
    apply_brightness: bool,
    apply_contrast: bool,
    apply_hue: bool,
    apply_saturation: bool,
    apply_gamma: bool,
    apply_alpha: bool,
    apply_day_night_alpha: bool,
    apply_split: bool,
    apply_cutout: bool,
    apply_color_to_alpha: bool,
}
impl AsBindGroup for TerrainMeshMaterial {
    type Data = ShaderDefines;
    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        image_assets: &RenderAssets<Image>,
        fallback_image: &FallbackImage,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        let mut images = vec![];
        for (index, handle) in self.textures.iter().enumerate() {
            match image_assets.get(handle) {
                Some(image) => images.push(&*image.texture_view),
                None => return Err(AsBindGroupError::RetryNextUpdate),
            }
        }
        // info!("terrain material image length is {}",images.len());
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
        let mut apply_color_to_alpha = false;
        for (index, _) in self.textures.iter().enumerate() {
            let translation_and_scale = self.translation_and_scale[index];
            buffer_data.push(translation_and_scale.x);
            buffer_data.push(translation_and_scale.y);
            buffer_data.push(translation_and_scale.z);
            buffer_data.push(translation_and_scale.w);
            let coordinate_rectangle = self.coordinate_rectangle[index];
            buffer_data.push(coordinate_rectangle.x);
            buffer_data.push(coordinate_rectangle.y);
            buffer_data.push(coordinate_rectangle.z);
            buffer_data.push(coordinate_rectangle.w);
            buffer_data.push(self.use_web_mercator_t[index]);

            buffer_data.push(self.alpha[index]);
            apply_alpha = apply_alpha || self.alpha[index] != 1.0;

            buffer_data.push(self.night_alpha[index]);
            apply_day_night_alpha = apply_day_night_alpha || self.night_alpha[index] != 1.0;

            buffer_data.push(self.day_alpha[index]);
            apply_day_night_alpha = apply_day_night_alpha || self.day_alpha[index] != 1.0;

            buffer_data.push(self.brightness[index]);
            apply_brightness = apply_brightness || self.brightness[index] != 1.0;

            buffer_data.push(self.contrast[index]);
            apply_contrast = apply_contrast || self.contrast[index] != 1.0;

            buffer_data.push(self.hue[index]);
            apply_hue = apply_hue || self.hue[index] != 1.0;

            buffer_data.push(self.saturation[index]);
            apply_saturation = apply_saturation || self.saturation[index] != 1.0;

            buffer_data.push(self.one_over_gamma[index]);
            apply_gamma = apply_gamma || self.one_over_gamma[index] != 1.0;
        }
        // info!("texture length is {}",self.textures.iter().len());
        // info!("webmercatort is {:?}",self.use_web_mercator_t);
        let uniform_buffer =
            render_device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
                label: Some("uniform_buffer"),
                contents: bytemuck::cast_slice(&buffer_data),
                usage: wgpu::BufferUsages::STORAGE,
            });
        let state_uniform_buffer_data = vec![images.len() as i32];
        let state_uniform_buffer =
            render_device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
                label: Some("state_uniform_buffer"),
                contents: bytemuck::cast_slice(&state_uniform_buffer_data),
                usage: wgpu::BufferUsages::UNIFORM,
            });
        let bind_group = render_device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain_material"),
            layout: layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureViewArray(&images),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&fallback_image.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: uniform_buffer.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 3,
                    resource: state_uniform_buffer.as_entire_binding(),
                },
            ],
        });
        Ok(PreparedBindGroup {
            bindings: vec![],
            bind_group,
            data: ShaderDefines {
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
            },
        })
    }
    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout
    where
        Self: Sized,
    {
        return render_device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain_material_bindgroup_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: NonZeroU32::new(MAX_TEXTURE),
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 3,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
    }
}
pub const ATTRIBUTE_WEB_MERCATOR_T: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_WebMercatorT", 15, VertexFormat::Float32);
