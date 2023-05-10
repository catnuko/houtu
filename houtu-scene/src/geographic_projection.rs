use bevy::{math::DVec3, prelude::Vec3};

use crate::{ellipsoid::Ellipsoid, math::Cartographic, projection::Projection};

pub struct GeographicProjection {
    ellipsoid: Ellipsoid,
    semimajor_axis: f64,
    one_over_semimajor_axis: f64,
}
impl Default for GeographicProjection {
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
impl Projection for GeographicProjection {
    type Output = GeographicProjection;
    fn project(&self, coord: Cartographic) -> DVec3 {
        let semimajorAxis = self.semimajor_axis;
        let x = coord.longitude * semimajorAxis;
        let y = coord.latitude * semimajorAxis;
        let z = coord.height;
        DVec3::new(x, y, z)
    }
    fn un_project(&self, vec: DVec3) -> Cartographic {
        let one_over_semimajor_axis = self.one_over_semimajor_axis;
        let x = vec.x * one_over_semimajor_axis;
        let y = vec.y * one_over_semimajor_axis;
        let z = vec.z;
        Cartographic::new(x, y, z)
    }
    fn from_ellipsoid(&self, ellipsoid: Ellipsoid) -> GeographicProjection {
        let a = ellipsoid.semimajor_axis();
        let b = 1.0 / ellipsoid.semimajor_axis();
        Self {
            ellipsoid: ellipsoid,
            semimajor_axis: a,
            one_over_semimajor_axis: b,
        }
    }
}
