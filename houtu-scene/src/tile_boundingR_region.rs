use bevy::prelude::Vec3;

use crate::rectangle::Rectangle;

pub struct TileBoundingRegion {
    pub west: f32,
    pub south: f32,
    pub east: f32,
    pub north: f32,
    pub minimum_height: f32,
    pub maximum_height: f32,
    rectangle: Rectangle,
    south_west_corner_cartesian: Vec3,
    north_east_corner_cartesian: Vec3,
    west_normal: Vec3,
    south_normal: Vec3,
    east_normal: Vec3,
    north_normal: Vec3,
}
