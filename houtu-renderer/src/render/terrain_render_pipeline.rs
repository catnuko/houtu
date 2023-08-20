use std::hash::Hash;
use std::marker::PhantomData;

use bevy::{
    core_pipeline::core_3d::Opaque3d,
    pbr::{DrawMesh, MeshPipeline, RenderMaterials, SetMaterialBindGroup, SetMeshViewBindGroup},
    prelude::*,
    render::{
        render_phase::{AddRenderCommand, DrawFunctions, RenderPhase, SetItemPipeline},
        render_resource::*,
        renderer::RenderDevice,
        texture::BevyDefault,
        RenderApp, RenderSet,
    },
};

use super::{terrian_material::ShaderDefines, TileRendered};
pub struct TerrainPipelineKey<M: Material> {
    pub bind_group_data: M::Data,
    pub texture_num: u32,
}
impl<M: Material> Eq for TerrainPipelineKey<M> where M::Data: PartialEq {}
impl<M: Material> PartialEq for TerrainPipelineKey<M>
where
    M::Data: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.bind_group_data == other.bind_group_data
    }
}

impl<M: Material> Clone for TerrainPipelineKey<M>
where
    M::Data: Clone,
{
    fn clone(&self) -> Self {
        Self {
            bind_group_data: self.bind_group_data.clone(),
            texture_num: self.texture_num.clone(),
        }
    }
}

impl<M: Material> Hash for TerrainPipelineKey<M>
where
    M::Data: Hash,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.bind_group_data.hash(state);
    }
}

#[derive(Resource)]
pub struct TerrainRenderPipeline<M: Material> {
    pub vertex_shader: Handle<Shader>,
    pub(crate) material_layout: BindGroupLayout,
    pub fragment_shader: Handle<Shader>,
    marker: PhantomData<M>,
}
impl<M: Material> FromWorld for TerrainRenderPipeline<M> {
    fn from_world(world: &mut World) -> Self {
        let device = world.resource::<RenderDevice>();
        let asset_server = world.resource::<AssetServer>();
        let vertex_shader = match M::vertex_shader() {
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => asset_server.load(path),
            _ => panic!("don't have a vertex shader"),
        };

        let fragment_shader = match M::fragment_shader() {
            ShaderRef::Handle(handle) => handle,
            ShaderRef::Path(path) => asset_server.load(path),
            _ => panic!("don't have a fragment shader"),
        };
        let material_layout = M::bind_group_layout(device);

        Self {
            vertex_shader: vertex_shader,
            fragment_shader: fragment_shader,
            material_layout,
            marker: PhantomData,
        }
    }
}

impl<M: Material> SpecializedRenderPipeline for TerrainRenderPipeline<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    type Key = TerrainPipelineKey<M>;
    fn specialize(
        &self,
        key: Self::Key,
    ) -> bevy::render::render_resource::RenderPipelineDescriptor {
        RenderPipelineDescriptor {
            label: None,
            layout: vec![self.material_layout.clone()],
            push_constant_ranges: default(),
            vertex: VertexState {
                shader: self.vertex_shader.clone(),
                entry_point: "vertex".into(),
                shader_defs: vec![],
                buffers: Vec::new(),
            },
            primitive: PrimitiveState {
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: PolygonMode::Fill,
                conservative: false,
                topology: PrimitiveTopology::TriangleStrip,
                strip_index_format: None,
            },
            fragment: Some(FragmentState {
                shader: self.fragment_shader.clone(),
                shader_defs: vec![],
                entry_point: "fragment".into(),
                targets: vec![Some(ColorTargetState {
                    format: TextureFormat::bevy_default(),
                    blend: Some(BlendState::REPLACE),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
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
            multisample: MultisampleState::default(),
        }
    }
}
pub struct TerrainMaterialPlugin<M: Material>(pub PhantomData<M>);

impl<M: Material> Default for TerrainMaterialPlugin<M> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<M: Material> Plugin for TerrainMaterialPlugin<M>
where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    fn build(&self, app: &mut App) {
        // Todo: don't use MaterialPlugin, but do the configuration here
        app.add_plugin(MaterialPlugin::<M>::default());

        if let Ok(render_app) = app.get_sub_app_mut(RenderApp) {
            render_app
                // .init_resource::<ExtractedMaterials<M>>()
                // .init_resource::<RenderMaterials<M>>()
                // .add_system_to_stage(RenderStage::Extract, extract_materials::<M>)
                // .add_system_to_stage(
                //     RenderStage::Prepare,
                //     prepare_materials::<M>.after(PrepareAssetLabel::PreAssetPrepare),
                // )
                .add_render_command::<Opaque3d, DrawTerrain<M>>()
                .init_resource::<TerrainRenderPipeline<M>>()
                .init_resource::<SpecializedRenderPipelines<TerrainRenderPipeline<M>>>()
                .add_system(queue_terrain::<M>.in_set(RenderSet::Queue));
        }
    }
}
/// The draw function of the terrain. It sets the pipeline and the bind groups and then issues the
/// draw call.
pub(crate) type DrawTerrain<M> = (SetItemPipeline, SetMaterialBindGroup<M, 1>, DrawMesh);
/// Queses all terrain entities for rendering via the terrain pipeline.
#[allow(clippy::too_many_arguments)]
pub(crate) fn queue_terrain<M: Material>(
    draw_functions: Res<DrawFunctions<Opaque3d>>,
    render_materials: Res<RenderMaterials<M>>,
    terrain_pipeline: Res<TerrainRenderPipeline<M>>,
    pipeline_cache: Res<PipelineCache>,
    mut pipelines: ResMut<SpecializedRenderPipelines<TerrainRenderPipeline<M>>>,
    mut view_query: Query<&mut RenderPhase<Opaque3d>>,
    terrain_query: Query<(Entity, &Handle<M>), With<TileRendered>>,
) where
    M::Data: PartialEq + Eq + Hash + Clone,
{
    let draw_function = draw_functions.read().get_id::<DrawTerrain<M>>().unwrap();

    for mut opaque_phase in view_query.iter_mut() {
        for (entity, material) in terrain_query.iter() {
            if let Some(material) = render_materials.get(material) {
                let key = TerrainPipelineKey {
                    bind_group_data: material.key.clone(),
                    texture_num: 5,
                };

                let pipeline_id = pipelines.specialize(&pipeline_cache, &terrain_pipeline, key);

                opaque_phase.add(Opaque3d {
                    entity,
                    pipeline: pipeline_id,
                    draw_function,
                    distance: f32::MIN, // draw terrain first
                });
            }
        }
    }
}
