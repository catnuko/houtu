use crate::{ellipsoid::Ellipsoid, geometry::Rectangle};
mod create_vertice;
mod height_map_encoding;
mod height_map_terrain_data;
mod terrain_exaggeration;
mod terrian_mesh;
use bevy::math::DVec3;
pub use create_vertice::*;
pub use height_map_encoding::*;
pub use height_map_terrain_data::*;
pub use terrain_exaggeration::*;
pub use terrian_mesh::*;
// mod create_mesh_job;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeightmapTerrainStructure {
    pub heightScale: f64,
    pub heightOffset: f64,
    pub elementsPerHeight: u32,
    pub stride: u32,
    pub elementMultiplier: u32,
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
