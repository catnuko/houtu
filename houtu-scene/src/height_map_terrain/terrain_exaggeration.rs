use bevy::math::DVec3;

use crate::{Cartesian3, Ellipsoid};

pub struct TerrainExaggeration;
impl TerrainExaggeration {
    pub fn getHeight(height: f64, scale: f64, relativeHeight: f64) -> f64 {
        return (height - relativeHeight) * scale + relativeHeight;
    }
    pub fn getPosition(
        position: &DVec3,
        ellipsoid: &Ellipsoid,
        terrainExaggeration: f64,
        terrainExaggerationRelativeHeight: f64,
    ) -> DVec3 {
        let cartographic = ellipsoid
            .cartesianToCartographic(position)
            .expect("TerrainExaggeration getPosition error");
        let newHeight = TerrainExaggeration::getHeight(
            cartographic.height,
            terrainExaggeration,
            terrainExaggerationRelativeHeight,
        );
        return DVec3::from_radians(
            cartographic.longitude,
            cartographic.latitude,
            Some(newHeight),
            Some(ellipsoid.radiiSquared),
        );
    }
}
