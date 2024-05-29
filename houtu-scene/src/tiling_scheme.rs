use std::f64::consts::PI;

use bevy::prelude::UVec2;

use crate::{Cartographic, Ellipsoid, Rectangle};

pub trait TilingScheme: Send + Sync {
    fn get_ellipsoid(&self) -> Ellipsoid;
    fn get_rectangle(&self) -> Rectangle;
    fn get_number_of_x_tiles_at_level(&self, level: u32) -> u32;
    fn get_number_of_y_tiles_at_level(&self, level: u32) -> u32;
    fn get_number_of_tiles_at_level(&self, level: u32) -> u32;
    fn tile_x_y_to_native_rectange(&self, x: u32, y: u32, level: u32) -> Rectangle;
    fn tile_x_y_to_rectange(&self, x: u32, y: u32, level: u32) -> Rectangle;
    fn position_to_tile_x_y(&self, position: &Cartographic, level: u32) -> Option<UVec2>;
    fn rectangle_to_native_rectangle(&self, rectangle: &Rectangle) -> Rectangle;
    fn get_name(&self) -> &'static str;
}
pub fn get_level_zero_maximum_geometric_error<T: TilingScheme>(tiling_scheme: &T) -> f64 {
    return get_estimated_level_zero_geometric_error_for_a_heightmap(
        &tiling_scheme.get_ellipsoid(),
        64,
        tiling_scheme.get_number_of_tiles_at_level(0),
    );
}
pub fn get_estimated_level_zero_geometric_error_for_a_heightmap(
    ellipsoid: &Ellipsoid,
    tile_image_width: u32,
    number_of_tiles_at_level_zero: u32,
) -> f64 {
    return (ellipsoid.maximum_radius * 2. * PI * 0.25)
        / (tile_image_width as f64 * number_of_tiles_at_level_zero as f64);
}
