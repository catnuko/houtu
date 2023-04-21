use bevy::prelude::*;

use crate::tile_key::TileKey;

#[derive(Bundle)]
pub struct Tile {
    pub key: TileKey,
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
