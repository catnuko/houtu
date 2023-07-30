use std::f64::consts::PI;

use houtu_scene::{
    Ellipsoid, GeographicTilingScheme, HeightmapTerrainData, Rectangle, TilingScheme,
};

use super::{tile_availability::TileAvailability, tile_key::TileKey};
pub trait TerrainProvider: Send + Sync {
    fn get_tiling_scheme(&self) -> &GeographicTilingScheme;
    fn get_ready(&self) -> bool;
    fn get_has_water_mask(&self) -> bool;
    fn get_has_vertex_normals(&self) -> bool;
    fn get_availability(&self) -> Option<TileAvailability>;
    // fn get_regular_grid_indices(&self, width: u32, height: u32);
    // fn get_regular_grid_indices_and_edge_indices(&self, width: u32, height: u32);
    fn request_tile_geometry(&self) -> Option<HeightmapTerrainData>;
    fn get_level_maximum_geometric_error(&self, level: u32) -> f64;
    fn load_tile_data_availability(&self, key: &TileKey) -> Option<bool>;
    fn get_tile_data_available(&self, key: &TileKey) -> Option<bool>;
}
