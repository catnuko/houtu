use bevy::math::{DVec2, DVec3};
use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::{ellipsoid::Ellipsoid, math::Cartographic, projection::Projection};
#[derive(Debug, Clone)]
pub struct WebMercatorProjection {
    pub ellipsoid: Ellipsoid,
    pub semimajor_axis: f64,
    pub one_over_semimajor_axis: f64,
}
impl Default for WebMercatorProjection {
    fn default() -> Self {
        let e = Ellipsoid::WGS84;
        let a = e.semimajor_axis();
        let b = 1.0 / e.semimajor_axis();
        Self {
            ellipsoid: e,
            semimajor_axis: a,
            one_over_semimajor_axis: b,
        }
    }
}
impl WebMercatorProjection {
    pub fn project(&self, coord: &Cartographic) -> DVec3 {
        let semimajor_axis = self.semimajor_axis;
        let x = coord.longitude * semimajor_axis;
        let y = geodetic_latitude_to_mercator_angle(coord.latitude) * semimajor_axis;
        let z = coord.height;
        return DVec3::new(x, y, z);
    }
    pub fn un_project(&self, vec: &DVec2) -> Cartographic {
        let one_over_earth_semimajor_axis = self.one_over_semimajor_axis;
        let longitude = vec.x * one_over_earth_semimajor_axis;
        let latitude = mercator_angle_to_geodetic_latitude(vec.y * one_over_earth_semimajor_axis);
        let height = 0.;
        return Cartographic::new(longitude, latitude, height);
    }
    pub fn from_ellipsoid(ellipsoid: &Ellipsoid) -> WebMercatorProjection {
        let a = ellipsoid.semimajor_axis();
        let b = 1.0 / ellipsoid.semimajor_axis();
        Self {
            ellipsoid: ellipsoid.clone(),
            semimajor_axis: a,
            one_over_semimajor_axis: b,
        }
    }
}
pub fn mercator_angle_to_geodetic_latitude(mercator_angle: f64) -> f64 {
    return FRAC_PI_2 - 2.0 * (-mercator_angle).exp().atan();
}
pub fn geodetic_latitude_to_mercator_angle(latitude: f64) -> f64 {
    let sin_latitude = latitude.sin();
    return 0.5 * ((1.0 + sin_latitude) / (1.0 - sin_latitude)).ln();
}
impl WebMercatorProjection {
    pub const MAXIMUM_LATITUDE: f64 = 1.4844222297453322;
    pub fn geodetic_latitude_to_mercator_angle(&self, latitude: f64) -> f64 {
        let mut latitude = latitude;
        // Clamp the latitude coordinate to the valid Mercator bounds.
        if latitude > WebMercatorProjection::MAXIMUM_LATITUDE {
            latitude = WebMercatorProjection::MAXIMUM_LATITUDE;
        } else if latitude < -WebMercatorProjection::MAXIMUM_LATITUDE {
            latitude = -WebMercatorProjection::MAXIMUM_LATITUDE;
        }
        let sin_latitude = latitude.sin();
        return 0.5 * (1.0 + sin_latitude) / (1.0 - sin_latitude).ln();
    }
}
