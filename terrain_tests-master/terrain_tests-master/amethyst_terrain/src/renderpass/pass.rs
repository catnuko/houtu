//! Terrain pass
//!
#[allow(unused_imports)]
use amethyst::{
    assets::{AssetStorage, Handle},
    core::{
        ecs::{shred::ResourceId, Entity, Join, Read, ReadExpect, ReadStorage, SystemData, World},
        math as na,
        math::base::coordinates::XYZW,
        math::Vector3,
        math::Vector4,
        Transform,
    },
    error::Error,
    renderer::{
        batch::{GroupIterator, OneLevelBatch, TwoLevelBatch},
        camera::{ActiveCamera, Camera},
        light::Light,
        mtl::{Material, MaterialDefaults},
        palette,
        pipeline::{PipelineDescBuilder, PipelinesBuilder},
        rendy::{
            command::{QueueId, RenderPassEncoder},
            factory::Factory,
            graph::{
                render::{PrepareResult, RenderGroup, RenderGroupDesc},
                GraphContext, NodeBuffer, NodeImage,
            },
            hal::{self, device::Device, Primitive},
            mesh::{AsVertex, MeshBuilder, Normal, Position, TexCoord, VertexFormat},
            shader::Shader,
        },
        resources::AmbientColor,
        submodules::{gather::CameraGatherer, DynamicVertexBuffer, EnvironmentSub},
        types::{Backend, Mesh, Texture},
        util,
        visibility::Visibility,
    },
    window::ScreenDimensions,
};

use derivative::Derivative;
use std::marker::PhantomData;

use crate::{
    component::{Terrain, TexAlbedo, TexHeightmap, TexNormal},
    renderpass::{
        pod,
        submodules::{TerrainId, TerrainSub},
    },
    TerrainConfig, TerrainViewMode,
};
use cnquadtree::TileTree;

macro_rules! profile_scope_impl {
    ($string:expr) => {
        #[cfg(feature = "profiler")]
        let _profile_scope = thread_profiler::ProfileScope::new(format!(
            "{} {}: {}",
            module_path!(),
            <T as Base3DPassDef<B>>::NAME,
            $string
        ));
    };
}

/// Draw mesh without lighting
#[derive(Clone, Derivative)]
#[derivative(Debug(bound = ""), Default(bound = ""))]
pub struct DrawTerrainDesc<B: Backend> {
    marker: PhantomData<B>,
}

impl<B: Backend> DrawTerrainDesc<B> {
    /// Create instance of `DrawTerrainDesc`
    pub fn new() -> Self {
        Default::default()
    }
}

impl<B: Backend> RenderGroupDesc<B, World> for DrawTerrainDesc<B> {
    fn build<'a>(
        self,
        _ctx: &GraphContext<B>,
        factory: &mut Factory<B>,
        queue: QueueId,
        _aux: &World,
        framebuffer_width: u32,
        framebuffer_height: u32,
        subpass: hal::pass::Subpass<'_, B>,
        _buffers: Vec<NodeBuffer>,
        _images: Vec<NodeImage>,
    ) -> Result<Box<dyn RenderGroup<B, World>>, failure::Error> {
        let env = EnvironmentSub::new(
            factory,
            [
                hal::pso::ShaderStageFlags::GRAPHICS - hal::pso::ShaderStageFlags::FRAGMENT,
                hal::pso::ShaderStageFlags::FRAGMENT,
            ],
        )?;
        let terrains = TerrainSub::new(factory)?;

        let mut vertex_format = vec![Position::vertex(), Normal::vertex(), TexCoord::vertex()];
        let (mut pipelines, pipeline_layout) = build_terrain_pipeline(
            factory,
            subpass,
            framebuffer_width,
            framebuffer_height,
            &vertex_format,
            vec![env.raw_layout(), terrains.raw_layout()],
        )?;

        vertex_format.sort();

        // Todo: Find Backport
        // Generate the different mesh variations here to avoid tesselation?
        let pos = vec![
            Position([-1.0, 0.0, -1.0].into()),
            Position([1.0, 0.0, -1.0].into()),
            Position([1.0, 0.0, 1.0].into()),
            Position([-1.0, 0.0, 1.0].into()),
        ];
        let norms = vec![
            Normal([0.0, 1.0, 0.0].into()),
            Normal([0.0, 1.0, 0.0].into()),
            Normal([0.0, 1.0, 0.0].into()),
            Normal([0.0, 1.0, 0.0].into()),
        ];
        let texs = vec![
            TexCoord([0.0, 1.0].into()),
            TexCoord([1.0, 1.0].into()),
            TexCoord([1.0, 0.0].into()),
            TexCoord([0.0, 0.0].into()),
        ];

        let basic_mesh = MeshBuilder::new()
            .with_vertices(pos)
            .with_vertices(norms)
            .with_vertices(texs)
            .with_prim_type(Primitive::PatchList(4))
            .build(queue, factory)?;

        Ok(Box::new(DrawTerrain::<B> {
            pipeline: pipelines.remove(0),
            pipeline_layout,
            vertex_format,
            env,
            terrains,
            terrain_patches: Default::default(),
            patches: DynamicVertexBuffer::new(),
            basic_mesh,
        }))
    }
}

