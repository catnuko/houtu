use bevy::prelude::*;
#[derive(Component, Reflect, Clone, Copy, Debug, Hash)]
#[reflect(Component)]
pub struct TileLayerId(pub Entity);

impl Default for TileLayerId {
    fn default() -> Self {
        Self(Entity::from_raw(0))
    }
}
