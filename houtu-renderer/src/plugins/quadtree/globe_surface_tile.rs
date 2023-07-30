use std::sync::{Arc, Mutex};

use bevy::{
    math::{DVec3, DVec4},
    prelude::{Handle, Image},
};
use houtu_scene::{HeightmapTerrainData, TerrainMesh, TileBoundingRegion};

use super::{
    imagery::{ImageryState, ShareMutImagery},
    imagery_layer_storage::{self, ImageryLayerStorage},
    quadtree_tile::{QuadtreeTile, QuadtreeTileLoadState},
    quadtree_tile_storage::QuadtreeTileStorage,
    terrain_provider::TerrainProvider,
    tile_imagery::TileImagery,
    tile_key::{self, TileKey},
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
    pub mesh: Option<TerrainMesh>,
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
            mesh: None,
            bounding_volume_source_tile: None,
            vertex_array: None,
            imagery: Vec::new(),
            terrain_data: None,
            waterMaskTexture: None,
        }
    }
    pub fn add(
        &mut self,
        imagery: ShareMutImagery,
        texture_coordinate_rectangle: Option<DVec4>,
        use_web_mercator_t: bool,
    ) {
        let tile_imagery =
            TileImagery::new(imagery, texture_coordinate_rectangle, use_web_mercator_t);
        self.imagery.push(tile_imagery);
    }
    pub fn eligible_for_unloading(&self) -> bool {
        let loading_is_transitioning = self.terrain_state == TerrainState::RECEIVING
            || self.terrain_state == TerrainState::TRANSFORMING;

        let mut should_removeTile = !loading_is_transitioning;

        //TODO
        let mut i = 0;
        let mut len = self.imagery.len();
        while should_removeTile && i < len {
            let tile_imagery = self.imagery.get(i).unwrap();
            should_removeTile = tile_imagery.loading_imagery.is_none()
                || tile_imagery.loading_imagery.as_ref().unwrap().lock().state
                    != ImageryState::TRANSITIONING;
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
    pub fn has_terrain_data(&self) -> bool {
        return self.terrain_data.is_some();
    }
    pub fn processImagery(
        storage: &mut QuadtreeTileStorage,
        tile_key: TileKey,
        imagery_layer_storage: &mut ImageryLayerStorage,
    ) -> bool {
        let tile = storage.get_mut(&tile_key).unwrap();

        let mut is_upsampled_only = tile.upsampled_from_parent;
        let mut is_any_tile_loaded = false;
        let mut is_done_loading = true;
        let tile_imagery_collection = &tile.data.imagery;
        for tile_imagery in tile_imagery_collection.iter() {
            if tile_imagery.loading_imagery.is_none() {
                is_upsampled_only = false;
                continue;
            }
            if tile_imagery.loading_imagery.as_ref().unwrap().get_state()
                == ImageryState::PLACEHOLDER
            {
                let imagery_layer_id = tile_imagery
                    .loading_imagery
                    .as_ref()
                    .unwrap()
                    .get_imagery_layer_id();
                let imagery_layer = imagery_layer_storage.get_mut(&imagery_layer_id).unwrap();
                if imagery_layer.ready && imagery_layer.imagery_provider.get_ready() {
                } else {
                    is_upsampled_only = false;
                }
            }
            let this_tile_done_loading = tile_imagery.process_state_machine();
            is_done_loading = is_done_loading && this_tile_done_loading;
            is_any_tile_loaded = is_any_tile_loaded
                || this_tile_done_loading
                || tile_imagery.ready_imagery.is_some();
            is_upsampled_only = is_upsampled_only
                && tile_imagery.loading_imagery.is_some()
                && (tile_imagery.loading_imagery.as_ref().unwrap().get_state()
                    == ImageryState::FAILED
                    || tile_imagery.loading_imagery.as_ref().unwrap().get_state()
                        == ImageryState::INVALID);
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
        terrainOnly: bool,
    ) {
        initialize(storage, tile_key, terrain_provider, imagery_layer_storage);
        let tile = storage.get_mut(&tile_key).unwrap();
        if tile.state == QuadtreeTileLoadState::LOADING {
            processTerrainStateMachine(storage, tile_key, terrain_provider, imagery_layer_storage);
        }
        if terrainOnly {
            return;
        }
        let tile = storage.get_mut(&tile_key).unwrap();

        let was_already_renderable = tile.renderable;
        tile.renderable = tile.data.vertex_array.is_some();
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
        let is_imagery_done_loading =
            GlobeSurfaceTile::processImagery(storage, tile_key, imagery_layer_storage);
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
            );
        }
    }
    let tile = storage.get_mut(&tile_key).unwrap();
    if tile.data.terrain_state == TerrainState::FAILED {}
    if tile.data.terrain_state == TerrainState::UNLOADED {}
    if tile.data.terrain_state == TerrainState::RECEIVED {}
    if tile.data.terrain_state == TerrainState::TRANSFORMED {}
}

fn initialize(
    storage: &mut QuadtreeTileStorage,
    tile_key: TileKey,
    terrain_provider: &Box<dyn TerrainProvider>,
    imagery_layer_storage: &mut ImageryLayerStorage,
) {
    let tile = storage.get_mut(&tile_key).unwrap();
    if (tile.state == QuadtreeTileLoadState::START) {
        prepare_new_tile(storage, tile_key, terrain_provider, imagery_layer_storage);
        let tile = storage.get_mut(&tile_key).unwrap();
        tile.state = QuadtreeTileLoadState::LOADING;
    }
}
fn prepare_new_tile(
    storage: &mut QuadtreeTileStorage,
    tile_key: TileKey,
    terrain_provider: &Box<dyn TerrainProvider>,
    imagery_layer_storage: &mut ImageryLayerStorage,
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
            imagery_layer._createTileImagerySkeletons(tile, terrain_provider);
        }
    }
}