pub type TerrainTextureSet = (TexHeightmap, TexNormal, TexAlbedo);

/// Draw a terrain
#[derive(Derivative)]
#[derivative(Debug(bound = ""))]
pub struct DrawTerrain<B: Backend> {
    pipeline: B::GraphicsPipeline,
    pipeline_layout: B::PipelineLayout,
    vertex_format: Vec<VertexFormat>,
    env: EnvironmentSub<B>,
    terrains: TerrainSub<B, TerrainTextureSet>,
    terrain_patches: OneLevelBatch<TerrainId, pod::InstancedPatchArgs>,
    patches: DynamicVertexBuffer<B, pod::InstancedPatchArgs>,
    basic_mesh: amethyst::renderer::rendy::mesh::Mesh<B>,
}

#[derive(SystemData)]
struct TerrainPassData<'a> {
    transforms: ReadStorage<'a, Transform>,
    terrains: ReadStorage<'a, Handle<Terrain>>,
    terrain_config: ReadExpect<'a, TerrainConfig>,
    terrain_storage: Read<'a, AssetStorage<Terrain>>,
}

impl<B: Backend> RenderGroup<B, World> for DrawTerrain<B> {
    fn prepare(
        &mut self,
        factory: &Factory<B>,
        _queue: QueueId,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        resources: &World,
    ) -> PrepareResult {
        log::trace!("prepare draw");
        let TerrainPassData {
            transforms,
            terrains,
            terrain_config: _,
            terrain_storage,
            ..
        } = TerrainPassData::fetch(resources);

        self.env.process(factory, index, resources);
        self.terrains.maintain();

        self.terrain_patches.clear_inner();

        let terrains_ref = &mut self.terrains;
        let terrain_patches_ref = &mut self.terrain_patches;

        profile_scope_impl!("gather_terrains");

        let CameraGatherer {
            camera_position,
            projview: _,
        } = CameraGatherer::gather(resources);
        let camer_pos_ref: [f32; 3] = *camera_position.as_ref();
        let camera_pos_2d = [camer_pos_ref[0], camer_pos_ref[2]];

        for (terrain_handle, _tform) in (&terrains, &transforms).join() {
            if let Some((terrain_id, _)) = terrains_ref.insert(factory, resources, terrain_handle) {
                let terrain = terrain_storage
                    .get(terrain_handle)
                    .expect("Invalid Terrain handle");
                // Todo: take the terrain transform `tform` into account.
                // Todo: remove terrain specific offset and use the transform for this.
                let quadtree = TileTree::new(
                    camera_pos_2d,
                    [0.0, 0.0, (terrain.size) as f32, (terrain.size) as f32].into(),
                    terrain.max_level,
                );
                let leaves = quadtree.leaves();
                let mut patches = Vec::with_capacity(leaves.len());
                for patch in leaves {
                    patches.push(pod::InstancedPatchArgs::from_object_data(patch, &quadtree));
                }
                terrain_patches_ref.insert(terrain_id, patches.drain(..));
            }
        }

        self.terrain_patches.prune();

        self.patches.write(
            factory,
            index,
            self.terrain_patches.count() as u64,
            self.terrain_patches.data(),
        );
        // Todo: Reenable this
        //     match terrain_config.view_mode {
        //         TerrainViewMode::Wireframe => {
        //             effect.update_global("toggle_wireframe", 1.0);
        //         }
        //         TerrainViewMode::Color => {
        //             effect.update_global("toggle_wireframe", 0.0);
        //             // Same as Above but with a color texture.
        //         }
        //         TerrainViewMode::LOD => {

        //         }
        //     }
        PrepareResult::DrawRecord
    }

