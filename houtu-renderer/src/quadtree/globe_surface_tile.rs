use std::sync::{Arc, Mutex};

use bevy::{
    math::{DVec3, DVec4},
    prelude::{AssetServer, Assets, Handle, Image, ResMut},
    render::renderer::RenderDevice,
};
use houtu_jobs::{FinishedJobs, JobSpawner};
use houtu_scene::{HeightmapTerrainData, TileBoundingRegion};

use crate::camera::GlobeCamera;

use super::{
    create_terrain_mesh_job::CreateTileJob,
    imagery_layer_storage::ImageryLayerStorage,
    imagery_storage::{Imagery, ImageryKey, ImageryState, ImageryStorage},
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    quadtree_primitive::QuadtreePrimitive,
    quadtree_tile::QuadtreeTileLoadState,
    quadtree_tile_storage::QuadtreeTileStorage,
    reproject_texture::ReprojectTextureTaskQueue,
    terrain_provider::TerrainProvider,
    tile_imagery::TileImagery,
    tile_key::TileKey,
    upsample_job::UpsampleJob,
};
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum TerrainState {
    FAILED = 0,
    UNLOADED = 1,
    RECEIVING = 2,
    RECEIVED = 3,
    TRANSFORMING = 4,
    TRANSFORMED = 5,
    READY = 6,
}
impl Default for TerrainState {
    fn default() -> Self {
        Self::UNLOADED
    }
}
pub struct GlobeSurfaceTile {
    pub tile_bounding_region: Option<TileBoundingRegion>,
    pub occludee_point_in_scaled_space: Option<DVec3>,
    pub terrain_state: TerrainState,
    pub bounding_volume_is_from_mesh: bool,
    pub clipped_by_boundaries: bool,
    pub bounding_volume_source_tile: Option<TileKey>,
    pub vertex_array: Option<bool>, //TODO 暂时不知道放什么数据结构，先放个bool值
    pub imagery: Vec<TileImagery>,
    pub terrain_data: Option<Arc<Mutex<HeightmapTerrainData>>>,
    pub waterMaskTexture: Option<Handle<Image>>,
}
impl GlobeSurfaceTile {
    pub fn new() -> Self {
        Self {
            tile_bounding_region: None,
            occludee_point_in_scaled_space: None,
            terrain_state: TerrainState::default(),
            clipped_by_boundaries: false,
            bounding_volume_is_from_mesh: false,
            bounding_volume_source_tile: None,
            vertex_array: None,
            imagery: Vec::new(),
            terrain_data: None,
            waterMaskTexture: None,
        }
    }
    ///新增一个TileImagery
    pub fn add_imagery(
        &mut self,
        imagery: ImageryKey,
        texture_coordinate_rectangle: Option<DVec4>,
        use_web_mercator_t: bool,
    ) {
        let tile_imagery =
            TileImagery::new(imagery, texture_coordinate_rectangle, use_web_mercator_t);
        self.imagery.push(tile_imagery);
    }
    pub fn eligible_for_unloading(&self, imagery_storage: &ImageryStorage) -> bool {
        let loading_is_transitioning = self.terrain_state == TerrainState::RECEIVING
            || self.terrain_state == TerrainState::TRANSFORMING;

        let mut should_removeTile = !loading_is_transitioning;

        //TODO
        let mut i = 0;
        let len = self.imagery.len();
        while should_removeTile && i < len {
            let tile_imagery = self.imagery.get(i).unwrap();
            should_removeTile = if tile_imagery.loading_imagery.is_none() {
                true
            } else {
                let imagery = imagery_storage
                    .get(tile_imagery.loading_imagery.as_ref().unwrap())
                    .unwrap();
                imagery.state != ImageryState::TRANSITIONING
            };

            i += 1;
        }
        return should_removeTile;
    }
    pub fn get_cloned_terrain_data(&self) -> Arc<Mutex<HeightmapTerrainData>> {
        self.terrain_data.as_ref().unwrap().clone()
    }
    pub fn has_mesh(&self) -> bool {
        if let Some(v) = self.terrain_data.as_ref() {
            v.clone().lock().unwrap().has_mesh()
        } else {
            false
        }
    }
    pub fn set_terrain_data(&mut self, new_terrain_data: HeightmapTerrainData) {
        self.terrain_data = Some(Arc::new(Mutex::new(new_terrain_data)));
        self.terrain_state = TerrainState::RECEIVED;
    }
    pub fn has_terrain_data(&self) -> bool {
        return self.terrain_data.is_some();
    }
    pub fn processImagery(
        storage: &mut QuadtreeTileStorage,
        tile_key: TileKey,
        imagery_layer_storage: &mut ImageryLayerStorage,
        skip_loading: bool,
        asset_server: &AssetServer,
        images: &mut Assets<Image>,
        render_world_queue: &mut ReprojectTextureTaskQueue,
        indices_and_edges_cache: &IndicesAndEdgesCacheArc,
        render_device: &RenderDevice,
        globe_camera: &GlobeCamera,
        imagery_storage: &mut ImageryStorage,
    ) -> bool {
        let tile = storage.get_mut(&tile_key).unwrap();

        let mut is_upsampled_only = tile.upsampled_from_parent;
        let mut is_any_tile_loaded = false;
        let mut is_done_loading = true;
        for i in 0..tile.data.imagery.len() {
            let tile_imagery = tile.data.imagery.get_mut(i).unwrap();
            if tile_imagery.loading_imagery.is_none() {
                is_upsampled_only = false;
                continue;
            }
            let loading_imagery = imagery_storage
                .get(tile_imagery.loading_imagery.as_ref().unwrap())
                .expect(
                    format!(
                        "tile_imagery.loading_imagery is not existed {:?}",
                        tile_imagery.loading_imagery
                    )
                    .as_str(),
                );
            if loading_imagery.state == ImageryState::PLACEHOLDER {
                // TODO: 进不到这里，可删除
                bevy::log::info!("placeholder");
                let imagery_layer_id = loading_imagery.key.layer_id;
                let imagery_layer = imagery_layer_storage.get(&imagery_layer_id).unwrap();
                if imagery_layer.ready && imagery_layer.imagery_provider.get_ready() {
                } else {
                    is_upsampled_only = false;
                }
            }
            let imagery_layer_id = loading_imagery.key.layer_id;
            let imagery_layer = imagery_layer_storage.get_mut(&imagery_layer_id).unwrap();
            let loading_imagery_key = tile_imagery.loading_imagery.as_ref().unwrap().clone();
            let this_tile_done_loading = TileImagery::process_state_machine(
                tile,
                i,
                skip_loading,
                imagery_layer,
                asset_server,
                images,
                render_world_queue,
                indices_and_edges_cache,
                render_device,
                globe_camera,
                imagery_storage,
            );
            let loading_imagery = imagery_storage.get(&loading_imagery_key).unwrap();
            let tile_imagery = tile.data.imagery.get_mut(i).unwrap();
            is_done_loading = is_done_loading && this_tile_done_loading;
            is_any_tile_loaded = is_any_tile_loaded
                || this_tile_done_loading
                || tile_imagery.ready_imagery.is_some();
            is_upsampled_only = is_upsampled_only
                && tile_imagery.loading_imagery.is_some()
                && (loading_imagery.state == ImageryState::FAILED
                    || loading_imagery.state == ImageryState::INVALID);
        }
        tile.upsampled_from_parent = is_upsampled_only;
        tile.renderable = tile.renderable && (is_any_tile_loaded || is_done_loading);
        return is_done_loading;
    }
    pub fn process_state_machine(
        storage: &mut QuadtreeTileStorage,
        tile_key: TileKey,
        terrain_provider: &Box<dyn TerrainProvider>,
        imagery_layer_storage: &mut ImageryLayerStorage,
        terrain_only: bool,
        job_spawner: &mut JobSpawner,
        indices_and_edges_cache: &IndicesAndEdgesCacheArc,
        asset_server: &AssetServer,
        images: &mut Assets<Image>,
        render_world_queue: &mut ReprojectTextureTaskQueue,
        render_device: &RenderDevice,
        globe_camera: &GlobeCamera,
        imagery_storage: &mut ImageryStorage,
    ) {
        initialize(
            storage,
            tile_key,
            terrain_provider,
            imagery_layer_storage,
            imagery_storage,
        );
        let tile = storage.get_mut(&tile_key).unwrap();
        if tile.state == QuadtreeTileLoadState::LOADING {
            processTerrainStateMachine(
                storage,
                tile_key,
                terrain_provider,
                imagery_layer_storage,
                job_spawner,
                indices_and_edges_cache,
                asset_server,
                images,
                render_world_queue,
                render_device,
                globe_camera,
                imagery_storage,
            );
        }
        if terrain_only {
            return;
        }
        let tile = storage.get_mut(&tile_key).unwrap();

        let was_already_renderable = tile.renderable;
        tile.renderable = tile.data.has_mesh();
        let is_terrain_done_loading = tile.data.terrain_state == TerrainState::READY;
        tile.upsampled_from_parent = tile.data.terrain_data.is_some()
            && tile
                .data
                .terrain_data
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .was_created_by_upsampling();
        let is_imagery_done_loading = GlobeSurfaceTile::processImagery(
            storage,
            tile_key,
            imagery_layer_storage,
            false,
            asset_server,
            images,
            render_world_queue,
            indices_and_edges_cache,
            render_device,
            globe_camera,
            imagery_storage,
        );
        let tile = storage.get_mut(&tile_key).unwrap();
        if is_terrain_done_loading && is_imagery_done_loading {
            tile.state = QuadtreeTileLoadState::DONE;
        }
        if was_already_renderable {
            tile.renderable = true
        }
    }
}
fn processTerrainStateMachine(
    storage: &mut QuadtreeTileStorage,
    tile_key: TileKey,
    terrain_provider: &Box<dyn TerrainProvider>,
    imagery_layer_storage: &mut ImageryLayerStorage,
    job_spawner: &mut JobSpawner,
    indices_and_edges_cache: &IndicesAndEdgesCacheArc,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    render_world_queue: &mut ReprojectTextureTaskQueue,
    render_device: &RenderDevice,
    globe_camera: &GlobeCamera,
    imagery_storage: &mut ImageryStorage,
) {
    let tile = storage.get_mut(&tile_key).unwrap();
    if tile.data.terrain_state == TerrainState::FAILED && tile.parent.is_some() {
        let parent_key = tile.parent.unwrap().clone();
        let parent = storage.get(&parent_key).unwrap();
        let parent_ready = parent.data.terrain_data.is_some()
            && parent
                .data
                .terrain_data
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .can_upsample();
        if parent_ready {
            GlobeSurfaceTile::process_state_machine(
                storage,
                tile_key,
                terrain_provider,
                imagery_layer_storage,
                true,
                job_spawner,
                indices_and_edges_cache,
                asset_server,
                images,
                render_world_queue,
                render_device,
                globe_camera,
                imagery_storage,
            );
        }
    }
    let tile = storage.get_mut(&tile_key).unwrap();
    if tile.data.terrain_state == TerrainState::FAILED {
        if tile.parent.is_some() {
            let parent_key = tile.parent.unwrap().clone();
            let parent = storage.get(&parent_key).unwrap();
            if parent.data.terrain_data.is_some() {
                job_spawner.spawn(UpsampleJob {
                    terrain_data: parent.data.get_cloned_terrain_data(),
                    tiling_scheme: terrain_provider.get_tiling_scheme().clone(),
                    parent_key: parent_key,
                    key: tile_key,
                })
            }
        } else {
            tile.state = QuadtreeTileLoadState::FAILED;
        }
    }
    let tile = storage.get_mut(&tile_key).unwrap();
    if tile.data.terrain_state == TerrainState::UNLOADED {
        tile.data.terrain_state = TerrainState::RECEIVING;
        let value = terrain_provider
            .request_tile_geometry()
            .expect("terrain_datasource.request_tile_geometry");
        tile.data.set_terrain_data(value);
    }
    let tile = storage.get_mut(&tile_key).unwrap();
    if tile.data.terrain_state == TerrainState::RECEIVING {}
    let tile = storage.get_mut(&tile_key).unwrap();
    if tile.data.terrain_state == TerrainState::RECEIVED {
        job_spawner.spawn(CreateTileJob {
            terrain_data: tile.data.get_cloned_terrain_data(),
            key: tile_key,
            tiling_scheme: terrain_provider.get_tiling_scheme().clone(),
            indices_and_edges_cache: indices_and_edges_cache.get_cloned_cache(),
        });
        tile.data.terrain_state = TerrainState::TRANSFORMING;
        // bevy::log::info!("{:?} is creating mesh", tile.key);
    }
    let tile = storage.get_mut(&tile_key).unwrap();
    if tile.data.terrain_state == TerrainState::TRANSFORMING {}
    if tile.data.terrain_state == TerrainState::TRANSFORMED {
        tile.data.terrain_state = TerrainState::READY;
        // bevy::log::info!("terrain state of tile {:?} is ready", tile.key);
    }
    if tile.data.terrain_state == TerrainState::READY {}
}
pub fn process_terrain_state_machine_system(
    mut finished_jobs: FinishedJobs,
    mut primitive: ResMut<QuadtreePrimitive>,
) {
    while let Some(result) = finished_jobs.take_next::<CreateTileJob>() {
        if let Ok(res) = result {
            let tile = primitive.storage.get_mut(&res.key).unwrap();
            tile.data.terrain_state = TerrainState::TRANSFORMED;
        }
    }
    while let Some(result) = finished_jobs.take_next::<UpsampleJob>() {
        if let Ok(res) = result {
            let tile = primitive.storage.get_mut(&res.key).unwrap();

            if let Some(new_terrain_data) = res.terrain_data {
                tile.data.set_terrain_data(new_terrain_data);
            } else {
                tile.data.terrain_state = TerrainState::FAILED;
            }
        }
    }
}
fn initialize(
    storage: &mut QuadtreeTileStorage,
    tile_key: TileKey,
    terrain_provider: &Box<dyn TerrainProvider>,
    imagery_layer_storage: &mut ImageryLayerStorage,
    imagery_storage: &mut ImageryStorage,
) {
    let tile = storage.get_mut(&tile_key).unwrap();
    if tile.state == QuadtreeTileLoadState::START {
        prepare_new_tile(
            storage,
            tile_key,
            terrain_provider,
            imagery_layer_storage,
            imagery_storage,
        );
        let tile = storage.get_mut(&tile_key).unwrap();
        tile.state = QuadtreeTileLoadState::LOADING;
    }
}
fn prepare_new_tile(
    storage: &mut QuadtreeTileStorage,
    tile_key: TileKey,
    terrain_provider: &Box<dyn TerrainProvider>,
    imagery_layer_storage: &mut ImageryLayerStorage,
    imagery_storage: &mut ImageryStorage,
) {
    let tile = storage.get_mut(&tile_key).unwrap();
    let mut available = terrain_provider.get_tile_data_available(&tile_key);
    if available.is_none() && tile.parent.is_some() {
        let parent_key = tile.parent.unwrap().clone();
        let parent = storage.get(&parent_key).unwrap();
        if parent.data.terrain_data.is_some() {
            available = Some(
                parent
                    .data
                    .terrain_data
                    .as_ref()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .is_child_available(parent_key.x, parent_key.y, tile_key.x, tile_key.y),
            );
        }
    }
    let tile = storage.get_mut(&tile_key).unwrap();
    if available == Some(false) {
        tile.data.terrain_state = TerrainState::FAILED;
    }
    for (_, imagery_layer) in imagery_layer_storage.map.iter_mut() {
        if imagery_layer.show {
            imagery_layer._create_tile_imagery_skeletons(tile, terrain_provider, imagery_storage);
        }
    }
}
