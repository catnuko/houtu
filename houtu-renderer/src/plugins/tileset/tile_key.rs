use bevy::{math::UVec3, prelude::*};

use crate::plugins::cnquadtree::NodeId;

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

impl From<NodeId> for TileKey {
    fn from(value: NodeId) -> Self {
        Self {
            x: value.x,
            y: value.y,
            level: value.level,
        }
    }
}
