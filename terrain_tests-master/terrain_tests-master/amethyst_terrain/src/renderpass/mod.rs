//! Render pass.

pub use self::pass::{DrawTerrain, DrawTerrainDesc};

use serde::{Deserialize, Serialize};

use crate::{Terrain};
use amethyst::{
    assets::Processor,
    core::{
        ecs::{
            DispatcherBuilder, World
        },
    },
    error::Error,
};

use amethyst::renderer::{
    bundle::{RenderOrder, RenderPlan, RenderPlugin, Target},
    rendy::{hal::pso::ShaderStageFlags, shader::SpirvShader, graph::render::RenderGroupDesc},
    Backend, Factory
};

use load_file::load_bytes;

mod pod;
mod submodules;
mod pass;


lazy_static::lazy_static! {
    static ref TERRAIN_VERTEX: SpirvShader = SpirvShader::new(
        load_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/compiled_shader/vertex/terrain.vert.spv")).to_vec(),
        ShaderStageFlags::VERTEX,
        "main",
    );

    static ref TERRAIN_CONTROL: SpirvShader = SpirvShader::new(
        load_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/compiled_shader/tesselation/terrain.tesc.spv")).to_vec(),
        ShaderStageFlags::HULL,
        "main",
    );

    static ref TERRAIN_EVAL: SpirvShader = SpirvShader::new(
        load_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/compiled_shader/tesselation/terrain.tese.spv")).to_vec(),
        ShaderStageFlags::DOMAIN,
        "main",
    );

    static ref TERRAIN_GEOM: SpirvShader = SpirvShader::new(
        load_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/compiled_shader/geometry/terrain.geom.spv")).to_vec(),
        ShaderStageFlags::GEOMETRY,
        "main",
    );

    static ref TERRAIN_FRAGMENT: SpirvShader = SpirvShader::new(
        load_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/compiled_shader/fragment/terrain.frag.spv")).to_vec(),
        ShaderStageFlags::FRAGMENT,
        "main",
    );
}


/// Different ViewModes for rendering the Terrain.
/// Mostly used for debugging or showcasing.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TerrainViewMode {
    /// Normal color mode
    Color,
    /// Show only the wireframe of the terrain
    Wireframe,
    /// Show LOD colorcoding for the terrain
    LOD,
}
impl Default for TerrainViewMode {
    fn default() -> Self {
        TerrainViewMode::Color
    }
}

/// Colors used for the gradient skybox
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerrainConfig {
    /// The color directly above the viewer
    pub view_mode: TerrainViewMode,
}

impl Default for TerrainConfig {
    fn default() -> TerrainConfig {
        TerrainConfig {
            view_mode: Default::default(),
        }
    }
}

/// A [RenderPlugin] for drawing a Terrain using Cardinal Neighbour Quadtrees for LOD.
#[derive(Default, Debug)]
pub struct RenderTerrain {
    target: Target,
}

impl RenderTerrain {
    /// Set target to which 2d sprites will be rendered.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}


impl<B: Backend> RenderPlugin<B> for RenderTerrain {
    fn on_build<'a, 'b>(
        &mut self,
        _world: &mut World,
        builder: &mut DispatcherBuilder<'a, 'b>,
    ) -> Result<(), Error> {
        builder.add(Processor::<Terrain>::new(), "terrain_processor", &[]);
        Ok(())
    }

    fn on_plan(
        &mut self,
        plan: &mut RenderPlan<B>,
        _factory: &mut Factory<B>,
        _res: &World,
    ) -> Result<(), Error> {
        plan.extend_target(self.target, |ctx| {
            ctx.add(RenderOrder::Opaque, DrawTerrainDesc::new().builder())?;
            Ok(())
        });
        Ok(())
    }
}