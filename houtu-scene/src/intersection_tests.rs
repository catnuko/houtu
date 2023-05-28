use bevy::math::DVec3;

use crate::{Cartesian3, Plane, Ray, EPSILON15};

pub struct IntersectionTests;
impl IntersectionTests {
    pub fn rayPlane(ray: &Ray, plane: &Plane) -> Option<DVec3> {
        let origin = ray.origin;
        let direction = ray.direction;
        let normal = plane.normal;
        let denominator = normal.dot(direction);

        if (denominator.abs() < EPSILON15) {
            // Ray is parallel to plane.  The ray may be in the polygon's plane.
            return None;
        }

        let t = (-plane.distance - normal.dot(origin)) / denominator;

        if (t < 0.) {
            return None;
        }

        return Some(origin + direction.multiply_by_scalar(t));
    }
}
