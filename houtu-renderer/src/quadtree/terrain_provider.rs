use bevy::prelude::{AssetServer, Assets, Handle};
use houtu_scene::{GeographicTilingScheme, HeightmapTerrainData};

use super::{
    terrain_data_asset::TerrainDataAsset, tile_availability::TileAvailability, tile_key::TileKey,
};
pub trait TerrainProvider: Send + Sync {
    fn get_tiling_scheme(&self) -> &GeographicTilingScheme;
    fn get_ready(&self) -> bool;
    fn get_has_water_mask(&self) -> bool;
    fn get_has_vertex_normals(&self) -> bool;
    fn get_availability(&mut self) -> Option<&mut TileAvailability>;
    fn request_tile_geometry(
        &self,
        key: &TileKey,
        asset_server: &AssetServer,
        assets: &mut Assets<TerrainDataAsset>,
    ) -> Option<Handle<TerrainDataAsset>>;
    fn get_level_maximum_geometric_error(&self, level: u32) -> f64;
    fn load_tile_data_availability(&self, key: &TileKey) -> Option<bool>;
    fn get_tile_data_available(&self, key: &TileKey) -> Option<bool>;
}
