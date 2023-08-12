use bevy::math::{DVec2, DVec3};

use crate::{ellipsoid::Ellipsoid, geometry::Ray, math::*};

use super::{rayPlane, AxisAlignedBoundingBox, Plane};

#[derive(Clone, Debug, PartialEq)]
pub struct EllipsoidTangentPlane {
    pub plane: Plane,
    pub origin: DVec3,
    pub x_axis: DVec3,
    pub y_axis: DVec3,
    pub z_axis: DVec3,
    pub normal: DVec3,
    pub ellipsoid: Ellipsoid,
}
impl EllipsoidTangentPlane {
    pub fn new(origin: DVec3, ellipsoid: Option<&Ellipsoid>) -> Self {
        let ellipsoid = ellipsoid.map_or(Ellipsoid::WGS84, |x| x.clone());
        let east_north_up = eastNorthUpToFixedFrame(&origin, Some(ellipsoid));
        let x_axis = DVec3::from_cartesian4(east_north_up.col(0));
        let y_axis = DVec3::from_cartesian4(east_north_up.col(1));
        let z_axis = DVec3::from_cartesian4(east_north_up.col(2));
        let normal = z_axis;
        let plane = Plane::from_point_normal(&origin, &normal);
        return Self {
            plane,
            origin,
            x_axis,
            y_axis,
            z_axis,
            normal,
            ellipsoid,
        };
    }
    pub fn from_points(cartesians: Vec<DVec3>, ellipsoid: Option<&Ellipsoid>) -> Self {
        let aabb = AxisAlignedBoundingBox::from_points(cartesians);
        return EllipsoidTangentPlane::new(aabb.center, ellipsoid);
    }
    pub fn project_point_to_nearest_on_plane(&self, cartesian: DVec3) -> DVec2 {
        let mut ray = Ray::default();
        ray.origin = cartesian;
        ray.direction = self.normal.clone();
        let mut intersection_point: DVec3 = DVec3::ZERO;
        let mut intersection_point_option = rayPlane(ray, self.plane);
        if intersection_point_option.is_none() {
            ray.direction = ray.direction.negate();
            intersection_point = rayPlane(ray, self.plane).unwrap();
        } else {
            intersection_point = intersection_point_option.unwrap();
        }
        let v = intersection_point - self.origin;
        let x = v.dot(self.x_axis);
        let y = v.dot(self.y_axis);
        return DVec2::new(x, y);
    }
    pub fn project_points_to_nearest_on_plane(&self, cartesians: Vec<DVec3>) -> Vec<DVec2> {
        let mut result: Vec<DVec2> = Vec::new();
        for cartesian in cartesians {
            result.push(self.project_point_to_nearest_on_plane(cartesian));
        }
        return result;
    }
    pub fn project_point_onto_ellipsoid(&self, cartesian: DVec2) -> DVec3 {
        let ellipsoid = self.ellipsoid;
        let origin = self.origin;
        let x_axis = self.x_axis;
        let y_axis = self.y_axis;
        let mut tmp = DVec3::default();

        tmp = x_axis.multiply_by_scalar(cartesian.x);
        let mut result = origin + tmp;
        tmp = y_axis.multiply_by_scalar(cartesian.y);
        result = result + tmp;
        return ellipsoid.scale_to_geocentric_surface(&result);
    }
    pub fn project_points_onto_ellipsoid(&self, cartesians: Vec<DVec2>) -> Vec<DVec3> {
        let mut result: Vec<DVec3> = Vec::new();
        for cartesian in cartesians {
            result.push(self.project_point_onto_ellipsoid(cartesian));
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
