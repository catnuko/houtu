use geodesy::Coord;

use crate::rectangle::Rectangle;

pub trait TilingScheme {
    fn get_ellipsoid(&self) -> Ellipsoid;
    fn get_rectangle(&self) -> Rectangle;
    fn get_number_of_levels(&self) -> u32;
    fn get_number_of_x_tiles_at_level(&self, level: u32) -> u32;
    fn get_number_of_y_tiles_at_level(&self, level: u32) -> u32;
    fn get_number_of_tiles_at_level(&self, level: u32) -> u32;
    fn tile_x_y_to_native_rectange(&self, x: u32, y: u32, level: u32) -> Rectangle;
    fn tile_x_y_to_rectange(&self, x: u32, y: u32, level: u32) -> Rectangle;
    fn position_to_tile_x_y(&self, position: &Coord, level: u32) -> (u32, u32);
    fn position_to_tile_x_y_level(&self, position: &Coord, level: u32) -> (u32, u32, u32);
}