    fn draw_inline(
        &mut self,
        mut encoder: RenderPassEncoder<'_, B>,
        index: usize,
        _subpass: hal::pass::Subpass<'_, B>,
        _resources: &World,
    ) {
        profile_scope_impl!("draw terrain");

        encoder.bind_graphics_pipeline(&self.pipeline);

        self.env.bind(index, &self.pipeline_layout, 0, &mut encoder);

        self.basic_mesh
            .bind(0, &self.vertex_format, &mut encoder)
            .unwrap();
        if self
            .patches
            .bind(index, self.vertex_format.len() as u32, 0, &mut encoder)
        {
            let mut instances_drawn = 0;
            // Iterate over multiple terrains
            for (&terrain_id, batch_data) in self.terrain_patches.iter() {
                if self.terrains.loaded(terrain_id) {
                    self.terrains
                        .bind(&self.pipeline_layout, 1, terrain_id, &mut encoder);
                    unsafe {
                        encoder.draw(
                            0..self.basic_mesh.len(),
                            instances_drawn..instances_drawn + batch_data.len() as u32,
                        );
                    }
                    instances_drawn += batch_data.len() as u32;
                }
            }
        }
    }

    fn dispose(self: Box<Self>, factory: &mut Factory<B>, _aux: &World) {
        profile_scope_impl!("dispose");
        unsafe {
            factory.device().destroy_graphics_pipeline(self.pipeline);
            factory
                .device()
                .destroy_pipeline_layout(self.pipeline_layout);
        }
    }
}

fn build_terrain_pipeline<B: Backend>(
    factory: &Factory<B>,
    subpass: hal::pass::Subpass<'_, B>,
    framebuffer_width: u32,
    framebuffer_height: u32,
    vertex_format: &[VertexFormat],
    layouts: Vec<&B::DescriptorSetLayout>,
) -> Result<(Vec<B::GraphicsPipeline>, B::PipelineLayout), failure::Error> {
    let pipeline_layout = unsafe {
        factory
            .device()
            .create_pipeline_layout(layouts, None as Option<(_, _)>)
    }?;

    let vertex_desc = vertex_format
        .iter()
        .map(|f| (f.clone(), hal::pso::VertexInputRate::Vertex))
        .chain(Some((
            pod::InstancedPatchArgs::vertex(),
            hal::pso::VertexInputRate::Instance(1),
        )))
        .collect::<Vec<_>>();

    let shader_vertex = unsafe { super::TERRAIN_VERTEX.module(factory).unwrap() };
    let shader_tsc = unsafe { super::TERRAIN_CONTROL.module(factory).unwrap() };
    let shader_tse = unsafe { super::TERRAIN_EVAL.module(factory).unwrap() };
    let shader_geom = unsafe { super::TERRAIN_GEOM.module(factory).unwrap() };
    let shader_fragment = unsafe { super::TERRAIN_FRAGMENT.module(factory).unwrap() };

    let pipe_desc = PipelineDescBuilder::new()
        .with_input_assembler(hal::pso::InputAssemblerDesc::new(
            hal::Primitive::PatchList(4),
        ))
        .with_vertex_desc(&vertex_desc)
        .with_shaders(util::simple_shader_set_ext(
            &shader_vertex,
            Some(&shader_fragment),
            Some(&shader_tsc),
            Some(&shader_tse),
            // Some(&shader_geom)
            None,
        ))
        .with_layout(&pipeline_layout)
        .with_subpass(subpass)
        .with_framebuffer_size(framebuffer_width, framebuffer_height)
        .with_face_culling(hal::pso::Face::BACK)
        .with_depth_test(hal::pso::DepthTest::On {
            fun: hal::pso::Comparison::Less,
            write: true,
        })
        .with_blend_targets(vec![hal::pso::ColorBlendDesc(
            hal::pso::ColorMask::ALL,
            hal::pso::BlendState::Off,
        )]);

    let pipelines = PipelinesBuilder::new()
        .with_pipeline(pipe_desc)
        .build(factory, None);

    unsafe {
        factory.destroy_shader_module(shader_vertex);
        factory.destroy_shader_module(shader_tsc);
        factory.destroy_shader_module(shader_tse);
        // factory.destroy_shader_module(shader_geom);
        factory.destroy_shader_module(shader_fragment);
    }

    match pipelines {
        Err(e) => {
            unsafe {
                factory.device().destroy_pipeline_layout(pipeline_layout);
            }
            Err(e)
        }
        Ok(pipelines) => Ok((pipelines, pipeline_layout)),
    }
}
