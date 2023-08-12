use bevy::{math::DVec3, prelude::Resource};

use crate::{
    ellipsoid::{self, Ellipsoid},
    math::Cartesian3,
};
#[derive(Clone, Debug, Resource)]
pub struct EllipsoidalOccluder {
    pub ellipsoid: Ellipsoid,
    camera_position: DVec3,
    camera_position_in_scaled_space: DVec3,
    distance_to_limb_in_scaled_space_squared: f64,
}
impl Default for EllipsoidalOccluder {
    fn default() -> Self {
        Self::new(&Ellipsoid::WGS84)
    }
}
impl EllipsoidalOccluder {
    pub fn new(ellipsoid: &Ellipsoid) -> Self {
        EllipsoidalOccluder {
            ellipsoid: ellipsoid.clone(),
            camera_position: DVec3::ZERO,
            camera_position_in_scaled_space: DVec3::ZERO,
            distance_to_limb_in_scaled_space_squared: 0.0,
        }
    }
    pub fn get_camera_position(&self) -> &DVec3 {
        return &self.camera_position;
    }
    pub fn set_camera_position(&mut self, camera_position: DVec3) {
        // See https://cesium.com/blog/2013/04/25/Horizon-culling/
        let ellipsoid = &self.ellipsoid;
        let cv = ellipsoid.transform_position_to_scaled_space(&camera_position);
        self.camera_position_in_scaled_space = cv.clone();
        let vh_magnitude_squared = cv.magnitude_squared() - 1.0;
        self.camera_position = camera_position.clone();
        self.camera_position_in_scaled_space = cv;
        self.distance_to_limb_in_scaled_space_squared = vh_magnitude_squared;
    }
    pub fn is_point_visible(&self, occludee: DVec3) -> bool {
        let ellipsoid = &self.ellipsoid;
        let occludee_scaled_space_position =
            ellipsoid.transform_position_to_scaled_space(&occludee);
        return Self::_is_scaled_space_point_visible(
            &occludee_scaled_space_position,
            &self.camera_position_in_scaled_space,
            self.distance_to_limb_in_scaled_space_squared,
        );
    }
    fn _is_scaled_space_point_visible(
        occludee_scaled_space_position: &DVec3,
        camera_position_in_scaled_space: &DVec3,
        distance_to_limb_in_scaled_space_squared: f64,
    ) -> bool {
        // See https://cesium.com/blog/2013/04/25/Horizon-culling/
        let cv = camera_position_in_scaled_space;
        let vh_magnitude_squared = distance_to_limb_in_scaled_space_squared;
        let vt = occludee_scaled_space_position.subtract(*cv);
        let vt_dot_vc = -1. * vt.dot(*cv);
        // If vh_magnitude_squared < 0 then we are below the surface of the ellipsoid and
        // in self case, set the culling plane to be on V.
        let is_occluded = {
            if vh_magnitude_squared < 0. {
                vt_dot_vc > 0.
            } else {
                vt_dot_vc > vh_magnitude_squared
                    && (vt_dot_vc * vt_dot_vc) / vt.magnitude_squared() > vh_magnitude_squared
            }
        };
        return !is_occluded;
    }
    pub fn is_scaled_space_point_visible(&self, occludee_scaled_space_position: &DVec3) -> bool {
        return Self::_is_scaled_space_point_visible(
            occludee_scaled_space_position,
            &self.camera_position_in_scaled_space,
            self.distance_to_limb_in_scaled_space_squared,
        );
    }
    pub fn is_scaled_space_point_visible_possibly_under_ellipsoid(
        &self,
        occludee_scaled_space_position: &DVec3,
        minimum_height: Option<f64>,
    ) -> bool {
        let ellipsoid = self.ellipsoid;
        let vh_magnitude_squared;
        let mut cv;
        if let Some(minimum_height) = minimum_height {
            if minimum_height < 0.0 && ellipsoid.minimum_radius > -minimum_height {
                // This code is similar to the camera_position setter, but unrolled for performance because it will be called a lot.
                cv = DVec3::ZERO;
                cv.x = self.camera_position.x / (ellipsoid.radii.x + minimum_height);
                cv.y = self.camera_position.y / (ellipsoid.radii.y + minimum_height);
                cv.z = self.camera_position.z / (ellipsoid.radii.z + minimum_height);
                vh_magnitude_squared = cv.x * cv.x + cv.y * cv.y + cv.z * cv.z - 1.0;
            } else {
                cv = self.camera_position_in_scaled_space;
                vh_magnitude_squared = self.distance_to_limb_in_scaled_space_squared;
            }
        } else {
            cv = self.camera_position_in_scaled_space;
            vh_magnitude_squared = self.distance_to_limb_in_scaled_space_squared;
        }
        return Self::_is_scaled_space_point_visible(
            occludee_scaled_space_position,
            &cv,
            vh_magnitude_squared,
        );
    }
    pub fn compute_horizon_culling_point_possibly_under_ellipsoid(
        &self,
        direction_to_point: &DVec3,
        positions: &Vec<DVec3>,
        minimum_height: f64,
    ) -> Option<DVec3> {
        let possibly_shrunk_ellipsoid =
            get_possibly_shrunk_ellipsoid(&self.ellipsoid, Some(minimum_height));
        return compute_horizon_culling_point_from_positions(
            &possibly_shrunk_ellipsoid,
            direction_to_point,
            positions,
        );
    }
    pub fn compute_horizon_culling_point(
        &self,
        direction_to_point: &DVec3,
        positions: &Vec<DVec3>,
    ) -> Option<DVec3> {
        return compute_horizon_culling_point_from_positions(
            &self.ellipsoid,
            direction_to_point,
            positions,
        );
    }
}
pub fn get_possibly_shrunk_ellipsoid(
    ellipsoid: &Ellipsoid,
    minimum_height: Option<f64>,
) -> Ellipsoid {
    if let Some(minimum_height) = minimum_height {
        if minimum_height < 0.0 && ellipsoid.minimum_radius > -minimum_height {
            let ellipsoid_shrunk_radii = DVec3::from_elements(
                ellipsoid.radii.x + minimum_height,
                ellipsoid.radii.y + minimum_height,
                ellipsoid.radii.z + minimum_height,
            );
            return Ellipsoid::from_vec3(ellipsoid_shrunk_radii);
        } else {
            return ellipsoid.clone();
        }
    } else {
        return ellipsoid.clone();
    }
}
pub fn compute_horizon_culling_point_from_positions(
    ellipsoid: &Ellipsoid,
    direction_to_point: &DVec3,
    positions: &Vec<DVec3>,
) -> Option<DVec3> {
    let scaled_space_direction_to_point =
        compute_scaled_space_direction_to_point(ellipsoid, direction_to_point);
    let mut result_magnitude: f64 = 0.0;
    for i in 0..positions.len() {
        let position = positions[i];
        let candidate_magnitude =
            compute_magnitude(ellipsoid, position, scaled_space_direction_to_point);
        if candidate_magnitude < 0.0 {
            // all points should face the same direction, but self one doesn't, so return undefined
            return None;
        }
        result_magnitude = result_magnitude.max(candidate_magnitude);
    }

    return magnitude_to_point(scaled_space_direction_to_point, result_magnitude);
}
pub fn compute_scaled_space_direction_to_point(
    ellipsoid: &Ellipsoid,
    direction_to_point: &DVec3,
) -> DVec3 {
    if direction_to_point == &DVec3::ZERO {
        return direction_to_point.clone();
    }
    let direction_to_point_scratch =
        ellipsoid.transform_position_to_scaled_space(&direction_to_point);
    return direction_to_point_scratch.normalize();
}
pub fn compute_magnitude(
    ellipsoid: &Ellipsoid,
    position: DVec3,
    scaled_space_direction_to_point: DVec3,
) -> f64 {
    let scaled_space_position = ellipsoid.transform_position_to_scaled_space(&position);
    let mut magnitude_squared = scaled_space_position.magnitude_squared();
    let mut magnitude = magnitude_squared.sqrt();
    let direction = scaled_space_position / magnitude;
    // For the purpose of self computation, points below the ellipsoid are consider to be on it instead.
    magnitude_squared = magnitude_squared.max(1.0);
    magnitude = magnitude.max(1.0);

    let cos_alpha = direction.dot(scaled_space_direction_to_point);
    let sin_alpha = direction.cross(scaled_space_direction_to_point).magnitude();
    let cos_beta = 1.0 / magnitude;
    let sin_beta = (magnitude_squared - 1.0).sqrt() * cos_beta;

    return 1.0 / (cos_alpha * cos_beta - sin_alpha * sin_beta);
}
pub fn magnitude_to_point(
    scaled_space_direction_to_point: DVec3,
    result_magnitude: f64,
) -> Option<DVec3> {
    // The horizon culling point is undefined if there were no positions from which to compute it,
    // the direction_to_point is pointing opposite all of the positions,  or if we computed NaN or infinity.
    if (result_magnitude <= 0.0
        || result_magnitude == 1.0 / 0.0
        || result_magnitude != result_magnitude)
    {
        return None;
    }

    return Some(scaled_space_direction_to_point.multiply_by_scalar(result_magnitude));
}
#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use crate::math::{equals_epsilon, EPSILON14};

    use super::*;

    #[test]
    fn compute_horizon_culling_point() {
        let ellipsoid = Ellipsoid::new(12345.0, 4567.0, 8910.0);
        let ellipsoidal_occluder = EllipsoidalOccluder::new(&ellipsoid);
        let positions = vec![DVec3::new(12345.0, 0.0, 0.0)];
        let direction_to_point = DVec3::new(1.0, 0.0, 0.0);

        let result = ellipsoidal_occluder
            .compute_horizon_culling_point(&direction_to_point, &positions)
            .unwrap();
        assert!(equals_epsilon(result.x, 1.0, Some(EPSILON14), None));
        assert!(equals_epsilon(result.y, 0.0, Some(EPSILON14), None));
        assert!(equals_epsilon(result.z, 0.0, Some(EPSILON14), None));
    }
    #[test]
    fn compute_horizon_culling_point_none() {
        let ellipsoid = Ellipsoid::new(12345.0, 4567.0, 8910.0);
        let ellipsoidal_occluder = EllipsoidalOccluder::new(&ellipsoid);
        let positions = vec![DVec3::new(0.0, 4567.0, 0.0)];
        let direction_to_point = DVec3::new(1.0, 0.0, 0.0);

        let result =
            ellipsoidal_occluder.compute_horizon_culling_point(&direction_to_point, &positions);
        assert!(result.is_none());
    }
    #[test]
    fn compute_horizon_culling_point_none_also() {
        let ellipsoid = Ellipsoid::new(1.0, 1.0, 1.0);
        let ellipsoidal_occluder = EllipsoidalOccluder::new(&ellipsoid);
        let positions = vec![DVec3::new(2.0, 0.0, 0.0), DVec3::new(-1.0, 0.0, 0.0)];
        let direction_to_point = DVec3::new(1.0, 0.0, 0.0);

        let result =
            ellipsoidal_occluder.compute_horizon_culling_point(&direction_to_point, &positions);
        assert!(result.is_none());
    }
    #[test]
    fn is_scaled_space_point_visible() {
        let camera_position = DVec3::new(0.0, 0.0, 2.5);
        let ellipsoid = Ellipsoid::new(1.0, 1.0, 0.9);
        let mut occluder = EllipsoidalOccluder::new(&ellipsoid);
        occluder.set_camera_position(camera_position);
        let point = DVec3::new(0.0, -3.0, -3.0);
        let scaled_space_point = ellipsoid.transform_position_to_scaled_space(&point);
        assert!(occluder.is_scaled_space_point_visible(&scaled_space_point));
    }
    #[test]
    fn is_scaled_space_point_visible_possibly_under_ellipsoid() {
        // Tests points that are halfway inside a unit sphere:
        // 1) on the diagonal
        // 2) on the +y-axis
        // The camera is on the +z-axis so it will be able to see the diagonal point but not the +y-axis point.
        let camera_position = DVec3::new(0.0, 0.0, 1.0);
        let ellipsoid = Ellipsoid::new(1.0, 1.0, 1.0);
        let mut occluder = EllipsoidalOccluder::new(&ellipsoid);
        occluder.set_camera_position(camera_position);
        let height = -0.5;
        let mut direction = DVec3::new(1.0, 1.0, 1.0).normalize();
        let mut point = direction.multiply_by_scalar(0.5);
        let scaled_space_point = occluder
            .compute_horizon_culling_point(&point, &vec![point])
            .unwrap();
        let scaled_space_point_shrunk = occluder
            .compute_horizon_culling_point_possibly_under_ellipsoid(&point, &vec![point], height)
            .unwrap();
        assert!(occluder.is_scaled_space_point_visible(&scaled_space_point) == false);
        assert!(
            occluder.is_scaled_space_point_visible_possibly_under_ellipsoid(
                &scaled_space_point_shrunk,
                Some(height)
            )
        );
        direction = DVec3::new(0.0, 1.0, 0.0);
        point = direction * 0.5;
        let scaled_space_point = occluder
            .compute_horizon_culling_point(&point, &vec![point])
            .unwrap();
        let scaled_space_point_shrunk = occluder
            .compute_horizon_culling_point_possibly_under_ellipsoid(&point, &vec![point], height)
            .unwrap();
        assert!(occluder.is_scaled_space_point_visible(&scaled_space_point) == false);
        assert!(
            occluder.is_scaled_space_point_visible_possibly_under_ellipsoid(
                &scaled_space_point_shrunk,
                Some(height)
            ) == false
        );
    }
    #[test]
    fn reports_not_visible_when_point_is_over_horizon() {
        let ellipsoid = Ellipsoid::WGS84;
        let mut occlude = EllipsoidalOccluder::new(&ellipsoid);
        occlude.set_camera_position(DVec3::new(7000000.0, 0.0, 0.0));
        let point = DVec3::new(4510635.0, 4510635.0, 0.0);
        assert!(occlude.is_point_visible(point) == false);
    }
}
