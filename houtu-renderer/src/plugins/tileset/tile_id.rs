use bevy::prelude::*;
#[derive(Component, Reflect, Clone, Copy, Debug, Hash)]
#[reflect(Component)]
pub struct TileId(pub Entity);

impl Default for TileId {
    fn default() -> Self {
        Self(Entity::from_raw(0))
    }
}
