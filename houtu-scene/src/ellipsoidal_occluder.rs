use bevy::{math::DVec3, prelude::Resource};

use crate::{
    ellipsoid::{self, Ellipsoid},
    math::Cartesian3,
};
#[derive(Clone, Debug, Resource)]
pub struct EllipsoidalOccluder {
    pub ellipsoid: Ellipsoid,
    _cameraPosition: DVec3,
    _cameraPositionInScaledSpace: DVec3,
    _distanceToLimbInScaledSpaceSquared: f64,
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
            _cameraPosition: DVec3::ZERO,
            _cameraPositionInScaledSpace: DVec3::ZERO,
            _distanceToLimbInScaledSpaceSquared: 0.0,
        }
    }
    pub fn get_camera_position(&self) -> &DVec3 {
        return &self._cameraPosition;
    }
    pub fn set_camera_position(&mut self, camera_position: DVec3) {
        // See https://cesium.com/blog/2013/04/25/Horizon-culling/
        let ellipsoid = &self.ellipsoid;
        let cv = ellipsoid.transformPositionToScaledSpace(&camera_position);
        self._cameraPositionInScaledSpace = cv.clone();
        let vhMagnitudeSquared = cv.magnitude_squared() - 1.0;
        self._cameraPosition = camera_position.clone();
        self._cameraPositionInScaledSpace = cv;
        self._distanceToLimbInScaledSpaceSquared = vhMagnitudeSquared;
    }
    pub fn isPointVisible(&self, occludee: DVec3) -> bool {
        let ellipsoid = &self.ellipsoid;
        let occludeeScaledSpacePosition = ellipsoid.transformPositionToScaledSpace(&occludee);
        return Self::_isScaledSpacePointVisible(
            &occludeeScaledSpacePosition,
            &self._cameraPositionInScaledSpace,
            self._distanceToLimbInScaledSpaceSquared,
        );
    }
    fn _isScaledSpacePointVisible(
        occludeeScaledSpacePosition: &DVec3,
        cameraPositionInScaledSpace: &DVec3,
        distanceToLimbInScaledSpaceSquared: f64,
    ) -> bool {
        // See https://cesium.com/blog/2013/04/25/Horizon-culling/
        let cv = cameraPositionInScaledSpace;
        let vhMagnitudeSquared = distanceToLimbInScaledSpaceSquared;
        let vt = occludeeScaledSpacePosition.subtract(*cv);
        let vtDotVc = -1. * vt.dot(*cv);
        // If vhMagnitudeSquared < 0 then we are below the surface of the ellipsoid and
        // in self case, set the culling plane to be on V.
        let isOccluded = {
            if vhMagnitudeSquared < 0. {
                vtDotVc > 0.
            } else {
                vtDotVc > vhMagnitudeSquared
                    && (vtDotVc * vtDotVc) / vt.magnitude_squared() > vhMagnitudeSquared
            }
        };
        return !isOccluded;
    }
    pub fn isScaledSpacePointVisible(&self, occludeeScaledSpacePosition: &DVec3) -> bool {
        return Self::_isScaledSpacePointVisible(
            occludeeScaledSpacePosition,
            &self._cameraPositionInScaledSpace,
            self._distanceToLimbInScaledSpaceSquared,
        );
    }
    pub fn isScaledSpacePointVisiblePossiblyUnderEllipsoid(
        &self,
        occludeeScaledSpacePosition: &DVec3,
        minimum_height: Option<f64>,
    ) -> bool {
        let ellipsoid = self.ellipsoid;
        let vhMagnitudeSquared;
        let mut cv;
        if let Some(minimum_height) = minimum_height {
            if minimum_height < 0.0 && ellipsoid.minimumRadius > -minimum_height {
                // This code is similar to the camera_position setter, but unrolled for performance because it will be called a lot.
                cv = DVec3::ZERO;
                cv.x = self._cameraPosition.x / (ellipsoid.radii.x + minimum_height);
                cv.y = self._cameraPosition.y / (ellipsoid.radii.y + minimum_height);
                cv.z = self._cameraPosition.z / (ellipsoid.radii.z + minimum_height);
                vhMagnitudeSquared = cv.x * cv.x + cv.y * cv.y + cv.z * cv.z - 1.0;
            } else {
                cv = self._cameraPositionInScaledSpace;
                vhMagnitudeSquared = self._distanceToLimbInScaledSpaceSquared;
            }
        } else {
            cv = self._cameraPositionInScaledSpace;
            vhMagnitudeSquared = self._distanceToLimbInScaledSpaceSquared;
        }
        return Self::_isScaledSpacePointVisible(
            occludeeScaledSpacePosition,
            &cv,
            vhMagnitudeSquared,
        );
    }
    pub fn computeHorizonCullingPointPossiblyUnderEllipsoid(
        &self,
        directionToPoint: &DVec3,
        positions: &Vec<DVec3>,
        minimum_height: f64,
    ) -> Option<DVec3> {
        let possiblyShrunkEllipsoid =
            getPossiblyShrunkEllipsoid(&self.ellipsoid, Some(minimum_height));
        return computeHorizonCullingPointFromPositions(
            &possiblyShrunkEllipsoid,
            directionToPoint,
            positions,
        );
    }
    pub fn computeHorizonCullingPoint(
        &self,
        directionToPoint: &DVec3,
        positions: &Vec<DVec3>,
    ) -> Option<DVec3> {
        return computeHorizonCullingPointFromPositions(
            &self.ellipsoid,
            directionToPoint,
            positions,
        );
    }
}
pub fn getPossiblyShrunkEllipsoid(ellipsoid: &Ellipsoid, minimum_height: Option<f64>) -> Ellipsoid {
    if let Some(minimum_height) = minimum_height {
        if minimum_height < 0.0 && ellipsoid.minimumRadius > -minimum_height {
            let ellipsoidShrunkRadii = DVec3::from_elements(
                ellipsoid.radii.x + minimum_height,
                ellipsoid.radii.y + minimum_height,
                ellipsoid.radii.z + minimum_height,
            );
            return Ellipsoid::from_vec3(ellipsoidShrunkRadii);
        } else {
            return ellipsoid.clone();
        }
    } else {
        return ellipsoid.clone();
    }
}
pub fn computeHorizonCullingPointFromPositions(
    ellipsoid: &Ellipsoid,
    directionToPoint: &DVec3,
    positions: &Vec<DVec3>,
) -> Option<DVec3> {
    let scaledSpaceDirectionToPoint =
        computeScaledSpaceDirectionToPoint(ellipsoid, directionToPoint);
    let mut resultMagnitude: f64 = 0.0;
    for i in 0..positions.len() {
        let position = positions[i];
        let candidateMagnitude = computeMagnitude(ellipsoid, position, scaledSpaceDirectionToPoint);
        if candidateMagnitude < 0.0 {
            // all points should face the same direction, but self one doesn't, so return undefined
            return None;
        }
        resultMagnitude = resultMagnitude.max(candidateMagnitude);
    }

    return magnitudeToPoint(scaledSpaceDirectionToPoint, resultMagnitude);
}
pub fn computeScaledSpaceDirectionToPoint(
    ellipsoid: &Ellipsoid,
    directionToPoint: &DVec3,
) -> DVec3 {
    if directionToPoint == &DVec3::ZERO {
        return directionToPoint.clone();
    }
    let directionToPointScratch = ellipsoid.transformPositionToScaledSpace(&directionToPoint);
    return directionToPointScratch.normalize();
}
pub fn computeMagnitude(
    ellipsoid: &Ellipsoid,
    position: DVec3,
    scaledSpaceDirectionToPoint: DVec3,
) -> f64 {
    let scaledSpacePosition = ellipsoid.transformPositionToScaledSpace(&position);
    let mut magnitudeSquared = scaledSpacePosition.magnitude_squared();
    let mut magnitude = magnitudeSquared.sqrt();
    let direction = scaledSpacePosition / magnitude;
    // For the purpose of self computation, points below the ellipsoid are consider to be on it instead.
    magnitudeSquared = magnitudeSquared.max(1.0);
    magnitude = magnitude.max(1.0);

    let cosAlpha = direction.dot(scaledSpaceDirectionToPoint);
    let sinAlpha = direction.cross(scaledSpaceDirectionToPoint).magnitude();
    let cosBeta = 1.0 / magnitude;
    let sinBeta = (magnitudeSquared - 1.0).sqrt() * cosBeta;

    return 1.0 / (cosAlpha * cosBeta - sinAlpha * sinBeta);
}
pub fn magnitudeToPoint(scaledSpaceDirectionToPoint: DVec3, resultMagnitude: f64) -> Option<DVec3> {
    // The horizon culling point is undefined if there were no positions from which to compute it,
    // the directionToPoint is pointing opposite all of the positions,  or if we computed NaN or infinity.
    if (resultMagnitude <= 0.0
        || resultMagnitude == 1.0 / 0.0
        || resultMagnitude != resultMagnitude)
    {
        return None;
    }

    return Some(scaledSpaceDirectionToPoint.multiply_by_scalar(resultMagnitude));
}
#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use crate::math::{equals_epsilon, EPSILON14};

    use super::*;

    #[test]
    fn computeHorizonCullingPoint() {
        let ellipsoid = Ellipsoid::new(12345.0, 4567.0, 8910.0);
        let ellipsoidalOccluder = EllipsoidalOccluder::new(&ellipsoid);
        let positions = vec![DVec3::new(12345.0, 0.0, 0.0)];
        let directionToPoint = DVec3::new(1.0, 0.0, 0.0);

        let result = ellipsoidalOccluder
            .computeHorizonCullingPoint(&directionToPoint, &positions)
            .unwrap();
        assert!(equals_epsilon(result.x, 1.0, Some(EPSILON14), None));
        assert!(equals_epsilon(result.y, 0.0, Some(EPSILON14), None));
        assert!(equals_epsilon(result.z, 0.0, Some(EPSILON14), None));
    }
    #[test]
    fn computeHorizonCullingPointNone() {
        let ellipsoid = Ellipsoid::new(12345.0, 4567.0, 8910.0);
        let ellipsoidalOccluder = EllipsoidalOccluder::new(&ellipsoid);
        let positions = vec![DVec3::new(0.0, 4567.0, 0.0)];
        let directionToPoint = DVec3::new(1.0, 0.0, 0.0);

        let result = ellipsoidalOccluder.computeHorizonCullingPoint(&directionToPoint, &positions);
        assert!(result.is_none());
    }
    #[test]
    fn computeHorizonCullingPointNoneAlso() {
        let ellipsoid = Ellipsoid::new(1.0, 1.0, 1.0);
        let ellipsoidalOccluder = EllipsoidalOccluder::new(&ellipsoid);
        let positions = vec![DVec3::new(2.0, 0.0, 0.0), DVec3::new(-1.0, 0.0, 0.0)];
        let directionToPoint = DVec3::new(1.0, 0.0, 0.0);

        let result = ellipsoidalOccluder.computeHorizonCullingPoint(&directionToPoint, &positions);
        assert!(result.is_none());
    }
    #[test]
    fn isScaledSpacePointVisible() {
        let camera_position = DVec3::new(0.0, 0.0, 2.5);
        let ellipsoid = Ellipsoid::new(1.0, 1.0, 0.9);
        let mut occluder = EllipsoidalOccluder::new(&ellipsoid);
        occluder.set_camera_position(camera_position);
        let point = DVec3::new(0.0, -3.0, -3.0);
        let scaled_space_point = ellipsoid.transformPositionToScaledSpace(&point);
        assert!(occluder.isScaledSpacePointVisible(&scaled_space_point));
    }
    #[test]
    fn isScaledSpacePointVisiblePossiblyUnderEllipsoid() {
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
            .computeHorizonCullingPoint(&point, &vec![point])
            .unwrap();
        let scaled_space_point_shrunk = occluder
            .computeHorizonCullingPointPossiblyUnderEllipsoid(&point, &vec![point], height)
            .unwrap();
        assert!(occluder.isScaledSpacePointVisible(&scaled_space_point) == false);
        assert!(occluder.isScaledSpacePointVisiblePossiblyUnderEllipsoid(
            &scaled_space_point_shrunk,
            Some(height)
        ));
        direction = DVec3::new(0.0, 1.0, 0.0);
        point = direction * 0.5;
        let scaled_space_point = occluder
            .computeHorizonCullingPoint(&point, &vec![point])
            .unwrap();
        let scaled_space_point_shrunk = occluder
            .computeHorizonCullingPointPossiblyUnderEllipsoid(&point, &vec![point], height)
            .unwrap();
        assert!(occluder.isScaledSpacePointVisible(&scaled_space_point) == false);
        assert!(
            occluder.isScaledSpacePointVisiblePossiblyUnderEllipsoid(
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
        assert!(occlude.isPointVisible(point) == false);
    }
}
