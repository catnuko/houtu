use bevy::math::{DVec2, DVec3};

use crate::{ellipsoid::Ellipsoid, geometry::Ray, math::*};

use super::{rayPlane, AxisAlignedBoundingBox, Plane};

#[derive(Clone, Debug, PartialEq)]
pub struct EllipsoidTangentPlane {
    pub plane: Plane,
    pub origin: DVec3,
    pub xAxis: DVec3,
    pub yAxis: DVec3,
    pub zAxis: DVec3,
    pub normal: DVec3,
    pub ellipsoid: Ellipsoid,
}
impl EllipsoidTangentPlane {
    pub fn new(origin: DVec3, ellipsoid: Option<Ellipsoid>) -> Self {
        let ellipsoid = ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let eastNorthUp = eastNorthUpToFixedFrame(origin, Some(ellipsoid));
        let xAxis = DVec3::from_cartesian4(eastNorthUp.col(0));
        let yAxis = DVec3::from_cartesian4(eastNorthUp.col(1));
        let zAxis = DVec3::from_cartesian4(eastNorthUp.col(2));
        let normal = zAxis;
        let plane = Plane::fromPointNormal(origin, normal);
        return Self {
            plane,
            origin,
            xAxis,
            yAxis,
            zAxis,
            normal,
            ellipsoid,
        };
    }
    pub fn fromPoints(cartesians: Vec<DVec3>, ellipsoid: Option<Ellipsoid>) -> Self {
        let aabb = AxisAlignedBoundingBox::fromPoints(cartesians);
        return EllipsoidTangentPlane::new(aabb.center, ellipsoid);
    }
    pub fn projectPointToNearestOnPlane(&self, cartesian: DVec3) -> DVec2 {
        let mut ray = Ray::default();
        ray.origin = cartesian;
        ray.direction = self.normal.clone();
        let mut intersectionPoint: DVec3 = DVec3::ZERO;
        let mut intersectionPointOption = rayPlane(ray, self.plane);
        if intersectionPointOption.is_none() {
            ray.direction = ray.direction.negate();
            intersectionPoint = rayPlane(ray, self.plane).unwrap();
        }
        let v = intersectionPoint - self.origin;
        let x = v.dot(self.xAxis);
        let y = v.dot(self.yAxis);
        return DVec2::new(x, y);
    }
    pub fn projectPointsToNearestOnPlane(&self, cartesians: Vec<DVec3>) -> Vec<DVec2> {
        let mut result: Vec<DVec2> = Vec::new();
        for cartesian in cartesians {
            result.push(self.projectPointToNearestOnPlane(cartesian));
        }
        return result;
    }
    pub fn projectPointOntoEllipsoid(&self, cartesian: DVec2) -> DVec3 {
        let ellipsoid = self.ellipsoid;
        let origin = self.origin;
        let xAxis = self.xAxis;
        let yAxis = self.yAxis;
        let mut tmp = DVec3::default();

        tmp = xAxis.multiply_by_scalar(cartesian.x);
        let mut result = origin + tmp;
        tmp = yAxis.multiply_by_scalar(cartesian.y);
        result = result + tmp;
        return ellipsoid.scaleToGeocentricSurface(&result);
    }
    pub fn projectPointsOntoEllipsoid(&self, cartesians: Vec<DVec2>) -> Vec<DVec3> {
        let mut result: Vec<DVec3> = Vec::new();
        for cartesian in cartesians {
            result.push(self.projectPointOntoEllipsoid(cartesian));
        }
        return result;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constructor() {
        let origin = DVec3::new(0.0, 0.0, 0.0);
        let plane = EllipsoidTangentPlane::new(origin, None);
        assert!(plane.ellipsoid.eq(&Ellipsoid::WGS84));
        assert!(plane.origin == origin);
    }
}
