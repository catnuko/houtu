use bevy::prelude::*;
use houtu_scene::{GeographicTilingScheme, TilingScheme, WebMercatorTilingScheme};
use std::collections::HashMap;

use super::{layer_id::LayerId, tile::Tile};
#[derive(Clone, Debug)]
pub enum TileLayerState {
    Start = 0,
}
#[derive(Clone, Debug, Resource)]
pub struct TileLayer<T: TilingScheme = GeographicTilingScheme> {
    pub tiles: HashMap<String, Entity>,
    pub tiling_scheme: T,
    pub state: TileLayerState,
}
impl<T: TilingScheme> TileLayer<T> {
    pub fn new(tiling_scheme: Option<T>) -> Self {
        Self {
            tiles: HashMap::new(),
            tiling_scheme: tiling_scheme.unwrap(),
            state: TileLayerState::Start,
        }
    }
    pub fn get_tile_entity(&self, x: u32, y: u32, level: u32) -> Option<&Entity> {
        let key = Tile::get_key(x, y, level);
        return self.tiles.get(&key);
    }
    pub fn add_tile(&mut self, x: u32, y: u32, level: u32, entity: Entity) {
        let key = Tile::get_key(x, y, level);
        self.tiles.insert(key, entity);
    }
    pub fn is_exist(&self, x: u32, y: u32, level: u32) -> bool {
        self.get_tile_entity(x, y, level).is_some()
    }
}

// #[derive(Resource)]
// pub struct TileLayers {
//     pub data: HashMap<LayerId, TileLayer>,
// }
// impl TileLayers {
//     pub fn new() -> Self {
//         Self {
//             data: HashMap::new(),
//         }
//     }
//     pub fn get_layer(&self, id: LayerId) -> &TileLayer {
//         self.data.get(&id)
//     }
// }
