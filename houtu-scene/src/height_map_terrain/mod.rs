use crate::{ellipsoid::Ellipsoid, geometry::Rectangle};
mod create_vertice;
use bevy::math::DVec3;
pub use create_vertice::*;
// mod create_mesh_job;
pub struct HeightmapTerrainData {
    pub buffer: Vec<u8>,
    pub width: i32,
    pub height: i32,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeightmapTerrainStructure {
    pub heightScale: f64,
    pub heightOffset: f64,
    pub elementsPerHeight: i32,
    pub stride: i32,
    pub elementMultiplier: i32,
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
            elementMultiplier: 256,
            isBigEndian: false,
            lowestEncodedHeight: 0.0,
            highestEncodedHeight: 256.0,
        }
    }
}
