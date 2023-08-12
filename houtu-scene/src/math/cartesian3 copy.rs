use std::{
    f64::consts::PI,
    fmt::{Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use bevy::math::DVec3;

use crate::{ellipsoid::Ellipsoid, math::equals_epsilon};
#[derive(Debug, Copy, Clone)]
pub struct Cartesian3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Default for Cartesian3 {
    fn default() -> Self {
        return Cartesian3::ZERO;
    }
}
impl Cartesian3 {
    pub const ZERO: Cartesian3 = Cartesian3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Cartesian3 = Cartesian3 {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    pub const UNIT_X: Cartesian3 = Cartesian3 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const UNIT_Y: Cartesian3 = Cartesian3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const UNIT_Z: Cartesian3 = Cartesian3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Cartesian3 { x, y, z }
    }
    pub fn from_vec3(vec3: DVec3) -> Self {
        Cartesian3 {
            x: vec3.x,
            y: vec3.y,
            z: vec3.z,
        }
    }
    pub fn from_array(array: [f64; 3]) -> Self {
        Cartesian3 {
            x: array[0],
            y: array[1],
            z: array[2],
        }
    }

    // pub fn from_spherical(spherical: Spherical) -> Self {
    //     let x = spherical.radius * spherical.sinTheta * spherical.cosPhi;
    //     let y = spherical.radius * spherical.sinTheta * spherical.sinPhi;
    //     let z = spherical.radius * spherical.cosTheta;
    //     return Cartesian3::new(x, y, z);
    // }
    pub fn from_elements(x: f64, y: f64, z: f64) -> Self {
        Cartesian3 { x, y, z }
    }
    pub fn maximum_component(&self) -> f64 {
        return self.x.max(self.y).max(self.z);
    }
    pub fn minimum_component(&self) -> f64 {
        return self.x.min(self.y).min(self.z);
    }
    pub fn minimum_by_component(&self, other: &Cartesian3) -> Cartesian3 {
        return Cartesian3::new(
            self.x.min(other.x),
            self.y.min(other.y),
            self.z.min(other.z),
        );
    }
    pub fn maximum_by_component(&self, other: &Cartesian3) -> Cartesian3 {
        return Cartesian3::new(
            self.x.max(other.x),
            self.y.max(other.y),
            self.z.max(other.z),
        );
    }
    pub fn clamp(&self, min: &Cartesian3, max: &Cartesian3) -> Cartesian3 {
        return Cartesian3::new(
            self.x.max(min.x).min(max.x),
            self.y.max(min.y).min(max.y),
            self.z.max(min.z).min(max.z),
        );
    }
    pub fn magnitude_squared(&self) -> f64 {
        return self.x * self.x + self.y * self.y + self.z * self.z;
    }
    pub fn magnitude(&self) -> f64 {
        return self.magnitude_squared().sqrt();
    }
    pub fn distance_squared(&self, cartesian: &Cartesian3) -> f64 {
        let x = self.x - cartesian.x;
        let y = self.y - cartesian.y;
        let z = self.z - cartesian.z;
        return x * x + y * y + z * z;
    }
    pub fn distance(&self, right: &Cartesian3) -> f64 {
        return (*self - right).magnitude();
    }
    pub fn normalize(&self) -> Cartesian3 {
        let magnitude = self.magnitude();
        if magnitude == 0.0 {
            return Cartesian3::ZERO;
        }
        return Cartesian3::new(self.x / magnitude, self.y / magnitude, self.z / magnitude);
    }
    pub fn dot(self, right: &Cartesian3) -> f64 {
        return self.x * right.x + self.y * right.y + self.z * right.z;
    }
    pub fn cross(&self, right: &Cartesian3) -> Cartesian3 {
        return Cartesian3::new(
            self.y * right.z - self.z * right.y,
            self.z * right.x - self.x * right.z,
            self.x * right.y - self.y * right.x,
        );
    }
    pub fn negate(&self) -> Cartesian3 {
        return Cartesian3::new(-self.x, -self.y, -self.z);
    }
    pub fn subtract(self, right: &Cartesian3) -> Cartesian3 {
        return Cartesian3::new(self.x - right.x, self.y - right.y, self.z - right.z);
    }
    pub fn multiply_components(&self, right: &Cartesian3) -> Cartesian3 {
        return Cartesian3::new(self.x * right.x, self.y * right.y, self.z * right.z);
    }
    pub fn multiply_by_scalar(&self, scalar: f64) -> Cartesian3 {
        return Cartesian3::new(self.x * scalar, self.y * scalar, self.z * scalar);
    }
    pub fn add(&self, right: &Cartesian3) -> Cartesian3 {
        return Cartesian3::new(self.x + right.x, self.y + right.y, self.z + right.z);
    }
    pub fn divide_by_scalar(&self, scalar: f64) -> Cartesian3 {
        return Cartesian3::new(self.x / scalar, self.y / scalar, self.z / scalar);
    }
    pub fn devide_components(&self, right: &Cartesian3) -> Cartesian3 {
        return Cartesian3::new(self.x / right.x, self.y / right.y, self.z / right.z);
    }
    pub fn abs(&self) -> Cartesian3 {
        return Cartesian3::new(self.x.abs(), self.y.abs(), self.z.abs());
    }
    pub fn lerp(&self, end: &Cartesian3, t: f64) -> Cartesian3 {
        let a = self.multiply_by_scalar(1.0 - t);
        let b = end.multiply_by_scalar(t);
        return b.add(&a);
    }
    pub fn angle_between(&self, right: &Cartesian3) -> f64 {
        let mag = self.magnitude() * right.magnitude();
        if mag == 0.0 {
            return 0.0;
        }
        let cosine = self.dot(right) / mag;
        return cosine.acos();
    }
    pub fn equals(&self, right: &Cartesian3) -> bool {
        return self.x == right.x && self.y == right.y && self.z == right.z;
    }
    pub fn equals_array(&self, right: [f64; 3]) -> bool {
        return self.x == right[0] && self.y == right[1] && self.z == right[2];
    }
    pub fn equals_epsilon(
        &self,
        right: &Cartesian3,
        relative_epsilon: Option<f64>,
        absolute_epsilon: Option<f64>,
    ) -> bool {
        let res = self.equals(right)
            || equals_epsilon(self.x, right.x, relative_epsilon, absolute_epsilon)
                && equals_epsilon(self.y, right.y, relative_epsilon, absolute_epsilon)
                && equals_epsilon(self.z, right.z, relative_epsilon, absolute_epsilon);
        return res;
    }
    pub fn midpoint(&self, right: &Cartesian3) -> Cartesian3 {
        return self.add(right).multiply_by_scalar(0.5);
    }
    pub fn from_degrees(
        longitude: f64,
        latitude: f64,
        height: Option<f64>,
        radii_squared: Option<Cartesian3>,
    ) -> Cartesian3 {
        let longitude = longitude.to_radians();
        let latitude = latitude.to_radians();
        return Cartesian3::from_radians(longitude, latitude, height, radii_squared);
    }
    pub fn from_radians(
        longitude: f64,
        latitude: f64,
        height: Option<f64>,
        radii_squared: Option<Cartesian3>,
    ) -> Cartesian3 {
        let result = Cartesian3::ZERO;
        let radii_squared = radii_squared.unwrap_or(Ellipsoid::WGS84.radii_squared);
        let height = height.unwrap_or(0.0);
        let mut scratchN = Cartesian3::ZERO;
        let mut scratchK = Cartesian3::ZERO;
        let cos_latitude = latitude.cos();
        scratchN.x = cos_latitude * longitude.cos();
        scratchN.y = cos_latitude * longitude.sin();
        scratchN.z = latitude.sin();
        scratchK = radii_squared.multiply_components(&scratchN);
        let gamma = scratchN.dot(scratchK).sqrt();
        scratchK = scratchK.divide_by_scalar(gamma);
        scratchN = scratchN.multiply_by_scalar(height);
        return scratchK.add(&scratchN);
    }
    pub fn from_degrees_array(
        coordinates: Vec<f64>,
        radii_squared: Option<Cartesian3>,
    ) -> Vec<Cartesian3> {
        let length = coordinates.len();
        if length == 0 {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(length / 2);
        for i in (0..length).step_by(2) {
            let longitude = coordinates[i];
            let latitude = coordinates[i + 1];
            let index = i / 2;
            result[index] = Cartesian3::from_degrees(longitude, latitude, None, radii_squared);
        }
        return result;
    }
    pub fn from_radians_array(
        coordinates: Vec<f64>,
        radii_squared: Option<Cartesian3>,
    ) -> Vec<Cartesian3> {
        let length = coordinates.len();
        if length == 0 {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(length / 2);
        for i in (0..length).step_by(2) {
            let longitude = coordinates[i];
            let latitude = coordinates[i + 1];
            let index = i / 2;
            result[index] = Cartesian3::from_radians(longitude, latitude, None, radii_squared);
        }
        return result;
    }
    pub fn from_degrees_array_heights(
        coordinates: Vec<f64>,
        radii_squared: Option<Cartesian3>,
    ) -> Vec<Cartesian3> {
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
            result[index] =
                Cartesian3::from_degrees(longitude, latitude, Some(height), radii_squared);
        }
        return result;
    }
    pub fn from_radians_array_heights(
        coordinates: Vec<f64>,
        radii_squared: Option<Cartesian3>,
    ) -> Vec<Cartesian3> {
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
            result[index] =
                Cartesian3::from_radians(longitude, latitude, Some(height), radii_squared);
        }
        return result;
    }
}
impl Add<&Cartesian3> for Cartesian3 {
    type Output = Cartesian3;
    fn add(self, other: &Cartesian3) -> Cartesian3 {
        return Cartesian3::new(self.x + other.x, self.y + other.y, self.z + other.z);
    }
}
impl Add for Cartesian3 {
    type Output = Cartesian3;
    fn add(self, other: Self) -> Cartesian3 {
        return Cartesian3::new(self.x + other.x, self.y + other.y, self.z + other.z);
    }
}
impl Sub<&Cartesian3> for Cartesian3 {
    type Output = Cartesian3;
    fn sub(self, rhs: &Self) -> Self::Output {
        self.subtract(rhs)
    }
}
impl Sub<Cartesian3> for Cartesian3 {
    type Output = Cartesian3;
    fn sub(self, rhs: Self) -> Self::Output {
        self.subtract(rhs)
    }
}
impl Mul<f64> for Cartesian3 {
    type Output = Cartesian3;
    fn mul(self, rhs: f64) -> Self::Output {
        self.multiply_by_scalar(rhs)
    }
}
impl Mul<&Cartesian3> for Cartesian3 {
    type Output = Cartesian3;
    fn mul(self, rhs: &Cartesian3) -> Self::Output {
        self.multiply_components(rhs)
    }
}
impl Mul for Cartesian3 {
    type Output = Cartesian3;
    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply_components(&rhs)
    }
}
impl Div<f64> for Cartesian3 {
    type Output = Cartesian3;
    fn div(self, rhs: f64) -> Self::Output {
        Cartesian3::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}
impl Div<&Cartesian3> for Cartesian3 {
    type Output = Cartesian3;
    fn div(self, rhs: &Cartesian3) -> Self::Output {
        self.devide_components(rhs)
    }
}
impl Div for Cartesian3 {
    type Output = Cartesian3;
    fn div(self, rhs: Self) -> Self::Output {
        self.devide_components(&rhs)
    }
}
impl PartialEq<&Cartesian3> for Cartesian3 {
    fn eq(&self, other: &&Cartesian3) -> bool {
        self.equals(other)
    }
}
impl PartialEq for Cartesian3 {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}
impl ToString for Cartesian3 {
    fn to_string(&self) -> String {
        format!("x:{},y:{},z:{}", self.x, self.y, self.z)
    }
}
impl From<[f64; 3]> for Cartesian3 {
    fn from(array: [f64; 3]) -> Self {
        Cartesian3::new(array[0], array[1], array[2])
    }
}
impl From<DVec3> for Cartesian3 {
    fn from(array: DVec3) -> Self {
        Cartesian3::new(array.x, array.y, array.z)
    }
}
impl From<Cartesian3> for DVec3 {
    fn from(array: Cartesian3) -> Self {
        DVec3::new(array.x, array.y, array.z)
    }
}
#[cfg(test)]
mod tests {
    use crate::{coord::Cartographic, math::ToRadians};

    use super::*;
    //编写测试代码
    #[test]

    fn test_cartesian3() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = Cartesian3::new(1.0, 2.0, 3.0);
        let c = a.add(&b);
        assert_eq!(c.x, 2.0);
        assert_eq!(c.y, 4.0);
        assert_eq!(c.z, 6.0);
    }
    #[test]
    fn test_cartesian3_sub() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = Cartesian3::new(1.0, 2.0, 3.0);
        let c = a.subtract(b);
        assert_eq!(c.x, 0.0);
        assert_eq!(c.y, 0.0);
        assert_eq!(c.z, 0.0);
    }
    #[test]
    fn test_cartesian3_multiply_by_scalar() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = a.multiply_by_scalar(2.0);
        assert_eq!(b.x, 2.0);
        assert_eq!(b.y, 4.0);
        assert_eq!(b.z, 6.0);
    }
    #[test]
    fn test_cartesian3_divide_by_scalar() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = a.divide_by_scalar(2.0);
        assert_eq!(b.x, 0.5);
        assert_eq!(b.y, 1.0);
        assert_eq!(b.z, 1.5);
    }
    #[test]
    fn test_cartesian3_magnitude_squared() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = a.magnitude_squared();
        assert_eq!(b, 14.0);
    }
    #[test]
    fn test_cartesian3_magnitude() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = a.magnitude();
        assert_eq!(b, 3.7416573867739413);
    }
    #[test]
    fn test_cartesian3_normalize() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = a.normalize();
        assert_eq!(b.x, 0.2672612419124244);
        assert_eq!(b.y, 0.5345224838248488);
        assert_eq!(b.z, 0.8017837257372732);
    }
    #[test]
    fn test_cartesian3_dot() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = Cartesian3::new(1.0, 2.0, 3.0);
        let c = a.dot(b);
        assert_eq!(c, 14.0);
    }
    #[test]
    fn test_cartesian3_cross() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = Cartesian3::new(1.0, 2.0, 3.0);
        let c = a.cross(&b);
        assert_eq!(c.x, 0.0);
        assert_eq!(c.y, 0.0);
        assert_eq!(c.z, 0.0);
    }
    #[test]
    fn test_cartesian3_distance() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = Cartesian3::new(1.0, 2.0, 3.0);
        let c = a.distance(b);
        assert_eq!(c, 0.0);
    }
    #[test]
    fn test_cartesian3_distance_squared() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = Cartesian3::new(1.0, 2.0, 3.0);
        let c = a.distance_squared(&b);
        assert_eq!(c, 0.0);
    }
    #[test]
    fn test_cartesian3_lerp() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = Cartesian3::new(1.0, 2.0, 3.0);
        let c = a.lerp(&b, 0.5);
        assert_eq!(c.x, 1.0);
        assert_eq!(c.y, 2.0);
        assert_eq!(c.z, 3.0);
    }
    #[test]
    fn test_cartesian3_equals() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = Cartesian3::new(1.0, 2.0, 3.0);
        assert_eq!(a.equals(&b), true);
    }
    #[test]
    fn test_cartesian3_equals_epsilon() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        assert_eq!(
            a.equals_epsilon(Cartesian3::new(1.0, 2.0, 3.0), Some(0.0), None),
            true
        );
        assert_eq!(
            a.equals_epsilon(Cartesian3::new(1.0, 2.0, 3.0), Some(1.0), None),
            true
        );
        assert_eq!(
            a.equals_epsilon(Cartesian3::new(2.0, 2.0, 3.0), Some(1.0), None),
            true
        );
        assert_eq!(
            a.equals_epsilon(Cartesian3::new(1.0, 3.0, 3.0), Some(1.0), None),
            true
        );
        assert_eq!(
            a.equals_epsilon(Cartesian3::new(1.0, 2.0, 4.0), Some(1.0), None),
            true
        );
    }
    #[test]
    fn test_cartesian3_to_string() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        assert_eq!(a.to_string(), "x:1,y:2,z:3");
    }
    #[test]

    fn test_cartesian3_clone() {
        let a = Cartesian3::new(1.0, 2.0, 3.0);
        let b = a.clone();
        assert_eq!(a, &b);
    }
    #[test]
    fn test_cartesian3_from_array() {
        let a = Cartesian3::from_array([1.0, 2.0, 3.0]);
        assert_eq!(a.x, 1.0);
        assert_eq!(a.y, 2.0);
        assert_eq!(a.z, 3.0);
    }
    #[test]
    fn test_cartesian3_from_degrees() {
        let lon = -115.0;
        let lat = 37.0;
        let height = 100000.0;
        let ellipsoid = Ellipsoid::WGS84;
        let actual = Cartesian3::from_degrees(lon, lat, Some(height), None);
        let expected =
            ellipsoid.cartographic_to_cartesian(&Cartographic::from_degrees(lon, lat, height));
        // expect(actual).toEqual(expected);
        assert!(actual.equals(&expected));
    }
    #[test]
    fn test_cartesian3_from_radians() {
        let lon = 150.0.to_radians();
        let lat = -40.0.to_radians();
        let ellipsoid = Ellipsoid::WGS84;
        let actual = Cartesian3::from_radians(lon, lat, None, None);
        let expected = ellipsoid.cartographic_to_cartesian(&Cartographic::new(lon, lat, 0.0));
        assert!(actual.equals(&expected));
    }
}
