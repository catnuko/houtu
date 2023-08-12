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

        if denominator.abs() < EPSILON15 {
            // Ray is parallel to plane.  The ray may be in the polygon's plane.
            return None;
        }

        let t = (-plane.distance - normal.dot(origin)) / denominator;

        if t < 0. {
            return None;
        }

        return Some(origin + direction.multiply_by_scalar(t));
    }
    pub fn ray_ellipsoid(ray: &Ray, ellipsoid: Option<&Ellipsoid>) -> Option<Interval> {
        let ellipsoid = ellipsoid.unwrap_or(&Ellipsoid::WGS84);
        let inverse_radii = ellipsoid.one_over_radii;
        let q = inverse_radii.multiply_components(&ray.origin);
        let w = inverse_radii.multiply_components(&ray.direction);

        let q2 = q.magnitude_squared();
        let qw = q.dot(w);

        let mut difference;
        let mut w2;
        let mut product;
        let mut discriminant;
        let mut temp;

        if q2 > 1.0 {
            // Outside ellipsoid.
            if qw >= 0.0 {
                // Looking outward or tangent (0 intersections).
                return None;
            }

            // qw < 0.0.
            let qw2 = qw * qw;
            difference = q2 - 1.0; // Positively valued.
            w2 = w.magnitude_squared();
            product = w2 * difference;

            if qw2 < product {
                // Imaginary roots (0 intersections).
                return None;
            } else if qw2 > product {
                // Distinct roots (2 intersections).
                discriminant = qw * qw - product;
                temp = -qw + discriminant.sqrt(); // Avoid cancellation.
                let root0 = temp / w2;
                let root1 = difference / temp;
                if root0 < root1 {
                    return Some(Interval::new(root0, root1));
                }
                return Some(Interval::new(root1, root0));
            }
            // qw2 == product.  Repeated roots (2 intersections).
            let root = (difference / w2).sqrt();
            return Some(Interval::new(root, root));
        } else if q2 < 1.0 {
            // Inside ellipsoid (2 intersections).
            difference = q2 - 1.0; // Negatively valued.
            w2 = w.magnitude_squared();
            product = w2 * difference; // Negatively valued.

            discriminant = qw * qw - product;
            temp = -qw + (discriminant).sqrt(); // Positively valued.
            return Some(Interval::new(0.0, temp / w2));
        }
        // q2 == 1.0. On ellipsoid.
        if qw < 0.0 {
            // Looking inward.
            w2 = w.magnitude_squared();
            return Some(Interval::new(0.0, -qw / w2));
        }

        // qw >= 0.0.  Looking outward or tangent.
        return None;
    }
    pub fn grazing_altitude_location(ray: &Ray, ellipsoid: Option<&Ellipsoid>) -> Option<DVec3> {
        let ellipsoid = ellipsoid.unwrap_or(&Ellipsoid::WGS84);
        let position = ray.origin;
        let direction = ray.direction;

        if !position.equals(DVec3::ZERO) {
            let normal = ellipsoid.geodetic_surface_normal(&position).unwrap();
            if direction.dot(normal) >= 0.0 {
                // The location provided is the closest point in altitude
                return Some(position);
            }
        }

        let intersects = IntersectionTests::ray_ellipsoid(&ray, Some(ellipsoid)).is_some();

        // Compute the scaled direction vector.
        let f = ellipsoid.transform_position_to_scaled_space(&direction);

        // letructs a basis from the unit scaled direction vector. letruct its rotation and transpose.
        let first_axis = f.normalize();
        let reference = f.most_orthogonal_axis();
        let second_axis = (reference.cross(first_axis)).normalize();
        let third_axis = (first_axis.cross(second_axis)).normalize();
        let mut bb: [f64; 9] = [0.; 9];
        bb[0] = first_axis.x;
        bb[1] = first_axis.y;
        bb[2] = first_axis.z;
        bb[3] = second_axis.x;
        bb[4] = second_axis.y;
        bb[5] = second_axis.z;
        bb[6] = third_axis.x;
        bb[7] = third_axis.y;
        bb[8] = third_axis.z;
        let B = DMat3::from_raw_list(bb);
        let b_t = B.transpose();

        // Get the scaling matrix and its inverse.
        let d_i = DMat3::from_scale3(ellipsoid.radii);
        let D = DMat3::from_scale3(ellipsoid.one_over_radii);

        let mut cc: [f64; 9] = [0.; 9];
        cc[0] = 0.0;
        cc[1] = -direction.z;
        cc[2] = direction.y;
        cc[3] = direction.z;
        cc[4] = 0.0;
        cc[5] = -direction.x;
        cc[6] = -direction.y;
        cc[7] = direction.x;
        cc[8] = 0.0;
        let C = DMat3::from_raw_list(cc);

        let temp = b_t.mul_mat3(&D).mul_mat3(&C);
        let A = temp.mul_mat3(&d_i).mul_mat3(&B);
        let b = temp.mul_vec3(position);

        // Solve for the solutions to the expression in standard form:
        let solutions = quadratic_vector_expression(&A, &b.negate(), 0.0, 0.0, 1.0);

        let mut s;
        let mut altitude;
        let length = solutions.len();
        if length > 0 {
            let mut closest = DVec3::ZERO.clone();
            let mut maximum_value = NEG_INFINITY;
            for i in 0..length {
                s = d_i.mul_vec3(B.mul_vec3(solutions[i]));
                let v = (s.subtract(position)).normalize();
                let dot_product = v.dot(direction);

                if dot_product > maximum_value {
                    maximum_value = dot_product;
                    closest = s.clone();
                }
            }

            let mut surface_point = ellipsoid.cartesian_to_cartographic(&closest).unwrap();
            maximum_value = maximum_value.clamp(0.0, 1.0);
            altitude = closest.subtract(position).magnitude()
                * (1.0 - maximum_value * maximum_value).sqrt();
            altitude = if intersects { -altitude } else { altitude };

            surface_point.height = altitude;
            return Some(ellipsoid.cartographic_to_cartesian(&surface_point));
        }

        return None;
    }
}
fn add_with_cancellation_check(left: f64, right: f64, tolerance: f64) -> f64 {
    let difference = left + right;
    if left.signum() != right.signum()
        && (difference / left.abs().max(right.abs())).abs() < tolerance
    {
        return 0.0;
    }

    return difference;
}
fn quadratic_vector_expression(A: &DMat3, b: &DVec3, c: f64, x: f64, w: f64) -> Vec<DVec3> {
    let mut aa: [f64; 9] = [0.; 9];
    A.write_cols_to_slice(&mut aa);

    let x_squared = x * x;
    let w_squared = w * w;

    let l2 = (aa[DMat3::COLUMN1ROW1] - aa[DMat3::COLUMN2ROW2]) * w_squared;
    let l1 = w
        * (x * add_with_cancellation_check(
            aa[DMat3::COLUMN1ROW0],
            aa[DMat3::COLUMN0ROW1],
            EPSILON15,
        ) + b.y);
    let l0 = aa[DMat3::COLUMN0ROW0] * x_squared + aa[DMat3::COLUMN2ROW2] * w_squared + x * b.x + c;

    let r1 = w_squared
        * add_with_cancellation_check(aa[DMat3::COLUMN2ROW1], aa[DMat3::COLUMN1ROW2], EPSILON15);
    let r0 = w
        * (x * add_with_cancellation_check(
            aa[DMat3::COLUMN2ROW0],
            aa[DMat3::COLUMN0ROW2],
            EPSILON15,
        ) + b.z);

    let cosines;
    let mut solutions = vec![];
    if r0 == 0.0 && r1 == 0.0 {
        cosines = QuadraticRealPolynomial::compute_real_roots(l2, l1, l0).unwrap();
        if cosines.len() == 0 {
            return solutions;
        }

        let cosine0 = cosines[0];
        let sine0 = ((1.0 - cosine0 * cosine0).max(0.0)).sqrt();
        solutions.push(DVec3::new(x, w * cosine0, w * -sine0));
        solutions.push(DVec3::new(x, w * cosine0, w * sine0));

        if cosines.len() == 2 {
            let cosine1 = cosines[1];
            let sine1 = ((1.0 - cosine1 * cosine1).max(0.0)).sqrt();
            solutions.push(DVec3::new(x, w * cosine1, w * -sine1));
            solutions.push(DVec3::new(x, w * cosine1, w * sine1));
        }

        return solutions;
    }

    let r0_squared = r0 * r0;
    let r1_squared = r1 * r1;
    let l2_squared = l2 * l2;
    let r0r1 = r0 * r1;

    let c4 = l2_squared + r1_squared;
    let c3 = 2.0 * (l1 * l2 + r0r1);
    let c2 = 2.0 * l0 * l2 + l1 * l1 - r1_squared + r0_squared;
    let c1 = 2.0 * (l0 * l1 - r0r1);
    let c0 = l0 * l0 - r0_squared;

    if c4 == 0.0 && c3 == 0.0 && c2 == 0.0 && c1 == 0.0 {
        return solutions;
    }

    cosines = QuarticRealPolynomial::compute_real_roots(c4, c3, c2, c1, c0).unwrap();
    let length = cosines.len();
    if length == 0 {
        return solutions;
    }
    for i in 0..length {
        let cosine = cosines[i];
        let cosine_squared = cosine * cosine;
        let sine_squared = (1.0 - cosine_squared).max(0.0);
        let sine = sine_squared.sqrt();

        //let left = l2 * cosine_squared + l1 * cosine + l0;
        let left;
        if l2.signum() == l0.signum() {
            left = add_with_cancellation_check(l2 * cosine_squared + l0, l1 * cosine, EPSILON12);
        } else if l0.signum() == (l1 * cosine).signum() {
            left = add_with_cancellation_check(l2 * cosine_squared, l1 * cosine + l0, EPSILON12);
        } else {
            left = add_with_cancellation_check(l2 * cosine_squared + l1 * cosine, l0, EPSILON12);
        }

        let right = add_with_cancellation_check(r1 * cosine, r0, EPSILON15);
        let product = left * right;

        if product < 0.0 {
            solutions.push(DVec3::new(x, w * cosine, w * sine));
        } else if product > 0.0 {
            solutions.push(DVec3::new(x, w * cosine, w * -sine));
        } else if sine != 0.0 {
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
        let actual = IntersectionTests::ray_ellipsoid(&ray, None);
        let actual = actual.unwrap();
        assert!(actual.start == 0.0);
        assert!(actual.stop == Ellipsoid::WGS84.radii.x + origin.x);
    }
    #[test]
    fn ray_inside_pointing_out_intersection() {
        let origin = DVec3::new(20000.0, 0.0, 0.0);
        let direction = origin.normalize();
        let ray = Ray::new(origin, direction);
        let actual = IntersectionTests::ray_ellipsoid(&ray, None);
        let actual = actual.unwrap();
        assert!(actual.start == 0.0);
        assert!(actual.stop == Ellipsoid::WGS84.radii.x - origin.x);
    }
    #[test]
    fn tangent_intersections() {
        let ray = Ray::new(DVec3::UNIT_X, DVec3::UNIT_Z);
        let actual = IntersectionTests::ray_ellipsoid(&ray, Some(&Ellipsoid::UNIT_SPHERE));
        assert!(actual.is_none());
    }
    #[test]
    fn grazing_altitude_location_inside_ellipsoid() {
        let ellipsoid = Ellipsoid::UNIT_SPHERE;
        let ray = Ray::new(DVec3::new(0.5, 0.0, 0.0), DVec3::UNIT_Z);
        let actual = IntersectionTests::grazing_altitude_location(&ray, Some(&ellipsoid));
        assert!(actual.unwrap().eq(&ray.origin));
    }
}
