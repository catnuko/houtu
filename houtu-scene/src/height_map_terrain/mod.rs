use crate::{ellipsoid::Ellipsoid, geometry::Rectangle};
mod create_mesh_job;
use bevy::math::DVec3;
pub use create_mesh_job::*;
// mod create_mesh_job;
pub struct HeightmapTerrainData {
    pub buffer: Vec<u8>,
    pub width: i64,
    pub height: i64,
}
pub struct HeightmapTerrainStructure {
    pub heightScale: f64,
    pub heightOffset: f64,
    pub elementsPerHeight: i64,
    pub stride: i64,
    pub elementMultiplier: f64,
    pub isBigEndian: bool,
    pub lowestEncodedHeight: f64,
    pub highestEncodedHeight: f64,
}
impl Default for HeightmapTerrainStructure {
    fn default() -> Self {
        HeightmapTerrainStructure {
            heightScale: 1.0,
            heightOffset: 0.0,
            elementsPerHeight: 1,
            stride: 1,
            elementMultiplier: 256.0,
            isBigEndian: false,
            lowestEncodedHeight: 0.0,
            highestEncodedHeight: 256.0,
        }
    }
}
