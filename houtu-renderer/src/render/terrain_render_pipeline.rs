use std::hash::Hash;
use std::marker::PhantomData;

use bevy::{
    asset::load_internal_asset,
    core_pipeline::core_3d::Opaque3d,
    ecs::{
        query::ROQueryItem,
        system::{lifetimeless::SRes, SystemParamItem},
    },
    pbr::{
        DrawMesh, MeshPipeline, MeshPipelineKey, RenderMaterials, SetMaterialBindGroup,
        SetMeshBindGroup, SetMeshViewBindGroup,
    },
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::MeshVertexBufferLayout,
        render_asset::RenderAssets,
        render_phase::{
            AddRenderCommand, DrawFunctions, PhaseItem, RenderCommand, RenderCommandResult,
            RenderPhase, SetItemPipeline, TrackedRenderPass,
        },
        render_resource::*,
        renderer::RenderDevice,
        texture::BevyDefault,
        view::ExtractedView,
        RenderApp, RenderSet,
    },
};

use crate::render::{
    terrain_data_render::{extract_terrain_config, prepare_terrain},
    terrian_material::TerrainMeshMaterial,
};

use super::{node_atlas::ShaderDefines, terrain_data_render::TerrainBindGroup, TileRendered, terrain_data::finish_loading_attachment_from_disk};

#[derive(PartialEq, Eq, Hash, Clone)]
pub struct TerrainPipelineKey {
    pub shader_defines: ShaderDefines,
    pub mesh_pipeline_key: MeshPipelineKey,
}
#[derive(Resource)]
pub struct TerrainRenderPipeline {
    shader: Handle<Shader>,
    mesh_pipeline: MeshPipeline,
    pub layout: BindGroupLayout,
}
impl FromWorld for TerrainRenderPipeline {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        let shader = asset_server.load("terrain_material.wgsl");
        let render_device = world.resource::<RenderDevice>();
        let layout = terrain_bind_group_layout(&render_device);
        let mesh_pipeline = world.resource::<MeshPipeline>();
        Self {
            layout,
            shader,
            mesh_pipeline: mesh_pipeline.clone(),
        }
    }
}

impl SpecializedMeshPipeline for TerrainRenderPipeline {
    type Key = TerrainPipelineKey;
    fn specialize(
        &self,
        key: Self::Key,
        layout: &MeshVertexBufferLayout,
    ) -> Result<RenderPipelineDescriptor, SpecializedMeshPipelineError> {
        let mut descriptor = self
            .mesh_pipeline
            .specialize(key.mesh_pipeline_key, layout)?;
        descriptor
            .vertex
            .shader_defs
            .push("MESH_BINDGROUP_1".into());
        descriptor.layout.splice(1..1, [self.layout.clone()]);
        descriptor.vertex.shader = self.shader.clone();
        descriptor.fragment.as_mut().unwrap().shader = self.shader.clone();
        descriptor = update_pipeline_layout(descriptor, key, layout);
        return Ok(descriptor);
    }
}
pub struct TerrainRenderPlugin;

impl Plugin for TerrainRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, finish_loading_attachment_from_disk);
    }
    fn finish(&self, app: &mut App) {
        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                .add_render_command::<Opaque3d, DrawTerrain>()
                .init_resource::<TerrainRenderPipeline>()
                .init_resource::<SpecializedMeshPipelines<TerrainRenderPipeline>>()
                .add_systems(ExtractSchedule, extract_terrain_config)
                .add_systems(
                    bevy::render::Render,
                    (
                        prepare_terrain.in_set(RenderSet::Prepare),
                        queue_terrain.in_set(RenderSet::Queue),
                    ),
                );
        }
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
/// draw call.
pub(crate) type DrawTerrain = (
    SetItemPipeline,
    SetMeshViewBindGroup<0>,
    SetTerrainBindGroup<1>,
    SetMeshBindGroup<2>,
    DrawMesh,
);
/// Queses all terrain entities for rendering via the terrain pipeline.
pub(crate) fn queue_terrain(
    draw_functions: Res<DrawFunctions<Opaque3d>>,
    terrain_pipeline: Res<TerrainRenderPipeline>,
    pipeline_cache: Res<PipelineCache>,
    msaa: Res<Msaa>,
    mut pipelines: ResMut<SpecializedMeshPipelines<TerrainRenderPipeline>>,
    mut view_query: Query<(&ExtractedView, &mut RenderPhase<Opaque3d>)>,
    render_meshes: Res<RenderAssets<Mesh>>,
    terrain_query: Query<(Entity, &Handle<Mesh>, &TerrainBindGroup)>,
) {
    let draw_function = draw_functions.read().get_id::<DrawTerrain>().unwrap();
    let msaa_key = MeshPipelineKey::from_msaa_samples(msaa.samples());

    for (view, mut opaque_phase) in view_query.iter_mut() {
        let view_key = msaa_key | MeshPipelineKey::from_hdr(view.hdr);

        for (entity, mesh_handle, terrain_bind_group) in terrain_query.iter() {
            if let Some(mesh) = render_meshes.get(mesh_handle) {
                let mesh_key =
                    view_key | MeshPipelineKey::from_primitive_topology(mesh.primitive_topology);
                let key = TerrainPipelineKey {
                    shader_defines: terrain_bind_group.shader_defines.clone(),
                    //TODO 可能不能正确反映Pipeline的变化
                    mesh_pipeline_key: mesh_key,
                };
                let pipeline_id =
                    pipelines.specialize(&pipeline_cache, &terrain_pipeline, key, &mesh.layout);
                let pipeline_id = match pipeline_id {
                    Ok(id) => id,
                    Err(err) => {
                        error!("{}", err);
                        return;
                    }
                };
                opaque_phase.add(Opaque3d {
                    entity,
                    pipeline: pipeline_id,
                    draw_function,
                    distance: f32::MIN, // draw terrain first
                    batch_range:0..1,
                    dynamic_offset:None
                });
            }
        }
    }
}
fn terrain_bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout {
    let layout = render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("terrain_material_bindgroup_layout"),
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
    return layout;
}
fn update_pipeline_layout(
    mut _descriptor: RenderPipelineDescriptor,
    key: TerrainPipelineKey,
    _layout: &MeshVertexBufferLayout,
) -> RenderPipelineDescriptor {
    let data = key.shader_defines;
    if _descriptor.fragment.is_none() {
        info!("no fragment,{:?}", _descriptor.label);
        let mut attributes = vec![
            TerrainMeshMaterial::ATTRIBUTE_POSITION_HEIGHT.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            TerrainMeshMaterial::ATTRIBUTE_WEB_MERCATOR_T.at_shader_location(2),
        ];
        let attribute_real = _layout.get_layout(&attributes).unwrap();
        _descriptor.vertex.buffers = vec![attribute_real];
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
    return _descriptor;
}
