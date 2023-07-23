use super::{tile_selection_result::TileSelectionResult, LayerId, TileState};
use bevy::{math::DVec3, prelude::*, utils::Uuid};
use houtu_scene::{
    HeightmapTerrainData, IndicesAndEdgesCache, Rectangle, TerrainMesh, TileBoundingRegion,
    TilingScheme, WebMercatorTilingScheme,
};

#[derive(Component, Default, Clone, Debug)]
pub struct TileTextures {
    pub texture: Vec<TileImagery>,
}
#[derive(Component, Default, Clone, Debug)]
pub struct TileImagery {
    pub url: String,
    pub data: Handle<Image>,
}
#[derive(Bundle)]
pub struct TileBundle {
    pub tile: Tile,
    pub visibility: Visibility,
}
#[derive(Debug)]
pub enum TileLoadQueueType {
    High,
    Medium,
    Low,
    None,
}
impl Default for TileLoadQueueType {
    fn default() -> Self {
        Self::None
    }
}
#[derive(Component, Default, Clone, Debug)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub state: TileState,
    pub width: u32,
    pub height: u32,
    pub texture: TileTextures,
    pub layer_id: LayerId,
    pub rectangle: Rectangle,
    pub terrain_mesh: Option<TerrainMesh>,
    pub renderable: bool,
    pub tileBoundingRegion: TileBoundingRegion,
    pub occludee_point_in_scaled_space: Option<DVec3>,
    pub boundingVolumeIsFromMesh: bool,
    pub _distance: f64, //到相机的距离
    pub clippedByBoundaries: bool,
    pub needsLoading: bool,

    pub to_update_heights: bool,
}
impl Tile {
    pub fn new(x: u32, y: u32, level: u32, width: Option<u32>, height: Option<u32>) -> Self {
        return Self {
            x,
            y,
            level,
            width: width.unwrap_or(32),
            height: height.unwrap_or(32),
            state: TileState::START,
            renderable: false,
            needsLoading: true,

            ..Default::default()
        };
    }
    pub fn get_key_string(&self) -> String {
        return Tile::get_key(self.x, self.y, self.level);
    }
    pub fn get_key(x: u32, y: u32, level: u32) -> String {
        return format!("{}_{}_{}", x, y, level);
    }
}
