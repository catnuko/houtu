use std::{hash::Hash, vec};

use bevy::{
    ecs::{
        query::ROQueryItem,
        system::{
            lifetimeless::{Read, SRes},
            SystemParamItem,
        },
    },
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, RenderMaterials, SetMaterialBindGroup,
        SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{MeshVertexBufferLayout, GpuBufferInfo},
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
    },
};

use crate::render::terrian_material::TerrainMeshMaterial;

use super::{terrain_plugin::TerrainBindGroup, TileRendered, TERRAIN_MATERIAN_SHADER_HANDLE};
#[derive(PartialEq, Eq, Clone, Hash)]
pub struct ShaderDefines {
    pub apply_brightness: bool,
    pub apply_contrast: bool,
    pub apply_hue: bool,
    pub apply_saturation: bool,
    pub apply_gamma: bool,
    pub apply_alpha: bool,
    pub apply_day_night_alpha: bool,
    pub apply_split: bool,
    pub apply_cutout: bool,
    pub apply_color_to_alpha: bool,
    pub apply_quantization_bits12: bool,
    pub apply_webmercator_t: bool,
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct TerrainPipelineKey {
    pub shader_defines: ShaderDefines,
}
#[derive(Component)]
pub struct TerrainPipelineId(CachedRenderPipelineId);
#[derive(Resource)]
pub struct TerrainPipeline {
    pub layout: BindGroupLayout,
}
impl FromWorld for TerrainPipeline {
    fn from_world(world: &mut World) -> Self {
        let render_device = world.resource::<RenderDevice>();
        let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("terrain_pipeline_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
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
        Self { layout }
    }
}

impl SpecializedMeshPipeline for TerrainPipeline {
    type Key = TerrainPipelineKey;
    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let data = key.shader_defines;
        let mut fragment_shader_defs = Vec::new();
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
        let mut vertex_shader_defs = Vec::new();
        let mut vertex_buffer = Vec::new();
        if data.apply_quantization_bits12 {
            vertex_shader_defs.push("QUANTIZATION_BITS12".into());
            let attributes = vec![
                TerrainMeshMaterial::COMPRESSED_0.at_shader_location(0),
                // TerrainMeshMaterial::COMPRESSED_1.at_shader_location(1),
            ];
            let attribute_real = layout.get_layout(&attributes).unwrap();
            vertex_buffer.push(attribute_real)
        } else {
            let attributes = vec![
                TerrainMeshMaterial::ATTRIBUTE_POSITION_HEIGHT.at_shader_location(0),
                Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
                TerrainMeshMaterial::ATTRIBUTE_WEB_MERCATOR_T.at_shader_location(2),
            ];
            let attribute_real = layout.get_layout(&attributes).unwrap();
            vertex_buffer.push(attribute_real)
        }
        let descriptor = RenderPipelineDescriptor {
            vertex: VertexState {
                shader: TERRAIN_MATERIAN_SHADER_HANDLE,
                entry_point: "vertex".into(),
                shader_defs: vertex_shader_defs,
                buffers: vertex_buffer,
            },
            fragment: Some(FragmentState {
                shader: TERRAIN_MATERIAN_SHADER_HANDLE,
                entry_point: "fragment".into(),
                shader_defs: fragment_shader_defs,
                targets: vec![],
            }),
            layout: vec![self.layout.clone()],
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::GreaterEqual,
                stencil: StencilState {
                    front: StencilFaceState::IGNORE,
                    back: StencilFaceState::IGNORE,
                    read_mask: 0,
                    write_mask: 0,
                },
                bias: DepthBiasState {
                    constant: 0,
                    slope_scale: 0.0,
                    clamp: 0.0,
                },
            }),
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            push_constant_ranges: vec![],
            label: Some("terrain_pipeline".into()),
        };
        return Ok(descriptor);
    }
}
pub struct SetTerrainBindGroup<const I: usize>;
impl<P: PhaseItem, const I: usize> RenderCommand<P> for SetTerrainBindGroup<I> {
    type Param = ();
    type ViewWorldQuery = ();
    type ItemWorldQuery = (bevy::ecs::system::lifetimeless::Read<TerrainBindGroup>);

    #[inline]
    fn render<'w>(
        _item: &P,
        _view: ROQueryItem<'w, Self::ViewWorldQuery>,
        (terrain_bind_group): ROQueryItem<'w, Self::ItemWorldQuery>,
        _: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        pass.set_bind_group(I, &terrain_bind_group.bind_group, &[]);
        RenderCommandResult::Success
    }
}
/// The draw function of the terrain. It sets the pipeline and the bind groups and then issues the
/// draw call.0
pub(crate) type DrawTerrain = (
    SetItemPipeline,
    // SetMeshViewBindGroup<0>,
    SetTerrainBindGroup<0>,
    // SetMeshBindGroup<1>,
    DrawTerrainCommand
);

pub struct DrawTerrainCommand;

impl<P: PhaseItem> RenderCommand<P> for DrawTerrainCommand {
    type ViewWorldQuery = ();
    type ItemWorldQuery = (Read<Handle<Mesh>>);
    type Param = SRes<RenderAssets<Mesh>>;
    fn render<'w>(
        item: &P,
        view: ROQueryItem<'w, Self::ViewWorldQuery>,
        mesh_handle: ROQueryItem<'w, Self::ItemWorldQuery>,
        meshes: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        if let Some(gpu_mesh) = meshes.into_inner().get(mesh_handle) {
            pass.set_vertex_buffer(0, gpu_mesh.vertex_buffer.slice(..));
            match &gpu_mesh.buffer_info {
                GpuBufferInfo::Indexed {
                    buffer,
                    index_format,
                    count,
                } => {
                    pass.set_index_buffer(buffer.slice(..), 0, *index_format);
                    pass.draw_indexed(0..*count, 0, 0..1);
                }
                GpuBufferInfo::NonIndexed => {
                    pass.draw(0..gpu_mesh.vertex_count, 0..1);
                }
            }
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}
