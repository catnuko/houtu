use bevy::prelude::*;

#[derive(Component, Clone)]
pub struct TileKey {
    pub x: u32,
    pub y: u32,
    pub level: u32,
}
impl TileKey {
    pub fn new(x: u32, y: u32, level: u32) -> Self {
        Self { x, y, level }
    }
    // pub fn geo_coord_to_tike_key(tiling_scheme:Tilin)
}
