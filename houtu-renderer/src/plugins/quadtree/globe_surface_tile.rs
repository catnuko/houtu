use std::sync::{Arc, Mutex};

use bevy::math::{DVec3, DVec4};
use houtu_scene::{HeightmapTerrainData, TerrainMesh, TileBoundingRegion};

use super::{
    imagery::{ImageryState, ShareMutImagery},
    tile_imagery::TileImagery,
    tile_key::TileKey,
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
}
