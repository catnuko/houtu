use std::f64::consts::PI;

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

#[derive(Debug, Clone)]
pub struct WebMercatorTilingScheme {
    pub ellipsoid: Ellipsoid,
    pub rectangle: Rectangle,
    pub projection: WebMercatorProjection,
    pub number_of_level_zero_tiles_x: u32,
    pub number_of_level_zero_tiles_y: u32,
    rectangleSouthwestInMeters: DVec2,
    rectangleNortheastInMeters: DVec2,
}
pub struct WebMercatorTilingSchemeOptions {
    ellipsoid: Ellipsoid,
    projection: WebMercatorProjection,
    number_of_level_zero_tiles_x: u32,
    number_of_level_zero_tiles_y: u32,
    rectangleSouthwestInMeters: Option<DVec2>,
    rectangleNortheastInMeters: Option<DVec2>,
}
impl Default for WebMercatorTilingSchemeOptions {
    fn default() -> Self {
        let e = Ellipsoid::WGS84;
        Self {
            ellipsoid: e,
            projection: WebMercatorProjection::from_ellipsoid(&e),
            number_of_level_zero_tiles_x: 1,
            number_of_level_zero_tiles_y: 1,
            rectangleSouthwestInMeters: None,
            rectangleNortheastInMeters: None,
        }
    }
}
impl Default for WebMercatorTilingScheme {
    fn default() -> Self {
        Self::new(WebMercatorTilingSchemeOptions::default())
    }
}
impl WebMercatorTilingScheme {
    fn new(options: WebMercatorTilingSchemeOptions) -> Self {
        let mut rectangleNortheastInMeters: DVec2;
        let mut rectangleSouthwestInMeters: DVec2;
        if options.rectangleNortheastInMeters.is_some()
            && options.rectangleSouthwestInMeters.is_some()
        {
            rectangleNortheastInMeters = options.rectangleNortheastInMeters.unwrap().clone();
            rectangleSouthwestInMeters = options.rectangleSouthwestInMeters.unwrap().clone();
        } else {
            let semimajorAxisTimesPi = options.ellipsoid.maximumRadius * PI;
            rectangleSouthwestInMeters = DVec2::new(-semimajorAxisTimesPi, -semimajorAxisTimesPi);
            rectangleNortheastInMeters = DVec2::new(semimajorAxisTimesPi, semimajorAxisTimesPi);
        }
        let southwest = options.projection.un_project(&rectangleSouthwestInMeters);
        let northeast = options.projection.un_project(&rectangleNortheastInMeters);
        let rectangle = Rectangle::new(
            southwest.longitude,
            southwest.latitude,
            northeast.longitude,
            northeast.latitude,
        );
        return Self {
            ellipsoid: options.ellipsoid,
            rectangle: rectangle,
            projection: options.projection,
            number_of_level_zero_tiles_x: options.number_of_level_zero_tiles_x,
            number_of_level_zero_tiles_y: options.number_of_level_zero_tiles_y,
            rectangleSouthwestInMeters,
            rectangleNortheastInMeters,
        };
    }
}
impl TilingScheme for WebMercatorTilingScheme {
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
        let xTiles = self.get_number_of_x_tiles_at_level(level);
        let yTiles = self.get_number_of_y_tiles_at_level(level);

        let xTileWidth =
            (self.rectangleNortheastInMeters.x - self.rectangleSouthwestInMeters.x) / xTiles as f64;
        let west = (x as f64) * xTileWidth + self.rectangleSouthwestInMeters.x;
        let east = ((x + 1) as f64) * xTileWidth + self.rectangleSouthwestInMeters.x;

