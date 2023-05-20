use crate::{bit_or_zero, Cartographic, Ellipsoid, Rectangle};
use bevy::{
    math::DVec2,
    prelude::{IVec2, UVec2, Vec2},
};

use crate::{
    geographic_projection::GeographicProjection,
    projection::{self, Projection},
    tiling_scheme::TilingScheme,
    web_mercator_projection::WebMercatorProjection,
};

#[derive(Debug, Clone, PartialEq)]
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
        let mut rectangleRadians = self.tile_x_y_to_rectange(x, y, level);
        rectangleRadians.west = rectangleRadians.west.to_degrees();
        rectangleRadians.south = rectangleRadians.south.to_degrees();
        rectangleRadians.east = rectangleRadians.east.to_degrees();
        rectangleRadians.north = rectangleRadians.north.to_degrees();
        return rectangleRadians;
    }
    fn tile_x_y_to_rectange(&self, x: u32, y: u32, level: u32) -> Rectangle {
        let rectangle = self.rectangle;

        let xTiles = self.get_number_of_x_tiles_at_level(level);
        let yTiles = self.get_number_of_y_tiles_at_level(level);

        let xTileWidth = rectangle.computeWidth() / xTiles as f64;
        let west = (x as f64) * xTileWidth + rectangle.west;
        let east = ((x + 1) as f64) * xTileWidth + rectangle.west;

        let yTileHeight = rectangle.computeHeight() / yTiles as f64;
        let north = rectangle.north - y as f64 * yTileHeight;
        let south = rectangle.north - (y + 1) as f64 * yTileHeight;
        return Rectangle::new(west, south, east, north);
    }
    fn position_to_tile_x_y(&self, coord: &Cartographic, level: u32) -> Option<UVec2> {
        let rectangle = self.rectangle;
        if (!rectangle.contains(coord)) {
            // outside the bounds of the tiling scheme
            return None;
        }
        let xTiles = self.get_number_of_x_tiles_at_level(level);
        let yTiles = self.get_number_of_y_tiles_at_level(level);

        let xTileWidth = rectangle.computeWidth() / xTiles as f64;
        let yTileHeight = rectangle.computeHeight() / yTiles as f64;

        let longitude = coord.longitude;
        let latitude = coord.latitude;

        let mut xTileCoordinate: u32 = ((longitude - rectangle.west) / xTileWidth).floor() as u32;
        if (xTileCoordinate >= xTiles) {
            xTileCoordinate = xTiles - 1;
        }

        let mut yTileCoordinate: u32 = ((rectangle.north - latitude) / yTileHeight).floor() as u32;
        if (yTileCoordinate >= yTiles) {
            yTileCoordinate = yTiles - 1;
        }

        return Some(UVec2::new(xTileCoordinate, yTileCoordinate));
    }
}
