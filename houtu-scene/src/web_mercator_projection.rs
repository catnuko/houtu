use bevy::prelude::*;
use geodesy::preamble::*;
use std::f32::consts::{FRAC_PI_2, PI, TAU};

use crate::projection::Projection;

pub struct WebMercatorProjection {
    ellipsoid: Ellipsoid,
    semimajor_axis: f64,
    one_over_semimajor_axis: f64,
}
impl Default for WebMercatorProjection {
    fn default() -> Self {
        let e = Ellipsoid::named("WGS84");
        Self {
            ellipsoid: e,
            semimajor_axis: e.semimajor_axis(),
            one_over_semimajor_axis: 1.0 / e.semimajor_axis(),
        }
    }
}
impl Projection for WebMercatorProjection {
    fn project(&self, coord: Coord) -> Vec3 {
        let semimajorAxis = self.semimajor_axis;
        let x = coord.first() * semimajorAxis;
        let y = geodeticLatitudeToMercatorAngle(coord.second()) * semimajorAxis;
        let z = coord.third();
        return Vec3::new(x, y, z);
    }
    fn un_project(&self, vec: Vec3) -> Coord {
        let oneOverEarthSemimajorAxis = self._oneOverSemimajorAxis;
        let longitude = vec.x * oneOverEarthSemimajorAxis;
        let latitude = mercatorAngleToGeodeticLatitude(vec.y * oneOverEarthSemimajorAxis);
        let height = vec.z;
        return Coord::gis(longitude, latitude, height, 0.0);
    }
}
pub fn mercatorAngleToGeodeticLatitude(mercatorAngle: f64) -> f64 {
    return FRAC_PI_2 - 2.0 * (-mercatorAngle).exp().atan();
}
pub fn geodeticLatitudeToMercatorAngle(latitude: f64) -> f64 {
    let sinLatitude = latitude.sin();
    return 0.5 * ((1.0 + sinLatitude) / (1.0 - sinLatitude)).ln();
}
