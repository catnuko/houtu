use std::f64::consts::PI;

use houtu_scene::{
    Ellipsoid, GeographicTilingScheme, HeightmapTerrainData, Rectangle, TilingScheme,
};

use super::{terrain_provider::TerrainProvider, tile_key::TileKey};

pub struct EllipsoidTerrainProvider {
    pub tiling_scheme: GeographicTilingScheme,
    _level_zero_maximum_geometric_error: f64,
    pub ready: bool,
    pub rectangle: Rectangle,
}
impl EllipsoidTerrainProvider {
    pub fn new() -> Self {
        let tiling_scheme = GeographicTilingScheme::default();
        let _level_zero_maximum_geometric_error =
            get_level_zero_maximum_geometric_error(&tiling_scheme);

        Self {
            tiling_scheme: tiling_scheme,
            _level_zero_maximum_geometric_error: _level_zero_maximum_geometric_error,
            ready: true,
            rectangle: Rectangle::MAX_VALUE.clone(),
        }
    }
}
impl TerrainProvider for EllipsoidTerrainProvider {
    fn get_tiling_scheme(&self) -> &GeographicTilingScheme {
        return &self.tiling_scheme;
    }
    fn get_tile_data_available(&self, _key: &TileKey) -> Option<bool> {
        return None;
    }
    fn load_tile_data_availability(&self, _key: &TileKey) -> Option<bool> {
        return None;
    }
    fn get_level_maximum_geometric_error(&self, level: u32) -> f64 {
        return self._level_zero_maximum_geometric_error / (1 << level) as f64;
    }
    fn request_tile_geometry(&self) -> Option<HeightmapTerrainData> {
        let width = 16;
        let height = 16;
        return Some(HeightmapTerrainData::new(
            vec![0.; width * height],
            width as u32,
            height as u32,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));
    }
    fn get_has_water_mask(&self) -> bool {
        false
    }
    fn get_has_vertex_normals(&self) -> bool {
        false
    }
    fn get_availability(&self) -> Option<super::tile_availability::TileAvailability> {
        return None;
    }
    fn get_ready(&self) -> bool {
        self.ready
    }
}
fn get_level_zero_maximum_geometric_error(tiling_scheme: &GeographicTilingScheme) -> f64 {
    return get_estimated_level_zero_geometric_error_for_a_heightmap(
        &tiling_scheme.ellipsoid,
        64,
        tiling_scheme.get_number_of_tiles_at_level(0),
    );
}
fn get_estimated_level_zero_geometric_error_for_a_heightmap(
    ellipsoid: &Ellipsoid,
    tile_image_width: u32,
    number_of_tiles_at_level_zero: u32,
) -> f64 {
    return (ellipsoid.maximum_radius * 2. * PI * 0.25)
        / (tile_image_width as f64 * number_of_tiles_at_level_zero as f64);
}
