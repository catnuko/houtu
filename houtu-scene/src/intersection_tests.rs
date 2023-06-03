use bevy::math::DVec3;

use crate::{Cartesian3, Ellipsoid, Plane, Ray, EPSILON15};
#[derive(Default)]
pub struct Interval {
    pub start: f64,
    pub stop: f64,
}
impl Interval {
    pub fn new(start: f64, stop: f64) -> Self {
        Self { start, stop }
    }
}
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
    pub fn rayEllipsoid(ray: &Ray) -> Option<Interval> {
        let ellipsoid = Ellipsoid::WGS84;
        let inverseRadii = ellipsoid.oneOverRadii;
        let q = inverseRadii.multiply_components(&ray.origin);
        let w = inverseRadii.multiply_components(&ray.direction);

        let q2 = q.magnitude_squared();
        let qw = q.dot(w);

        let mut difference;
        let mut w2;
        let mut product;
        let mut discriminant;
        let mut temp;

        if (q2 > 1.0) {
            // Outside ellipsoid.
            if (qw >= 0.0) {
                // Looking outward or tangent (0 intersections).
                return None;
            }

            // qw < 0.0.
            let qw2 = qw * qw;
            difference = q2 - 1.0; // Positively valued.
            w2 = w.magnitude_squared();
            product = w2 * difference;

            if (qw2 < product) {
                // Imaginary roots (0 intersections).
                return None;
            } else if (qw2 > product) {
                // Distinct roots (2 intersections).
                discriminant = qw * qw - product;
                temp = -qw + discriminant.sqrt(); // Avoid cancellation.
                let root0 = temp / w2;
                let root1 = difference / temp;
                if (root0 < root1) {
                    return Some(Interval::new(root0, root1));
                }
                return Some(Interval::new(root1, root0));
            }
            // qw2 == product.  Repeated roots (2 intersections).
            let root = (difference / w2).sqrt();
            return Some(Interval::new(root, root));
        } else if (q2 < 1.0) {
            // Inside ellipsoid (2 intersections).
            difference = q2 - 1.0; // Negatively valued.
            w2 = w.magnitude_squared();
            product = w2 * difference; // Negatively valued.

            discriminant = qw * qw - product;
            temp = -qw + (discriminant).sqrt(); // Positively valued.
            return Some(Interval::new(0.0, temp / w2));
        }
        // q2 == 1.0. On ellipsoid.
        if (qw < 0.0) {
            // Looking inward.
            w2 = w.magnitude_squared();
            return Some(Interval::new(0.0, -qw / w2));
        }

        // qw >= 0.0.  Looking outward or tangent.
        return None;
    }
}
