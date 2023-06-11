use std::f64::{MAX, NEG_INFINITY};

use bevy::math::{DMat3, DVec3};

use crate::{
    ellipsoid, Cartesian3, Ellipsoid, Matrix3, Plane, QuadraticRealPolynomial,
    QuarticRealPolynomial, Ray, EPSILON12, EPSILON15,
};
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
    pub fn rayEllipsoid(ray: &Ray, ellipsoid: Option<&Ellipsoid>) -> Option<Interval> {
        let ellipsoid = ellipsoid.unwrap_or(&Ellipsoid::WGS84);
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
    pub fn grazingAltitudeLocation(ray: &Ray, ellipsoid: Option<&Ellipsoid>) -> Option<DVec3> {
        let ellipsoid = ellipsoid.unwrap_or(&Ellipsoid::WGS84);
        let position = ray.origin;
        let direction = ray.direction;

        if (!position.equals(DVec3::ZERO)) {
            let normal = ellipsoid.geodeticSurfaceNormal(&position).unwrap();
            if (direction.dot(normal) >= 0.0) {
                // The location provided is the closest point in altitude
                return Some(position);
            }
        }

        let intersects = IntersectionTests::rayEllipsoid(&ray, Some(ellipsoid)).is_some();

        // Compute the scaled direction vector.
        let f = ellipsoid.transformPositionToScaledSpace(&direction);

        // letructs a basis from the unit scaled direction vector. letruct its rotation and transpose.
        let firstAxis = f.normalize();
        let reference = f.most_orthogonal_axis();
        let secondAxis = (reference.cross(firstAxis)).normalize();
        let thirdAxis = (firstAxis.cross(secondAxis)).normalize();
        let mut BB: [f64; 9] = [0.; 9];
        BB[0] = firstAxis.x;
        BB[1] = firstAxis.y;
        BB[2] = firstAxis.z;
        BB[3] = secondAxis.x;
        BB[4] = secondAxis.y;
        BB[5] = secondAxis.z;
        BB[6] = thirdAxis.x;
        BB[7] = thirdAxis.y;
        BB[8] = thirdAxis.z;
        let B = DMat3::from_row_list(BB);
        let B_T = B.transpose();

        // Get the scaling matrix and its inverse.
        let D_I = DMat3::from_scale3(ellipsoid.radii);
        let D = DMat3::from_scale3(ellipsoid.oneOverRadii);

        let mut CC: [f64; 9] = [0.; 9];
        CC[0] = 0.0;
        CC[1] = -direction.z;
        CC[2] = direction.y;
        CC[3] = direction.z;
        CC[4] = 0.0;
        CC[5] = -direction.x;
        CC[6] = -direction.y;
        CC[7] = direction.x;
        CC[8] = 0.0;
        let C = DMat3::from_row_list(CC);

        let temp = B_T.mul_mat3(&D).mul_mat3(&C);
        let A = temp.mul_mat3(&D_I).mul_mat3(&B);
        let b = temp.mul_vec3(position);

        // Solve for the solutions to the expression in standard form:
        let solutions = quadraticVectorExpression(&A, &b.negate(), 0.0, 0.0, 1.0);

        let mut s;
        let mut altitude;
        let length = solutions.len();
        if (length > 0) {
            let mut closest = DVec3::ZERO.clone();
            let mut maximumValue = NEG_INFINITY;
            for i in 0..length {
                s = D_I.mul_vec3(B.mul_vec3(solutions[i]));
                let v = (s.subtract(position)).normalize();
                let dotProduct = v.dot(direction);

                if (dotProduct > maximumValue) {
                    maximumValue = dotProduct;
                    closest = s.clone();
                }
            }

            let mut surfacePoint = ellipsoid.cartesianToCartographic(&closest).unwrap();
            maximumValue = maximumValue.clamp(0.0, 1.0);
            altitude =
                closest.subtract(position).magnitude() * (1.0 - maximumValue * maximumValue).sqrt();
            altitude = if intersects { -altitude } else { altitude };

            surfacePoint.height = altitude;
            return Some(ellipsoid.cartographicToCartesian(&surfacePoint));
        }

        return None;
    }
}
fn addWithCancellationCheck(left: f64, right: f64, tolerance: f64) -> f64 {
    let difference = left + right;
    if (left.signum() != right.signum()
        && (difference / left.abs().max(right.abs())).abs() < tolerance)
    {
        return 0.0;
    }

    return difference;
}
fn quadraticVectorExpression(A: &DMat3, b: &DVec3, c: f64, x: f64, w: f64) -> Vec<DVec3> {
    let mut AA: [f64; 9] = [0.; 9];
    A.write_cols_to_slice(&mut AA);

    let xSquared = x * x;
    let wSquared = w * w;

    let l2 = (AA[DMat3::COLUMN1ROW1] - AA[DMat3::COLUMN2ROW2]) * wSquared;
    let l1 = w
        * (x * addWithCancellationCheck(AA[DMat3::COLUMN1ROW0], AA[DMat3::COLUMN0ROW1], EPSILON15)
            + b.y);
    let l0 = AA[DMat3::COLUMN0ROW0] * xSquared + AA[DMat3::COLUMN2ROW2] * wSquared + x * b.x + c;

    let r1 = wSquared
        * addWithCancellationCheck(AA[DMat3::COLUMN2ROW1], AA[DMat3::COLUMN1ROW2], EPSILON15);
    let r0 = w
        * (x * addWithCancellationCheck(AA[DMat3::COLUMN2ROW0], AA[DMat3::COLUMN0ROW2], EPSILON15)
            + b.z);

    let cosines;
    let mut solutions = vec![];
    if (r0 == 0.0 && r1 == 0.0) {
        cosines = QuadraticRealPolynomial::computeRealRoots(l2, l1, l0).unwrap();
        if (cosines.len() == 0) {
            return solutions;
        }

        let cosine0 = cosines[0];
        let sine0 = ((1.0 - cosine0 * cosine0).max(0.0)).sqrt();
        solutions.push(DVec3::new(x, w * cosine0, w * -sine0));
        solutions.push(DVec3::new(x, w * cosine0, w * sine0));

        if (cosines.len() == 2) {
            let cosine1 = cosines[1];
            let sine1 = ((1.0 - cosine1 * cosine1).max(0.0)).sqrt();
            solutions.push(DVec3::new(x, w * cosine1, w * -sine1));
            solutions.push(DVec3::new(x, w * cosine1, w * sine1));
        }

        return solutions;
    }

    let r0Squared = r0 * r0;
    let r1Squared = r1 * r1;
    let l2Squared = l2 * l2;
    let r0r1 = r0 * r1;

    let c4 = l2Squared + r1Squared;
    let c3 = 2.0 * (l1 * l2 + r0r1);
    let c2 = 2.0 * l0 * l2 + l1 * l1 - r1Squared + r0Squared;
    let c1 = 2.0 * (l0 * l1 - r0r1);
    let c0 = l0 * l0 - r0Squared;

    if (c4 == 0.0 && c3 == 0.0 && c2 == 0.0 && c1 == 0.0) {
        return solutions;
    }

    cosines = QuarticRealPolynomial::computeRealRoots(c4, c3, c2, c1, c0).unwrap();
    let length = cosines.len();
    if (length == 0) {
        return solutions;
    }
    for i in 0..length {
        let cosine = cosines[i];
        let cosineSquared = cosine * cosine;
        let sineSquared = (1.0 - cosineSquared).max(0.0);
        let sine = sineSquared.sqrt();

        //let left = l2 * cosineSquared + l1 * cosine + l0;
        let left;
        if (l2.signum() == l0.signum()) {
            left = addWithCancellationCheck(l2 * cosineSquared + l0, l1 * cosine, EPSILON12);
        } else if (l0.signum() == (l1 * cosine).signum()) {
            left = addWithCancellationCheck(l2 * cosineSquared, l1 * cosine + l0, EPSILON12);
        } else {
            left = addWithCancellationCheck(l2 * cosineSquared + l1 * cosine, l0, EPSILON12);
        }

        let right = addWithCancellationCheck(r1 * cosine, r0, EPSILON15);
        let product = left * right;

        if (product < 0.0) {
            solutions.push(DVec3::new(x, w * cosine, w * sine));
        } else if (product > 0.0) {
            solutions.push(DVec3::new(x, w * cosine, w * -sine));
        } else if (sine != 0.0) {
            solutions.push(DVec3::new(x, w * cosine, w * -sine));
            solutions.push(DVec3::new(x, w * cosine, w * sine));
            // i += 1;
        } else {
            solutions.push(DVec3::new(x, w * cosine, w * sine));
        }
    }

    return solutions;
}
#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use crate::math::{equals_epsilon, EPSILON14};

    use super::*;

    #[test]
    fn ray_inside_pointing_in_intersection() {
        let origin = DVec3::new(20000.0, 0.0, 0.0);
        let direction = origin.normalize().negate();
        let ray = Ray::new(origin, direction);
        let actual = IntersectionTests::rayEllipsoid(&ray, None);
        let actual = actual.unwrap();
        assert!(actual.start == 0.0);
        assert!(actual.stop == Ellipsoid::WGS84.radii.x + origin.x);
    }
    #[test]
    fn ray_inside_pointing_out_intersection() {
        let origin = DVec3::new(20000.0, 0.0, 0.0);
        let direction = origin.normalize();
        let ray = Ray::new(origin, direction);
        let actual = IntersectionTests::rayEllipsoid(&ray, None);
        let actual = actual.unwrap();
        assert!(actual.start == 0.0);
        assert!(actual.stop == Ellipsoid::WGS84.radii.x - origin.x);
    }
    #[test]
    fn tangent_intersections() {
        let ray = Ray::new(DVec3::UNIT_X, DVec3::UNIT_Z);
        let actual = IntersectionTests::rayEllipsoid(&ray, Some(&Ellipsoid::UNIT_SPHERE));
        assert!(actual.is_none());
    }
    #[test]
    fn grazingAltitudeLocation_inside_ellipsoid() {
        let ellipsoid = Ellipsoid::UNIT_SPHERE;
        let ray = Ray::new(DVec3::new(0.5, 0.0, 0.0), DVec3::UNIT_Z);
        let actual = IntersectionTests::grazingAltitudeLocation(&ray, Some(&ellipsoid));
        assert!(actual.unwrap().eq(&ray.origin));
    }
}
