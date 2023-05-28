use std::{
    f64::consts::PI,
    fmt::{Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use bevy::math::{DVec3, DVec4};

use crate::{ellipsoid::Ellipsoid, math::*};
pub trait Cartesian3 {
    const ZERO: DVec3 = DVec3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    const ONE: DVec3 = DVec3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    const UNIT_X: DVec3 = DVec3 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    const UNIT_Y: DVec3 = DVec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    const UNIT_Z: DVec3 = DVec3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    fn from_radians(
        longitude: f64,
        latitude: f64,
        height: Option<f64>,
        radii_squared: Option<DVec3>,
    ) -> DVec3;
    fn from_degrees(
        longitude: f64,
        latitude: f64,
        height: Option<f64>,
        radii_squared: Option<DVec3>,
    ) -> DVec3;
    fn from_degrees_array(coordinates: Vec<f64>, radii_squared: Option<DVec3>) -> Vec<DVec3>;
    fn from_radians_array(coordinates: Vec<f64>, radii_squared: Option<DVec3>) -> Vec<DVec3>;
    fn from_degrees_array_heights(
        coordinates: Vec<f64>,
        radii_squared: Option<DVec3>,
    ) -> Vec<DVec3>;
    fn from_radians_array_heights(
        coordinates: Vec<f64>,
        radii_squared: Option<DVec3>,
    ) -> Vec<DVec3>;
    fn equals_epsilon(
        &self,
        right: DVec3,
        relative_epsilon: Option<f64>,
        absolute_epsilon: Option<f64>,
    ) -> bool;
    fn equals(&self, right: DVec3) -> bool;
    fn midpoint(&self, right: DVec3) -> DVec3;
    fn multiply_by_scalar(&self, scalar: f64) -> DVec3;
    fn multiply_components(&self, right: &DVec3) -> DVec3;
    fn subtract(self, right: DVec3) -> DVec3;
    fn divide_by_scalar(&self, scalar: f64) -> DVec3;
    fn devide_components(&self, right: DVec3) -> DVec3;
    fn equals_array(&self, right: [f64; 3]) -> bool;
    fn from_elements(x: f64, y: f64, z: f64) -> Self;
    fn maximum_component(&self) -> f64;
    fn minimum_component(&self) -> f64;
    fn minimum_by_component(&self, other: DVec3) -> DVec3;
    fn maximum_by_component(&self, other: DVec3) -> DVec3;
    fn magnitude_squared(&self) -> f64;
    fn magnitude(&self) -> f64;
    fn negate(&self) -> DVec3;
    fn from_cartesian4(vec4: DVec4) -> DVec3;
}
impl Cartesian3 for DVec3 {
    fn negate(&self) -> DVec3 {
        return DVec3::new(-self.x, -self.y, -self.z);
    }
    fn magnitude(&self) -> f64 {
        return self.length();
    }
    fn magnitude_squared(&self) -> f64 {
        return self.length_squared();
    }
    fn from_elements(x: f64, y: f64, z: f64) -> Self {
        return DVec3::new(x, y, z);
    }
    fn maximum_component(&self) -> f64 {
        return self.x.max(self.y).max(self.z);
    }
    fn minimum_component(&self) -> f64 {
        return self.x.min(self.y).min(self.z);
    }
    fn minimum_by_component(&self, other: DVec3) -> DVec3 {
        return DVec3::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        );
    }
    fn maximum_by_component(&self, other: DVec3) -> DVec3 {
        return DVec3::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        );
    }
    fn equals(&self, right: DVec3) -> bool {
        return self.eq(&right);
    }
    fn equals_array(&self, right: [f64; 3]) -> bool {
        return self.x == right[0] && self.y == right[1] && self.z == right[2];
    }
    fn equals_epsilon(
        &self,
        right: DVec3,
        relative_epsilon: Option<f64>,
        absolute_epsilon: Option<f64>,
    ) -> bool {
        let res = self.equals(right)
            || equals_epsilon(self.x, right.x, relative_epsilon, absolute_epsilon)
                && equals_epsilon(self.y, right.y, relative_epsilon, absolute_epsilon)
                && equals_epsilon(self.z, right.z, relative_epsilon, absolute_epsilon);
        return res;
    }
    fn multiply_components(&self, right: &DVec3) -> DVec3 {
        return DVec3::new(self.x * right.x, self.y * right.y, self.z * right.z);
    }
    fn multiply_by_scalar(&self, scalar: f64) -> DVec3 {
        return DVec3::new(self.x * scalar, self.y * scalar, self.z * scalar);
    }
    fn divide_by_scalar(&self, scalar: f64) -> DVec3 {
        return DVec3::new(self.x / scalar, self.y / scalar, self.z / scalar);
    }
    fn devide_components(&self, right: DVec3) -> DVec3 {
        return DVec3::new(self.x / right.x, self.y / right.y, self.z / right.z);
    }
    fn midpoint(&self, right: DVec3) -> DVec3 {
        return self.add(right).multiply_by_scalar(0.5);
    }
    fn subtract(self, right: DVec3) -> DVec3 {
        return self.sub(right);
    }
    fn from_degrees(
        longitude: f64,
        latitude: f64,
        height: Option<f64>,
        radii_squared: Option<DVec3>,
    ) -> DVec3 {
        let longitude = longitude.to_radians();
        let latitude = latitude.to_radians();
        return DVec3::from_radians(longitude, latitude, height, radii_squared);
    }
    fn from_radians(
        longitude: f64,
        latitude: f64,
        height: Option<f64>,
        radii_squared: Option<DVec3>,
    ) -> DVec3 {
        let result = DVec3::ZERO;
        let radii_squared = radii_squared.unwrap_or(Ellipsoid::WGS84.radiiSquared);
        let height = height.unwrap_or(0.0);
        let mut scratchN = DVec3::ZERO;
        let mut scratchK = DVec3::ZERO;
        let cosLatitude = latitude.cos();
        scratchN.x = cosLatitude * longitude.cos();
        scratchN.y = cosLatitude * longitude.sin();
        scratchN.z = latitude.sin();
        scratchK = radii_squared.multiply_components(&scratchN);
        let gamma = scratchN.dot(scratchK).sqrt();
        scratchK = scratchK.divide_by_scalar(gamma);
        scratchN = scratchN.multiply_by_scalar(height);
        return scratchK.add(scratchN);
    }
    fn from_degrees_array(coordinates: Vec<f64>, radii_squared: Option<DVec3>) -> Vec<DVec3> {
        let length = coordinates.len();
        if length == 0 {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(length / 2);
        for i in (0..length).step_by(2) {
            let longitude = coordinates[i];
            let latitude = coordinates[i + 1];
            let index = i / 2;
            result[index] = DVec3::from_degrees(longitude, latitude, None, radii_squared);
        }
        return result;
    }
    fn from_radians_array(coordinates: Vec<f64>, radii_squared: Option<DVec3>) -> Vec<DVec3> {
        let length = coordinates.len();
        if length == 0 {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(length / 2);
        for i in (0..length).step_by(2) {
            let longitude = coordinates[i];
            let latitude = coordinates[i + 1];
            let index = i / 2;
            result[index] = DVec3::from_radians(longitude, latitude, None, radii_squared);
        }
        return result;
    }
    fn from_degrees_array_heights(
        coordinates: Vec<f64>,
        radii_squared: Option<DVec3>,
    ) -> Vec<DVec3> {
        let length = coordinates.len();
        if length == 0 {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(length / 3);
        for i in (0..length).step_by(3) {
            let longitude = coordinates[i];
            let latitude = coordinates[i + 1];
            let height = coordinates[i + 2];
            let index = i / 3;
            result[index] = DVec3::from_degrees(longitude, latitude, Some(height), radii_squared);
        }
        return result;
    }
    fn from_radians_array_heights(
        coordinates: Vec<f64>,
        radii_squared: Option<DVec3>,
    ) -> Vec<DVec3> {
        let length = coordinates.len();
        if length == 0 {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(length / 3);
        for i in (0..length).step_by(3) {
            let longitude = coordinates[i];
            let latitude = coordinates[i + 1];
            let height = coordinates[i + 2];
            let index = i / 3;
            result[index] = DVec3::from_radians(longitude, latitude, Some(height), radii_squared);
        }
        return result;
    }
    fn from_cartesian4(vec4: DVec4) -> DVec3 {
        return DVec3::new(vec4.x, vec4.y, vec4.z);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    //编写测试代码
    #[test]

    fn test_vec3() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = DVec3::new(1.0, 2.0, 3.0);
        let c = a.add(b);
        assert_eq!(c.x, 2.0);
        assert_eq!(c.y, 4.0);
        assert_eq!(c.z, 6.0);
    }
    #[test]
    fn test_vec3_sub() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = DVec3::new(1.0, 2.0, 3.0);
        let c = a.subtract(b);
        assert_eq!(c.x, 0.0);
        assert_eq!(c.y, 0.0);
        assert_eq!(c.z, 0.0);
    }
    #[test]
    fn test_vec3_multiply_by_scalar() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = a.multiply_by_scalar(2.0);
        assert_eq!(b.x, 2.0);
        assert_eq!(b.y, 4.0);
        assert_eq!(b.z, 6.0);
    }
    #[test]
    fn test_vec3_divide_by_scalar() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = a.divide_by_scalar(2.0);
        assert_eq!(b.x, 0.5);
        assert_eq!(b.y, 1.0);
        assert_eq!(b.z, 1.5);
    }
    #[test]
    fn test_vec3_magnitude_squared() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = a.magnitude_squared();
        assert_eq!(b, 14.0);
    }
    #[test]
    fn test_vec3_magnitude() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = a.magnitude();
        assert_eq!(b, 3.7416573867739413);
    }
    #[test]
    fn test_vec3_normalize() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = a.normalize();
        assert_eq!(b.x, 0.2672612419124244);
        assert_eq!(b.y, 0.5345224838248488);
        assert_eq!(b.z, 0.8017837257372732);
    }
    #[test]
    fn test_vec3_dot() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = DVec3::new(1.0, 2.0, 3.0);
        let c = a.dot(b);
        assert_eq!(c, 14.0);
    }
    #[test]
    fn test_vec3_cross() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = DVec3::new(1.0, 2.0, 3.0);
        let c = a.cross(b);
        assert_eq!(c.x, 0.0);
        assert_eq!(c.y, 0.0);
        assert_eq!(c.z, 0.0);
    }
    #[test]
    fn test_vec3_distance() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = DVec3::new(1.0, 2.0, 3.0);
        let c = a.distance(b);
        assert_eq!(c, 0.0);
    }
    #[test]
    fn test_vec3_distance_squared() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = DVec3::new(1.0, 2.0, 3.0);
        let c = a.distance_squared(b);
        assert_eq!(c, 0.0);
    }
    #[test]
    fn test_vec3_lerp() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = DVec3::new(1.0, 2.0, 3.0);
        let c = a.lerp(b, 0.5);
        assert_eq!(c.x, 1.0);
        assert_eq!(c.y, 2.0);
        assert_eq!(c.z, 3.0);
    }
    #[test]
    fn test_vec3_equals() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = DVec3::new(1.0, 2.0, 3.0);
        assert_eq!(a.equals(b), true);
    }
    #[test]
    fn test_vec3_equals_epsilon() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        assert_eq!(
            a.equals_epsilon(DVec3::new(1.0, 2.0, 3.0), Some(0.0), None),
            true
        );
        assert_eq!(
            a.equals_epsilon(DVec3::new(1.0, 2.0, 3.0), Some(1.0), None),
            true
        );
        assert_eq!(
            a.equals_epsilon(DVec3::new(2.0, 2.0, 3.0), Some(1.0), None),
            true
        );
        assert_eq!(
            a.equals_epsilon(DVec3::new(1.0, 3.0, 3.0), Some(1.0), None),
            true
        );
        assert_eq!(
            a.equals_epsilon(DVec3::new(1.0, 2.0, 4.0), Some(1.0), None),
            true
        );
    }
    #[test]
    fn test_vec3_to_string() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        assert_eq!(a.to_string(), "x:1,y:2,z:3");
    }

    fn test_vec3_clone() {
        let a = DVec3::new(1.0, 2.0, 3.0);
        let b = a.clone();
        assert_eq!(a, b);
    }
    #[test]
    fn test_vec3_from_array() {
        let a = DVec3::from_array([1.0, 2.0, 3.0]);
        assert_eq!(a.x, 1.0);
        assert_eq!(a.y, 2.0);
        assert_eq!(a.z, 3.0);
    }
    #[test]
    fn test_vec3_from_degrees() {
        let lon = -115.0;
        let lat = 37.0;
        let height = 100000.0;
        let ellipsoid = Ellipsoid::WGS84;
        let actual = DVec3::from_degrees(lon, lat, Some(height), None);
        let expected =
            ellipsoid.cartographicToCartesian(&Cartographic::from_degrees(lon, lat, height));
        // expect(actual).toEqual(expected);
        assert!(actual.equals(expected));
    }
    #[test]
    fn test_vec3_from_radians() {
        let lon = 150.0.to_radians();
        let lat = -40.0.to_radians();
        let ellipsoid = Ellipsoid::WGS84;
        let actual = DVec3::from_radians(lon, lat, None, None);
        let expected = ellipsoid.cartographicToCartesian(&Cartographic::new(lon, lat, 0.0));
        assert!(actual.equals(expected));
    }
}
