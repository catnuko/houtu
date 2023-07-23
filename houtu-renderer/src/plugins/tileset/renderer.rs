use std::cmp::Ordering;
use std::f64::consts::PI;
use std::sync::{Arc, Mutex};

use bevy::core::FrameCount;
use bevy::ecs::system::{EntityCommands, QueryComponentError};
use bevy::prelude::*;
use bevy::render::renderer::RenderDevice;
use bevy::window::PrimaryWindow;
use houtu_jobs::{FinishedJobs, JobSpawner};
use houtu_scene::{
    Cartographic, CullingVolume, Ellipsoid, EllipsoidalOccluder, GeographicTilingScheme,
    HeightmapTerrainData, IndicesAndEdgesCache, Matrix4, Rectangle, TerrainExaggeration,
    TerrainMesh, TileBoundingRegion, TilingScheme,
};
use rand::Rng;

use crate::plugins::camera::GlobeCamera;

use super::create_terrain_mesh_job::CreateTileJob;
use super::globe_surface_tile::{
    self, computeTileVisibility, GlobeSurfaceTile, TerrainState, TileVisibility,
};
use super::imagery::{Imagery, ImageryState};
use super::imagery_layer::{
    self, ImageryLayer, ImageryLayerOtherState, TerrainDataSource, XYZDataSource,
};
use super::reproject_texture::{self, ReprojectTextureTaskQueue};
use super::terrian_material::TerrainMeshMaterial;
use super::tile_selection_result::TileSelectionResult;
use super::traversal_details::{AllTraversalQuadDetails, RootTraversalDetails, TraversalDetails};
use super::upsample_job::UpsampleJob;
use super::TileKey;

use super::quadtree_tile::{
    NodeChildren, Quadrant, QuadtreeTile, QuadtreeTileData, QuadtreeTileLoadState,
    QuadtreeTileMark, QuadtreeTileOtherState, QuadtreeTileParent, TileLoadHigh, TileLoadLow,
    TileLoadMedium, TileNode, TileToLoad, TileToRender, TileToUpdateHeight,
};
use super::tile_replacement_queue::{TileReplacementQueue, TileReplacementState};
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(imagery_layer::Plugin);
        app.register_type::<TileKey>()
            .register_type::<TileReplacementState>()
            .register_type::<Quadrant>()
            .register_type::<NodeChildren>()
            .register_type::<QuadtreeTileMark>()
            .register_type::<QuadtreeTileParent>()
            .register_type::<TileToRender>()
            .register_type::<TileToUpdateHeight>()
            .register_type::<TileLoadHigh>()
            .register_type::<TileLoadMedium>()
            .register_type::<TileLoadLow>()
            .register_type::<TileToLoad>();

        app.insert_resource(IndicesAndEdgesCacheArc::new());
    }
}
#[derive(Resource)]
pub struct IndicesAndEdgesCacheArc(pub Arc<Mutex<IndicesAndEdgesCache>>);
impl IndicesAndEdgesCacheArc {
    fn new() -> Self {
        IndicesAndEdgesCacheArc(Arc::new(Mutex::new(IndicesAndEdgesCache::new())))
    }
    fn get_cloned_cache(&self) -> Arc<Mutex<IndicesAndEdgesCache>> {
        return self.0.clone();
    }
}
#[derive(Component, PartialEq, Eq)]
pub enum RenderStage {
    Start = 0,  //默认阶段
    Load = 1,   //加载阶段
    Render = 2, //渲染阶段
}
impl Default for RenderStage {
    fn default() -> Self {
        Self::Start
    }
}
#[derive(Component, PartialEq, Eq)]
pub enum StartStageState {}
#[derive(Component, PartialEq, Eq)]
pub enum LoadStageState {
    Start = 0,
    Loading = 1,
    Loaded = 2,
}
#[derive(Component, PartialEq, Eq)]
pub enum RenderStageState {
    Start = 0,
    Rendering = 1,
    Rendered = 2,
}
fn render_stage_system(stage_query: Query<(Entity, &RenderStage)>) {
    for (entity, render_stage) in stage_query.iter() {
        match *render_stage {
            RenderStage::Start => {}

            RenderStage::Load => {}

            RenderStage::Render => {}
        }
    }
}

fn start_stage_state_system(stage_query: Query<(Entity, &StartStageState)>) {}
fn load_stage_state_system(stage_query: Query<(Entity, &LoadStageState)>) {
    for (entity, stage_state) in stage_query.iter() {
        match *stage_state {
            LoadStageState::Start => {}
            LoadStageState::Loading => {}
            LoadStageState::Loaded => {}
        }
    }
}
fn render_stage_state_system(stage_query: Query<(Entity, &RenderStageState)>) {
    for (entity, stage_state) in stage_query.iter() {
        match *stage_state {
            RenderStageState::Start => {}
            RenderStageState::Rendering => {}
            RenderStageState::Rendered => {}
        }
    }
}
