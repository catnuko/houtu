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
    pub web_mercator_t: Vec<f32>,
    pub alpha: Vec<f32>,
    pub night_alpha: Vec<f32>,
    pub day_alpha: Vec<f32>,
    pub brightness: Vec<f32>,
    pub contrast: Vec<f32>,
    pub hue: Vec<f32>,
    pub saturation: Vec<f32>,
    pub one_over_gamma: Vec<f32>,
    pub has_web_mercator_t: bool,
    pub quantization_bits12: bool,
    pub scale_and_bias: Mat4,
    pub min_max_height: Vec2,
    pub center_3d: Vec3,
    pub mvp: Mat4,
}
impl TerrainMeshMaterial {
    pub const ATTRIBUTE_WEB_MERCATOR_T: MeshVertexAttribute =
        MeshVertexAttribute::new("ATTRIBUTE_WEB_MERCATOR_T", 1000, VertexFormat::Float32);
    pub const POSITION_3D_AND_HEIGHT: MeshVertexAttribute =
        MeshVertexAttribute::new("POSITION_3D_AND_HEIGHT", 1001, VertexFormat::Float32x4);
    pub const TEXTURE_COORD_AND_ENCODED_NORMALS: MeshVertexAttribute = MeshVertexAttribute::new(
        "TEXTURE_COORD_AND_ENCODED_NORMALS",
        1002,
        VertexFormat::Float32x4,
    );
    pub const COMPRESSED_0: MeshVertexAttribute =
        MeshVertexAttribute::new("COMPRESSED_0", 1003, VertexFormat::Float32x4);
    pub const COMPRESSED_1: MeshVertexAttribute =
        MeshVertexAttribute::new("COMPRESSED_1", 1004, VertexFormat::Float32);
    pub const ATTRIBUTE_POSITION_HEIGHT: MeshVertexAttribute =
        MeshVertexAttribute::new("ATTRIBUTE_POSITION_HEIGHT", 1005, VertexFormat::Float32x4);
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
        let data = _key.bind_group_data;
        if _descriptor.fragment.is_none() {
            info!("no fragment,{:?}", _descriptor.label);
            let mut attributes = vec![
                TerrainMeshMaterial::ATTRIBUTE_POSITION_HEIGHT.at_shader_location(0),
                Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
                TerrainMeshMaterial::ATTRIBUTE_WEB_MERCATOR_T.at_shader_location(2),
            ];
            let attribute_real = _layout.get_layout(&attributes).unwrap();
            _descriptor.vertex.buffers = vec![attribute_real];
            return Ok(());
        }
        let fragment_shader_defs = &mut _descriptor.fragment.as_mut().unwrap().shader_defs;

        if data.apply_brightness {
            fragment_shader_defs.push("APPLY_BRIGHTNESS".into());
        }
        if data.apply_contrast {
            fragment_shader_defs.push("APPLY_CONTRAST".into());
        }
        if data.apply_hue {
            fragment_shader_defs.push("APPLY_HUE".into());
        }
        if data.apply_saturation {
            fragment_shader_defs.push("APPLY_SATURATION".into());
        }
        if data.apply_gamma {
            fragment_shader_defs.push("APPLY_GAMMA".into());
        }
        if data.apply_alpha {
            fragment_shader_defs.push("APPLY_ALPHA".into());
        }
        if data.apply_day_night_alpha {
            fragment_shader_defs.push("APPLY_DAY_NIGHT_ALPHA".into());
        }
        let vertex_shader_defs = &mut _descriptor.vertex.shader_defs;
        if data.apply_quantization_bits12 {
            vertex_shader_defs.push("QUANTIZATION_BITS12".into());
            let attributes = vec![
                TerrainMeshMaterial::COMPRESSED_0.at_shader_location(0),
                // TerrainMeshMaterial::COMPRESSED_1.at_shader_location(1),
            ];
            let attribute_real = _layout.get_layout(&attributes).unwrap();
            _descriptor.vertex.buffers = vec![attribute_real];
        } else {
            let mut attributes = vec![
                TerrainMeshMaterial::ATTRIBUTE_POSITION_HEIGHT.at_shader_location(0),
                Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
                TerrainMeshMaterial::ATTRIBUTE_WEB_MERCATOR_T.at_shader_location(2),
            ];
            let attribute_real = _layout.get_layout(&attributes).unwrap();
            _descriptor.vertex.buffers = vec![attribute_real];
        }

        Ok(())
    }
}
const MAX_TEXTURE_COUNT: usize = 16;
#[derive(ShaderType, Default)]
struct TerrainMeshMaterialUniform {
    translation_and_scale: Vec4,
    coordinate_rectangle: Vec4,
    web_mercator_t: f32,
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
    apply_quantization_bits12: bool,
    apply_webmercator_t: bool,
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
        let mut apply_quantization_bits12 = self.quantization_bits12;
        let mut apply_webmercator_t = self.has_web_mercator_t;

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
            buffer_data.push(self.web_mercator_t[index]);

            buffer_data.push(self.alpha[index]);
            apply_alpha = apply_alpha || self.alpha[index] != 1.0;

            buffer_data.push(self.night_alpha[index]);
            apply_day_night_alpha = apply_day_night_alpha || self.night_alpha[index] != 1.0;

            // 耗费了四天时间，查这个渲染的bug，原来是这里多传个f32
            // 导致传入的数据比着色器中的TerrainMaterialUniform多了4个字节，以至于第二次及其之后的循环的数据都不对
            // TODO alpha暂时在着色器中用不到，先不管
            // buffer_data.push(self.day_alpha[index]);
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
        // info!("webmercatort is {:?}",self.web_mercator_t);
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
        let mut vertex_uniform_buffer_data = vec![0.0; 40];
        let mut mvp_slice = vec![0.0; 16];
        self.mvp.write_cols_to_slice(&mut mvp_slice);
        if self.quantization_bits12 {
            let mut slice = [0.0; 16];
            self.scale_and_bias.write_cols_to_slice(&mut slice);
            let mut res = vec![
                self.min_max_height.x,
                self.min_max_height.y,
                0.0_f32,
                0.0_f32,
                self.center_3d.x,
                self.center_3d.y,
                self.center_3d.z,
                0.0_f32,
            ];
            res.extend_from_slice(&slice);
            res.extend_from_slice(&mvp_slice);
            vertex_uniform_buffer_data = res;
        } else {
            let mut res = vec![
                0f32,
                0f32,
                0f32,
                0f32,
                self.center_3d.x,
                self.center_3d.y,
                self.center_3d.z,
                0.0_f32,
            ];
            let scale_bias = [0.0; 16];
            res.extend_from_slice(&scale_bias);
            res.extend_from_slice(&mvp_slice);
            vertex_uniform_buffer_data = res;
        };
        let vertex_uniform_buffer =
            render_device.create_buffer_with_data(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex_uniform_buffer"),
                contents: bytemuck::cast_slice(&vertex_uniform_buffer_data),
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
                BindGroupEntry {
                    binding: 4,
                    resource: vertex_uniform_buffer.as_entire_binding(),
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
                apply_quantization_bits12,
                apply_webmercator_t,
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
                BindGroupLayoutEntry {
                    binding: 4,
                    visibility: ShaderStages::VERTEX,
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
