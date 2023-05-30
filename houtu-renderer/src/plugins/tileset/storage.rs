use bevy::{prelude::*, utils::HashMap};

use super::tile::TileKey;

/// Used to store tile entities for fast look up.
/// Tile entities are stored in a grid. The grid is always filled with None.
#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect(Component)]
pub struct TileStorage {
    pub tiles: Vec<Option<Entity>>,
    id_to_index: HashMap<String, usize>,
}

impl TileStorage {
    /// Creates a new tile storage that is empty.
    pub fn empty() -> Self {
        Self {
            tiles: vec![None],
            id_to_index: HashMap::new(),
        }
    }
    pub fn get(&self, tile_key: &TileKey) -> Option<Entity> {
        if let Some(index) = self.id_to_index.get(&tile_key.get_id()) {
            return self.tiles[*index];
        } else {
            return None;
        }
    }
    pub fn set(&mut self, tile_key: &TileKey, tile_entity: Entity) {
        let index = self.tiles.len();
        self.tiles.push(Some(tile_entity));
        let id = tile_key.get_id();
        self.id_to_index.insert(id, index);
    }
    pub fn iter(&self) -> impl Iterator<Item = &Option<Entity>> {
        self.tiles.iter()
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Option<Entity>> {
        self.tiles.iter_mut()
    }
    pub fn remove(&mut self, tile_key: &TileKey) {
        if let Some(index) = self.id_to_index.get(&tile_key.get_id()) {
            self.tiles[*index].take();
        }
    }
}
