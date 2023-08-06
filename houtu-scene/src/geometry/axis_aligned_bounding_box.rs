use bevy::math::DVec3;

use crate::math::*;

use super::{Intersect, Plane};

#[derive(Debug, Clone, Copy, Default)]
pub struct AxisAlignedBoundingBox {
    pub minimum: DVec3,
    pub maximum: DVec3,
    pub center: DVec3,
}
impl AxisAlignedBoundingBox {
    pub fn new(minimum: DVec3, maximum: DVec3, center: DVec3) -> Self {
        Self {
            minimum,
            maximum,
            center,
        }
    }
    pub fn fromCorners(minimum: DVec3, maximum: DVec3) -> Self {
        let mut result = Self::default();
        result.minimum = minimum;
        result.maximum = maximum;
        result.center = minimum.midpoint(maximum);
        return result;
    }
    pub fn fromPoints(positions: Vec<DVec3>) -> Self {
        let mut result = Self::default();
        let mut minimum_x = positions[0].x;
        let mut minimum_y = positions[0].y;
        let mut minimumZ = positions[0].z;

        let mut maximum_x = positions[0].x;
        let mut maximum_y = positions[0].y;
        let mut maximumZ = positions[0].z;

        let length = positions.len();
        for i in 1..length {
            let p = positions[i];
            let x = p.x;
            let y = p.y;
            let z = p.z;

            minimum_x = x.min(minimum_x);
            maximum_x = x.max(maximum_x);
            minimum_y = y.min(minimum_y);
            maximum_y = y.max(maximum_y);
            minimumZ = z.min(minimumZ);
            maximumZ = z.max(maximumZ);
        }

        let mut minimum = result.minimum;
        minimum.x = minimum_x;
        minimum.y = minimum_y;
        minimum.z = minimumZ;

        let mut maximum = result.maximum;
        maximum.x = maximum_x;
        maximum.y = maximum_y;
        maximum.z = maximumZ;

        result.center = minimum.midpoint(maximum);
        return result;
    }
    pub fn equals(&self, right: &Self) -> bool {
        return self.minimum == right.minimum
            && self.maximum == right.maximum
            && self.center == right.center;
    }
    pub fn intersectPlane(&self, plane: Plane) -> Intersect {
        let intersectScratch = self.maximum - self.minimum;
        let h = intersectScratch.multiply_by_scalar(0.5); //The positive half diagonal
        let normal = plane.normal;
        let e = h.x * normal.x.abs() + h.y * normal.y.abs() + h.z * normal.z.abs();
        let s = self.center.dot(normal) + plane.distance; //signed distance from center

        if s - e > 0. {
            return Intersect::INSIDE;
        }

        if s + e < 0. {
            //Not in front because normals point inward
            return Intersect::OUTSIDE;
        }

        return Intersect::INTERSECTING;
    }
}
