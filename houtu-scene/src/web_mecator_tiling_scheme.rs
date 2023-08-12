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
    rectangle_south_west_in_meters: DVec2,
    rectangle_north_east_in_meters: DVec2,
}
pub struct WebMercatorTilingSchemeOptions {
    ellipsoid: Ellipsoid,
    projection: WebMercatorProjection,
    number_of_level_zero_tiles_x: u32,
    number_of_level_zero_tiles_y: u32,
    rectangle_south_west_in_meters: Option<DVec2>,
    rectangle_north_east_in_meters: Option<DVec2>,
}
impl Default for WebMercatorTilingSchemeOptions {
    fn default() -> Self {
        let e = Ellipsoid::WGS84;
        Self {
            ellipsoid: e,
            projection: WebMercatorProjection::from_ellipsoid(&e),
            number_of_level_zero_tiles_x: 1,
            number_of_level_zero_tiles_y: 1,
            rectangle_south_west_in_meters: None,
            rectangle_north_east_in_meters: None,
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
        let mut rectangle_north_east_in_meters: DVec2;
        let mut rectangle_south_west_in_meters: DVec2;
        if options.rectangle_north_east_in_meters.is_some()
            && options.rectangle_south_west_in_meters.is_some()
        {
            rectangle_north_east_in_meters =
                options.rectangle_north_east_in_meters.unwrap().clone();
            rectangle_south_west_in_meters =
                options.rectangle_south_west_in_meters.unwrap().clone();
        } else {
            let semimajorAxisTimesPi = options.ellipsoid.maximum_radius * PI;
            rectangle_south_west_in_meters =
                DVec2::new(-semimajorAxisTimesPi, -semimajorAxisTimesPi);
            rectangle_north_east_in_meters = DVec2::new(semimajorAxisTimesPi, semimajorAxisTimesPi);
        }
        let southwest = options
            .projection
            .un_project(&rectangle_south_west_in_meters);
        let northeast = options
            .projection
            .un_project(&rectangle_north_east_in_meters);
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
            rectangle_south_west_in_meters,
            rectangle_north_east_in_meters,
        };
    }
}
impl TilingScheme for WebMercatorTilingScheme {
    fn get_name(&self) -> &'static str {
        "WebMercatorTilingScheme"
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
        let x_tiles = self.get_number_of_x_tiles_at_level(level);
        let y_tiles = self.get_number_of_y_tiles_at_level(level);

        let x_tile_width = (self.rectangle_north_east_in_meters.x
            - self.rectangle_south_west_in_meters.x)
            / x_tiles as f64;
        let west = (x as f64) * x_tile_width + self.rectangle_south_west_in_meters.x;
        let east = ((x + 1) as f64) * x_tile_width + self.rectangle_south_west_in_meters.x;

        let y_tile_height = (self.rectangle_north_east_in_meters.y
            - self.rectangle_south_west_in_meters.y)
            / y_tiles as f64;
        let north = self.rectangle_north_east_in_meters.y - y as f64 * y_tile_height;
        let south = self.rectangle_north_east_in_meters.y - (y + 1) as f64 * y_tile_height;
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
        if !rectangle.contains(coord) {
            // outside the bounds of the tiling scheme
            return None;
        }
        let x_tiles = self.get_number_of_x_tiles_at_level(level);
        let y_tiles = self.get_number_of_y_tiles_at_level(level);

        let x_tile_width = (self.rectangle_north_east_in_meters.x
            - self.rectangle_south_west_in_meters.x)
            / x_tiles as f64;
        let y_tile_height = (self.rectangle_north_east_in_meters.y
            - self.rectangle_south_west_in_meters.y)
            / y_tiles as f64;

        let web_mercator_position = self.projection.project(coord);
        let distance_from_west = web_mercator_position.x - self.rectangle_south_west_in_meters.x;
        let distance_from_north = self.rectangle_north_east_in_meters.y - web_mercator_position.y;

        let mut x_tile_coordinate: u32 = (distance_from_west / x_tile_width).floor() as u32;
        if x_tile_coordinate >= x_tiles {
            x_tile_coordinate = x_tiles - 1;
        }

        let mut y_tile_coordinate: u32 = (distance_from_north / y_tile_height).floor() as u32;
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

#[cfg(test)]
mod tests {
    use crate::{equals_epsilon, EPSILON10};

    use super::*;

    #[test]
    fn test_tile_xy_to_rectangle() {
        let tiling_scheme = WebMercatorTilingScheme::default();
        let rectangle = tiling_scheme.tile_x_y_to_rectange(0, 0, 0);
        let tiling_scheme_rectangle = tiling_scheme.rectangle;
        assert!(equals_epsilon(
            rectangle.west,
            tiling_scheme_rectangle.west,
            Some(EPSILON10),
            None
        ));
        assert!(equals_epsilon(
            rectangle.south,
            tiling_scheme_rectangle.south,
            Some(EPSILON10),
            None
        ));
        assert!(equals_epsilon(
            rectangle.east,
            tiling_scheme_rectangle.east,
            Some(EPSILON10),
            None
        ));
        assert!(equals_epsilon(
            rectangle.north,
            tiling_scheme_rectangle.north,
            Some(EPSILON10),
            None
        ));
    }
    #[test]
    fn test_tiles_northwest_corner() {
        let tiling_scheme = WebMercatorTilingScheme::default();
        let northwest = tiling_scheme.tile_x_y_to_rectange(0, 0, 1);
        let northeast = tiling_scheme.tile_x_y_to_rectange(1, 0, 1);
        let southeast = tiling_scheme.tile_x_y_to_rectange(1, 1, 1);
        let southwest = tiling_scheme.tile_x_y_to_rectange(0, 1, 1);
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
        let tiling_scheme = WebMercatorTilingScheme::default();

        let center_of_south_western_child = Cartographic::new(-PI / 2.0, -PI / 4.0, 0.);
        assert!(
            tiling_scheme
                .position_to_tile_x_y(&center_of_south_western_child, 1)
                .unwrap()
                == UVec2::new(0, 1)
        );

        let center_of_north_eastern_child = Cartographic::new(PI / 2.0, PI / 4.0, 0.);
        assert!(
            tiling_scheme
                .position_to_tile_x_y(&center_of_north_eastern_child, 1)
                .unwrap()
                == UVec2::new(1, 0)
        );
    }
}
