use houtu_scene::GeographicTilingScheme;

use super::tile_key::TileKey;

pub struct TileAvailability {
    pub tiling_scheme: GeographicTilingScheme,
    pub maximum_level: u32,
    pub root_nodes: Vec<TileKey>,
}
