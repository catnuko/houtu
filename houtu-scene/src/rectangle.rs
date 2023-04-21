use geodesy::preamble::*;
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, Debug)]
pub struct Rectangle {
    pub west: f32,
    pub south: f32,
    pub east: f32,
    pub north: f32,
}
impl Rectangle {
    pub fn width(&self) -> f32 {
        return self.compute_width();
    }
    pub fn height(&self) -> f32 {
        return self.compute_height();
    }
    pub fn compute_width(&self) -> f32 {
        let east = self.east;
        let west = self.west;
        if (east < west) {
            east += TAU;
        }
        return east - west;
    }
    pub fn compute_height(&self) -> f32 {
        return self.north - self.south;
    }
    pub fn from_degree(west: f32, south: f32, east: f32, north: f32) -> Self {
        Self {
            west: west.to_radians(),
            south: south.to_radians(),
            east: east.to_radians(),
            north: north.to_radians(),
        }
    }
    pub fn from_radian(west: f32, south: f32, east: f32, north: f32) -> Self {
        Self {
            west: west,
            south: south,
            east: east,
            north: north,
        }
    }
    pub fn from_center(center: Coord, width: f32, height: f32) -> Self {
        let half_width = width / 2.;
        let half_height = height / 2.;
        let west = center.lon - half_width;
        let east = center.lon + half_width;
        let south = center.lat - half_height;
        let north = center.lat + half_height;
        Self {
            west: west,
            south: south,
            east: east,
            north: north,
        }
    }
    pub fn equals(&self, other: &Self) -> bool {
        return self.west == other.west
            && self.south == other.south
            && self.east == other.east
            && self.north == other.north;
    }

    pub fn south_west(&self) -> Coord {
        return Coord::gis(self.west, self.south, 0.0, 0.0);
    }
    pub fn north_west(&self) -> Coord {
        return Coord::gis(self.west, self.north, 0.0, 0.0);
    }
    pub fn south_east(&self) -> Coord {
        return Coord::gis(self.east, self.south, 0.0, 0.0);
    }
    pub fn north_east(&self) -> Coord {
        return Coord::gis(self.east, self.north, 0.0, 0.0);
    }
    pub fn center(&self) -> Coord {
        let center_lon = (self.west + self.east) / 2.;
        let center_lat = (self.south + self.north) / 2.;
        return Coord::gis(center_lon, center_lat, 0.0, 0.0);
    }
    pub fn contains(coord: &Coord) -> bool {
        return false;
    }
}
