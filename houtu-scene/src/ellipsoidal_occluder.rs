use bevy::{math::DVec3, prelude::Resource};

use crate::{
    ellipsoid::{self, Ellipsoid},
    math::Cartesian3,
};
#[derive(Clone, Debug, Resource)]
pub struct EllipsoidalOccluder {
    pub ellipsoid: Ellipsoid,
    pub cameraPosition: DVec3,
    pub cameraPositionInScaledSpace: DVec3,
    pub distanceToLimbInScaledSpaceSquared: f64,
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
            cameraPosition: DVec3::ZERO,
            cameraPositionInScaledSpace: DVec3::ZERO,
            distanceToLimbInScaledSpaceSquared: 0.0,
        }
    }
    pub fn set_camera_position(&mut self, camera_position: DVec3) {
        // See https://cesium.com/blog/2013/04/25/Horizon-culling/
        let ellipsoid = &self.ellipsoid;
        let cv = ellipsoid.transformPositionToScaledSpace(&camera_position);
        self.cameraPositionInScaledSpace = cv.clone();
        let vhMagnitudeSquared = cv.magnitude_squared() - 1.0;
        self.cameraPosition = camera_position.clone();
        self.cameraPositionInScaledSpace = cv;
        self.distanceToLimbInScaledSpaceSquared = vhMagnitudeSquared;
    }
    pub fn isPointVisible(&self, occludee: DVec3) -> bool {
        let ellipsoid = &self.ellipsoid;
        let occludeeScaledSpacePosition = ellipsoid.transformPositionToScaledSpace(&occludee);
        return self.isScaledSpacePointVisible(
            &occludeeScaledSpacePosition,
            &self.cameraPositionInScaledSpace,
            self.distanceToLimbInScaledSpaceSquared,
        );
    }
    pub fn isScaledSpacePointVisible(
        &self,
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
    pub fn isScaledSpacePointVisiblePossiblyUnderEllipsoid(
        &self,
        occludeeScaledSpacePosition: &DVec3,
        minimumHeight: Option<f64>,
    ) -> bool {
        let ellipsoid = self.ellipsoid;
        let vhMagnitudeSquared;
        let mut cv;
        if let Some(minimumHeight) = minimumHeight {
            if minimumHeight < 0.0 && ellipsoid.minimumRadius > -minimumHeight {
                // This code is similar to the cameraPosition setter, but unrolled for performance because it will be called a lot.
                cv = DVec3::ZERO;
                cv.x = self.cameraPosition.x / (ellipsoid.radii.x + minimumHeight);
                cv.y = self.cameraPosition.y / (ellipsoid.radii.y + minimumHeight);
                cv.z = self.cameraPosition.z / (ellipsoid.radii.z + minimumHeight);
                vhMagnitudeSquared = cv.x * cv.x + cv.y * cv.y + cv.z * cv.z - 1.0;
            } else {
                cv = self.cameraPositionInScaledSpace;
                vhMagnitudeSquared = self.distanceToLimbInScaledSpaceSquared;
            }
        } else {
            cv = self.cameraPositionInScaledSpace;
            vhMagnitudeSquared = self.distanceToLimbInScaledSpaceSquared;
        }
        return self.isScaledSpacePointVisible(
            occludeeScaledSpacePosition,
            &cv,
            vhMagnitudeSquared,
        );
    }
    pub fn computeHorizonCullingPointPossiblyUnderEllipsoid(
        &self,
        directionToPoint: &DVec3,
        positions: &Vec<DVec3>,
        minimumHeight: f64,
    ) -> Option<DVec3> {
        let possiblyShrunkEllipsoid =
            getPossiblyShrunkEllipsoid(&self.ellipsoid, Some(minimumHeight));
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
pub fn getPossiblyShrunkEllipsoid(ellipsoid: &Ellipsoid, minimumHeight: Option<f64>) -> Ellipsoid {
    if let Some(minimumHeight) = minimumHeight {
        if minimumHeight < 0.0 && ellipsoid.minimumRadius > -minimumHeight {
            let ellipsoidShrunkRadii = DVec3::from_elements(
                ellipsoid.radii.x + minimumHeight,
                ellipsoid.radii.y + minimumHeight,
                ellipsoid.radii.z + minimumHeight,
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
}
