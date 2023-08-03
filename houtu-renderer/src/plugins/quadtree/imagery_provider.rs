use bevy::prelude::{AssetServer, Handle, Image};
use houtu_scene::{GeographicTilingScheme, Rectangle};

use super::credit::Credit;
use super::tile_key::TileKey;
pub trait ImageryProvider: Send + Sync {
    fn get_tile_credits(&self, key: &TileKey) -> Vec<Credit>;
    fn request_image(&self, key: &TileKey, asset_server: &AssetServer) -> Option<Handle<Image>>;
    fn pick_features(&self, key: &TileKey, longitude: f64, latitude: f64);
    fn load_image(&self, url: String);
    fn get_tile_width(&self) -> u32;
    fn get_tile_height(&self) -> u32;
    fn get_ready(&self) -> bool;
    fn get_rectangle(&self) -> &Rectangle;
    fn get_maximum_level(&self) -> u32;
    fn get_minimum_level(&self) -> u32;
    fn get_tiling_scheme(&self) -> &GeographicTilingScheme;
}
