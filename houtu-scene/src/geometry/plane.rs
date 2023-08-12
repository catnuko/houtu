use bevy::math::{DVec3, DVec4};

use crate::{equals_epsilon, EPSILON6};

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Plane {
    pub normal: DVec3,
    pub distance: f64,
}
impl Plane {
    pub fn from_point_normal(point: &DVec3, normal: &DVec3) -> Self {
        let distance = -normal.dot(*point);
        Self {
            normal: normal.clone(),
            distance,
        }
    }
    pub fn get_point_distance(&self, point: DVec3) -> f64 {
        return self.normal.dot(point) + self.distance;
    }
    pub fn project_point_onto_plane(&self, point: DVec3) -> DVec3 {
        return point - self.normal * self.get_point_distance(point);
    }
    pub fn from_vec4(coefficients: &DVec4) -> Self {
        let normal = DVec3::new(coefficients.x, coefficients.y, coefficients.z);
        let distance = coefficients.w;

        //>>includeStart('debug', pragmas.debug);
        if !equals_epsilon(normal.length(), 1.0, Some(EPSILON6), None) {
            panic!("normal must be normalized.");
        }
        //>>includeEnd('debug');
        return Plane {
            distance: distance,
            normal: normal,
        };
    }
}
