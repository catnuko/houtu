use bevy::math::DVec3;

use crate::{Cartesian3, Ellipsoid};

pub struct TerrainExaggeration;
impl TerrainExaggeration {
    pub fn get_height(height: f64, scale: f64, relative_height: f64) -> f64 {
        return (height - relative_height) * scale + relative_height;
    }
    pub fn get_position(
        position: &DVec3,
        ellipsoid: &Ellipsoid,
        terrain_exaggeration: f64,
        terrain_exaggeration_relative_height: f64,
    ) -> DVec3 {
        let cartographic = ellipsoid
            .cartesian_to_cartographic(position)
            .expect("TerrainExaggeration get_position error");
        let new_height = TerrainExaggeration::get_height(
            cartographic.height,
            terrain_exaggeration,
            terrain_exaggeration_relative_height,
        );
        return DVec3::from_radians(
            cartographic.longitude,
            cartographic.latitude,
            Some(new_height),
            Some(ellipsoid.radii_squared),
        );
    }
}
