use bevy::{
    math::UVec3,
    prelude::{Bundle, Color, Component, FromReflect, Reflect, ReflectComponent, Visibility},
};
use houtu_scene::TerrainMesh;

use super::{layer::TileLayerId, storage::TileStorage};
#[derive(
    Component, Reflect, FromReflect, Default, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd,
)]
#[reflect(Component)]
pub struct TileKey {
    x: u32,
    y: u32,
    level: u32,
}
impl TileKey {
    pub fn new(x: u32, y: u32, level: u32) -> Self {
        Self { x, y, level }
    }

    pub fn get_id(&self) -> String {
        format!("{}_{}_{}", self.x, self.y, self.level)
    }
}

impl From<TileKey> for UVec3 {
    fn from(pos: TileKey) -> Self {
        UVec3::new(pos.x, pos.y, pos.level)
    }
}

impl From<&TileKey> for UVec3 {
    fn from(pos: &TileKey) -> Self {
        UVec3::new(pos.x, pos.y, pos.level)
    }
}

impl From<UVec3> for TileKey {
    fn from(v: UVec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            level: v.z,
        }
    }
}
/// Hides or shows a tile based on the boolean. Default: True
#[derive(Component, Reflect, Clone, Copy, Debug, Hash)]
#[reflect(Component)]
pub struct TileVisible(pub bool);

impl Default for TileVisible {
    fn default() -> Self {
        Self(true)
    }
}
/// A texture index into the atlas or texture array for a single tile. Indices in an atlas are horizontal based.
#[derive(Component, Reflect, Default, Clone, Copy, Debug, Hash)]
#[reflect(Component)]
pub struct TileTextureIndex(pub u32);
#[derive(Component, Default, Clone, Debug)]
pub struct TerrainMeshWrap(pub Option<TerrainMesh>);

#[derive(Bundle, Clone, Debug)]
pub struct TileBundle {
    pub key: TileKey,
    pub visible: Visibility,
    pub tile_layer_id: TileLayerId,
    pub terrain_mesh: TerrainMeshWrap,
}
pub fn tile_system() {}
