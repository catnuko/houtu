use crate::{ellipsoid::Ellipsoid, geometry::Rectangle};
mod create_vertice;
mod height_map_encoding;
mod height_map_terrain_data;
mod terrain_exaggeration;
mod terrian_mesh;

pub use create_vertice::*;
pub use height_map_encoding::*;
pub use height_map_terrain_data::*;
pub use terrain_exaggeration::*;
pub use terrian_mesh::*;
// mod create_mesh_job;
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeightmapTerrainStructure {
    pub height_scale: f64,
    pub height_offset: f64,
    pub elements_per_height: u32,
    pub stride: u32,
    pub element_multiplier: u32,
    pub is_big_endian: bool,
    pub lowestEncodedHeight: f64,
    pub highestEncodedHeight: f64,
}
impl Default for HeightmapTerrainStructure {
    fn default() -> Self {
        HeightmapTerrainStructure {
            height_scale: 1.0,
            height_offset: 0.0,
            elements_per_height: 1,
            stride: 1,
            element_multiplier: 256,
            is_big_endian: false,
            lowestEncodedHeight: 0.0,
            highestEncodedHeight: 256.0,
        }
    }
}
