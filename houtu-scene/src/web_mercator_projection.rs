use bevy::math::DVec3;
use std::f64::consts::{FRAC_PI_2, PI, TAU};

use crate::{ellipsoid::Ellipsoid, math::Cartographic, projection::Projection};

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
impl Projection for WebMercatorProjection {
    type Output = WebMercatorProjection;

    fn project(&self, coord: Cartographic) -> DVec3 {
        let semimajorAxis = self.semimajor_axis;
        let x = coord.longitude * semimajorAxis;
        let y = geodeticLatitudeToMercatorAngle(coord.latitude) * semimajorAxis;
        let z = coord.height;
        return DVec3::new(x, y, z);
    }
    fn un_project(&self, vec: DVec3) -> Cartographic {
        let oneOverEarthSemimajorAxis = self.one_over_semimajor_axis;
        let longitude = vec.x * oneOverEarthSemimajorAxis;
        let latitude = mercatorAngleToGeodeticLatitude(vec.y * oneOverEarthSemimajorAxis);
        let height = vec.z;
        return Cartographic::new(longitude, latitude, height);
    }
    fn from_ellipsoid(&self, ellipsoid: Ellipsoid) -> WebMercatorProjection {
        let a = ellipsoid.semimajor_axis();
        let b = 1.0 / ellipsoid.semimajor_axis();
        Self {
            ellipsoid: ellipsoid,
            semimajor_axis: a,
            one_over_semimajor_axis: b,
        }
    }
}
pub fn mercatorAngleToGeodeticLatitude(mercatorAngle: f64) -> f64 {
    return FRAC_PI_2 - 2.0 * (-mercatorAngle).exp().atan();
}
pub fn geodeticLatitudeToMercatorAngle(latitude: f64) -> f64 {
    let sinLatitude = latitude.sin();
    return 0.5 * ((1.0 + sinLatitude) / (1.0 - sinLatitude)).ln();
}
impl WebMercatorProjection {
    const MaximumLatitude: f64 = 1.4844222297453322;
    pub fn geodeticLatitude_to_mercator_angle(&self, latitude: f64) -> f64 {
        let mut latitude = latitude;
        // Clamp the latitude coordinate to the valid Mercator bounds.
        if (latitude > WebMercatorProjection::MaximumLatitude) {
            latitude = WebMercatorProjection::MaximumLatitude;
        } else if (latitude < -WebMercatorProjection::MaximumLatitude) {
            latitude = -WebMercatorProjection::MaximumLatitude;
        }
        let sinLatitude = latitude.sin();
        return 0.5 * (1.0 + sinLatitude) / (1.0 - sinLatitude).ln();
    }
}
