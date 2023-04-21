use bevy::prelude::*;

#[derive(Component)]
pub struct TileKey {
    pub row: u32,
    pub column: u32,
    pub level: u32,
}
impl TileKey {
    pub fn new(row: u32, column: u32, level: u32) -> Self {
        Self { row, column, level }
    }
    // pub fn geo_coord_to_tike_key(tiling_scheme:Tilin)
}
