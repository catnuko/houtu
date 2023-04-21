use bevy::prelude::Vec2;
use geodesy::{Coord, Ellipsoid};

use crate::{
    geographic_projection::GeographicProjection,
    projection::{self, Projection},
    rectangle::Rectangle,
    tiling_scheme::TilingScheme,
    web_mercator_projection::WebMercatorProjection,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct GeographicTilingScheme<T = GeographicProjection>
where
    T: Projection,
{
    ellipsoid: Ellipsoid,
    rectangle: Rectangle,
    projection: T,
    number_of_level_zero_tiles_x: u32,
    number_of_level_zero_tiles_y: u32,
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
impl Default for GeographicTilingSchemeOptions<T>
where
    T: Projection,
{
    fn default() -> Self {
        let e = Ellipsoid::named("WGS84");
        Self {
            ellipsoid: e,
            rectangle: Rectangle::default(),
            projection: GeographicProjection::from_ellipsoid(e),
            number_of_level_zero_tiles_x: 1,
            number_of_level_zero_tiles_y: 1,
        }
    }
}
impl Default for GeographicTilingScheme<T>
where
    T: Projection,
{
    fn default() -> Self {
        Self::new(GeographicTilingSchemeOptions::default())
    }
}
impl GeographicTilingScheme<T>
where
    T: Projection,
{
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
    fn get_projection(&self) -> dyn Projection {
        return self.projection;
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
        rectangleRadians.west = rectangleRadians.west.to_radians();
        rectangleRadians.south = rectangleRadians.south.to_radians();
        rectangleRadians.east = rectangleRadians.east.to_radians();
        rectangleRadians.north = rectangleRadians.north.to_radians();
        return rectangleRadians;
    }
    fn tile_x_y_to_rectange(&self, x: u32, y: u32, level: u32) -> Rectangle {
        let rectangle = self._rectangle;

        let xTiles = self.get_number_of_x_tiles_at_level(level);
        let yTiles = self.get_number_of_y_tiles_at_level(level);

        let xTileWidth = rectangle.width / xTiles;
        let west = x * xTileWidth + rectangle.west;
        let east = (x + 1) * xTileWidth + rectangle.west;

        let yTileHeight = rectangle.height / yTiles;
        let north = rectangle.north - y * yTileHeight;
        let south = rectangle.north - (y + 1) * yTileHeight;
        return Rectangle::new(west, south, east, north);
    }
    fn position_to_tile_x_y(&self, coord: Coord, level: u32) -> Option<Vec2> {
        let rectangle = self.rectangle;
        if (!rectangle.contains(coord)) {
            // outside the bounds of the tiling scheme
            return None;
        }
        let xTiles = self.get_number_of_x_tiles_at_level(level);
        let yTiles = self.get_number_of_y_tiles_at_level(level);

        let xTileWidth = rectangle.width / xTiles;
        let yTileHeight = rectangle.height / yTiles;

        let longitude = coord.first();
        let latitude = coord.second();

        let xTileCoordinate = ((longitude - rectangle.west) / xTileWidth) | 0;
        if (xTileCoordinate >= xTiles) {
            xTileCoordinate = xTiles - 1;
        }

        let yTileCoordinate = ((rectangle.north - latitude) / yTileHeight) | 0;
        if (yTileCoordinate >= yTiles) {
            yTileCoordinate = yTiles - 1;
        }

        return Some(Vec2::new(xTileCoordinate, yTileCoordinate));
    }
}