        let yTileHeight =
            (self.rectangleNortheastInMeters.y - self.rectangleSouthwestInMeters.y) / yTiles as f64;
        let north = self.rectangleNortheastInMeters.y - y as f64 * yTileHeight;
        let south = self.rectangleNortheastInMeters.y - (y + 1) as f64 * yTileHeight;
        return Rectangle::new(west, south, east, north);
    }
    fn tile_x_y_to_rectange(&self, x: u32, y: u32, level: u32) -> Rectangle {
        let mut nativeRectangle = self.tile_x_y_to_native_rectange(x, y, level);
        let southwest = self
            .projection
            .un_project(&DVec2::new(nativeRectangle.west, nativeRectangle.south));
        let northeast = self
            .projection
            .un_project(&DVec2::new(nativeRectangle.east, nativeRectangle.north));

        nativeRectangle.west = southwest.longitude;
        nativeRectangle.south = southwest.latitude;
        nativeRectangle.east = northeast.longitude;
        nativeRectangle.north = northeast.latitude;
        return nativeRectangle;
    }
    fn position_to_tile_x_y(&self, coord: &Cartographic, level: u32) -> Option<UVec2> {
        let rectangle = self.rectangle;
        if (!rectangle.contains(coord)) {
            // outside the bounds of the tiling scheme
            return None;
        }
        let xTiles = self.get_number_of_x_tiles_at_level(level);
        let yTiles = self.get_number_of_y_tiles_at_level(level);

        let xTileWidth =
            (self.rectangleNortheastInMeters.x - self.rectangleSouthwestInMeters.x) / xTiles as f64;
        let yTileHeight =
            (self.rectangleNortheastInMeters.y - self.rectangleSouthwestInMeters.y) / yTiles as f64;

        let webMercatorPosition = self.projection.project(coord);
        let distanceFromWest = webMercatorPosition.x - self.rectangleSouthwestInMeters.x;
        let distanceFromNorth = self.rectangleNortheastInMeters.y - webMercatorPosition.y;

        let mut xTileCoordinate: u32 = (distanceFromWest / xTileWidth).floor() as u32;
        if (xTileCoordinate >= xTiles) {
            xTileCoordinate = xTiles - 1;
        }

        let mut yTileCoordinate: u32 = (distanceFromNorth / yTileHeight).floor() as u32;
        if (yTileCoordinate >= yTiles) {
            yTileCoordinate = yTiles - 1;
        }

        return Some(UVec2::new(xTileCoordinate, yTileCoordinate));
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

#[cfg(test)]
mod tests {
    use crate::{equals_epsilon, EPSILON10};

    use super::*;

    #[test]
    fn test_tileXYToRectangle() {
        let tilingScheme = WebMercatorTilingScheme::default();
        let rectangle = tilingScheme.tile_x_y_to_rectange(0, 0, 0);
        let tilingSchemeRectangle = tilingScheme.rectangle;
        assert!(equals_epsilon(
            rectangle.west,
            tilingSchemeRectangle.west,
            Some(EPSILON10),
            None
        ));
        assert!(equals_epsilon(
            rectangle.south,
            tilingSchemeRectangle.south,
            Some(EPSILON10),
            None
        ));
        assert!(equals_epsilon(
            rectangle.east,
            tilingSchemeRectangle.east,
            Some(EPSILON10),
            None
        ));
        assert!(equals_epsilon(
            rectangle.north,
            tilingSchemeRectangle.north,
            Some(EPSILON10),
            None
        ));
    }
    #[test]
    fn test_tiles_northwest_corner() {
        let tilingScheme = WebMercatorTilingScheme::default();
        let northwest = tilingScheme.tile_x_y_to_rectange(0, 0, 1);
        let northeast = tilingScheme.tile_x_y_to_rectange(1, 0, 1);
        let southeast = tilingScheme.tile_x_y_to_rectange(1, 1, 1);
        let southwest = tilingScheme.tile_x_y_to_rectange(0, 1, 1);
        assert!(northeast.north == northwest.north);
        assert!(northeast.south == northwest.south);
        assert!(southeast.north == southwest.north);
        assert!(southeast.south == southwest.south);

        assert!(northwest.west == southwest.west);
        assert!(northwest.east == southwest.east);
        assert!(northeast.west == southeast.west);
        assert!(northeast.east == southeast.east);
        assert!(northeast.north > southeast.north);
        assert!(northeast.south > southeast.south);
        assert!(northwest.north > southwest.north);
        assert!(northwest.south > southwest.south);

        assert!(northeast.east > northwest.east);
        assert!(northeast.west > northwest.west);
        assert!(southeast.east > southwest.east);
        assert!(southeast.west > southwest.west);
    }
    #[test]
    fn test_return_correct_tile() {
        let tilingScheme = WebMercatorTilingScheme::default();

        let centerOfSouthwesternChild = Cartographic::new(-PI / 2.0, -PI / 4.0, 0.);
        assert!(
            tilingScheme
                .position_to_tile_x_y(&centerOfSouthwesternChild, 1)
                .unwrap()
                == UVec2::new(0, 1)
        );

        let centerOfNortheasternChild = Cartographic::new(PI / 2.0, PI / 4.0, 0.);
        assert!(
            tilingScheme
                .position_to_tile_x_y(&centerOfNortheasternChild, 1)
                .unwrap()
                == UVec2::new(1, 0)
        );
    }
}
