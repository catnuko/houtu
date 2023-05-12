use bevy::math::DVec3;

use crate::math::*;

use super::{Plane, Ray};

pub fn rayPlane(ray: Ray, plane: Plane) -> Option<DVec3> {
    let origin = ray.origin;
    let direction = ray.direction;
    let normal = plane.normal;
    let denominator = normal.dot(direction);

    if denominator.abs() < EPSILON15 {
        return None;
    }

    let t = (-plane.distance - normal.dot(origin)) / denominator;

    if (t < 0.) {
        return None;
    }

    return Some(origin + direction * (t));
}
