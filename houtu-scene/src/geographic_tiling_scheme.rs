use crate::{bit_or_zero, Cartographic, Ellipsoid, Rectangle};
use bevy::{
    math::DVec2,
    prelude::{info, Component, IVec2, Resource, UVec2, Vec2},
};

use crate::{
    geographic_projection::GeographicProjection,
    projection::{self, Projection},
    tiling_scheme::TilingScheme,
    web_mercator_projection::WebMercatorProjection,
};
#[derive(Debug, Clone, PartialEq, Component)]
pub struct GeographicTilingScheme {
    pub ellipsoid: Ellipsoid,
    pub rectangle: Rectangle,
    pub projection: GeographicProjection,
    pub number_of_level_zero_tiles_x: u32,
    pub number_of_level_zero_tiles_y: u32,
}
pub struct GeographicTilingSchemeOptions<T = GeographicProjection>
where
    T: Projection,
{
    ellipsoid: Ellipsoid,
    rectangle: Rectangle,
    projection: T,
    number_of_level_zero_tiles_x: u32,
    number_of_level_zero_tiles_y: u32,
}
impl Default for GeographicTilingSchemeOptions {
    fn default() -> Self {
        let e = Ellipsoid::WGS84;
        Self {
            ellipsoid: e,
            rectangle: Rectangle::MAX_VALUE,
            projection: GeographicProjection::from_ellipsoid(&e),
            number_of_level_zero_tiles_x: 2,
            number_of_level_zero_tiles_y: 1,
        }
    }
}
impl Default for GeographicTilingScheme {
    fn default() -> Self {
        Self::new(GeographicTilingSchemeOptions::default())
    }
}
impl GeographicTilingScheme {
    fn new(options: GeographicTilingSchemeOptions) -> Self {
        return Self {
            ellipsoid: options.ellipsoid,
            rectangle: options.rectangle,
            projection: options.projection,
            number_of_level_zero_tiles_x: options.number_of_level_zero_tiles_x,
            number_of_level_zero_tiles_y: options.number_of_level_zero_tiles_y,
        };
    }
}
impl TilingScheme for GeographicTilingScheme {
    fn get_name(&self) -> &'static str {
        "GeographicTilingScheme"
    }
    fn get_ellipsoid(&self) -> Ellipsoid {
        return self.ellipsoid;
    }
    fn get_rectangle(&self) -> Rectangle {
        return self.rectangle;
    }
    fn get_number_of_x_tiles_at_level(&self, level: u32) -> u32 {
        return self.number_of_level_zero_tiles_x << level;
    }
    fn get_number_of_y_tiles_at_level(&self, level: u32) -> u32 {
        return self.number_of_level_zero_tiles_y << level;
    }
    fn get_number_of_tiles_at_level(&self, level: u32) -> u32 {
        return self.get_number_of_x_tiles_at_level(level)
            * self.get_number_of_y_tiles_at_level(level);
    }
    fn tile_x_y_to_native_rectange(&self, x: u32, y: u32, level: u32) -> Rectangle {
        let mut rectangle_radians = self.tile_x_y_to_rectange(x, y, level);
        rectangle_radians.west = rectangle_radians.west.to_degrees();
        rectangle_radians.south = rectangle_radians.south.to_degrees();
        rectangle_radians.east = rectangle_radians.east.to_degrees();
        rectangle_radians.north = rectangle_radians.north.to_degrees();
        return rectangle_radians;
    }
    fn tile_x_y_to_rectange(&self, x: u32, y: u32, level: u32) -> Rectangle {
        let rectangle = self.rectangle;

        let x_tiles = self.get_number_of_x_tiles_at_level(level);
        let y_tiles = self.get_number_of_y_tiles_at_level(level);

        let x_tile_width = rectangle.compute_width() / x_tiles as f64;
        let west = (x as f64) * x_tile_width + rectangle.west;
        let east = ((x + 1) as f64) * x_tile_width + rectangle.west;

        let y_tile_height = rectangle.compute_height() / y_tiles as f64;
        let north = rectangle.north - y as f64 * y_tile_height;
        let south = rectangle.north - (y + 1) as f64 * y_tile_height;
        return Rectangle::new(west, south, east, north);
    }
    fn position_to_tile_x_y(&self, coord: &Cartographic, level: u32) -> Option<UVec2> {
        let rectangle = self.rectangle;
        if !rectangle.contains(coord) {
            // outside the bounds of the tiling scheme
            return None;
        }
        let x_tiles = self.get_number_of_x_tiles_at_level(level);
        let y_tiles = self.get_number_of_y_tiles_at_level(level);

        let x_tile_width = rectangle.compute_width() / x_tiles as f64;
        let y_tile_height = rectangle.compute_height() / y_tiles as f64;

        let longitude = coord.longitude;
        let latitude = coord.latitude;

        let mut x_tile_coordinate: u32 =
            ((longitude - rectangle.west) / x_tile_width).floor() as u32;
        if x_tile_coordinate >= x_tiles {
            x_tile_coordinate = x_tiles - 1;
        }

        let mut y_tile_coordinate: u32 =
            ((rectangle.north - latitude) / y_tile_height).floor() as u32;
        if y_tile_coordinate >= y_tiles {
            y_tile_coordinate = y_tiles - 1;
        }

        return Some(UVec2::new(x_tile_coordinate, y_tile_coordinate));
    }
    fn rectangle_to_native_rectangle(&self, rectangle: &Rectangle) -> Rectangle {
        let west = rectangle.west.to_degrees();
        let south = rectangle.south.to_degrees();
        let east = rectangle.east.to_degrees();
        let north = rectangle.north.to_degrees();
        let mut result = Rectangle::default();
        result.west = west;
        result.south = south;
        result.east = east;
        result.north = north;
        return result;
    }
}
