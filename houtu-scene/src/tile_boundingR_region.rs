use bevy::prelude::Vec3;

use crate::rectangle::Rectangle;

pub struct TileBoundingRegion {
    pub west: f32,
    pub south: f32,
    pub east: f32,
    pub north: f32,
    pub minimumHeight: f32,
    pub maximumHeight: f32,
    rectangle: Rectangle,
    southwestCornerCartesian: Vec3,
    northeastCornerCartesian: Vec3,
    westNormal: Vec3,
    southNormal: Vec3,
    eastNormal: Vec3,
    northNormal: Vec3,
}
