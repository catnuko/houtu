use std::{
    f64::consts::PI,
    fmt::{Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use bevy::math::DVec4;

use crate::{ellipsoid::Ellipsoid, math::*};
pub trait Cartesian4 {
    fn from_elements(x: f64, y: f64, z: f64, w: f64) -> Self;
    fn divide_by_scalar(&self, scalar: f64) -> DVec4;
}
impl Cartesian4 for DVec4 {
    fn from_elements(x: f64, y: f64, z: f64, w: f64) -> Self {
        return DVec4::new(x, y, z, w);
    }
    fn divide_by_scalar(&self, scalar: f64) -> DVec4 {
        let mut result = DVec4::ZERO;
        result.x = self.x / scalar;
        result.y = self.y / scalar;
        result.z = self.z / scalar;
        result.w = self.w / scalar;
        return result;
    }
}
