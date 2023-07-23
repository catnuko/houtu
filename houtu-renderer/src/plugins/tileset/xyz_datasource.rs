use bevy::prelude::*;
use houtu_scene::{GeographicTilingScheme, Rectangle};

use super::TileKey;
#[derive(Component)]
pub struct XYZDataSource {
    pub ready: bool,
    pub rectangle: Rectangle,
    pub tiling_scheme: GeographicTilingScheme,
    pub tile_width: u32,
    pub tile_height: u32,
    pub minimum_level: u32,
    pub maximum_level: u32,
}
impl Default for XYZDataSource {
    fn default() -> Self {
        Self {
            ready: true,
            rectangle: Rectangle::MAX_VALUE.clone(),
            tiling_scheme: GeographicTilingScheme::default(),
            tile_height: 256,
            tile_width: 256,
            minimum_level: 0,
            maximum_level: 31,
        }
    }
}
impl XYZDataSource {
    pub fn requestImage(
        &self,
        key: &TileKey,
        asset_server: &Res<AssetServer>,
    ) -> Option<Handle<Image>> {
        return Some(asset_server.load(format!(
            "https://maps.omniscale.net/v2/houtu-b8084b0b/style.default/{}/{}/{}.png",
            key.level, key.x, key.y,
        )));
    }
}
