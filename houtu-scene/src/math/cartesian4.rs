use std::{
    f64::consts::PI,
    fmt::{Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use bevy::math::{DMat4, DVec4};

use crate::{ellipsoid::Ellipsoid, math::*};
pub trait Cartesian4 {
    fn from_elements(x: f64, y: f64, z: f64, w: f64) -> Self;
    fn divide_by_scalar(&self, scalar: f64) -> DVec4;
    fn equals_epsilon(
        &self,
        right: DVec4,
        relative_epsilon: Option<f64>,
        absolute_epsilon: Option<f64>,
    ) -> bool;
    fn equals(&self, right: DVec4) -> bool;
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

    fn equals(&self, right: DVec4) -> bool {
        return self.eq(&right);
    }
    fn equals_epsilon(
        &self,
        right: DVec4,
        relative_epsilon: Option<f64>,
        absolute_epsilon: Option<f64>,
    ) -> bool {
        let res = self.equals(right)
            || equals_epsilon(self.x, right.x, relative_epsilon, absolute_epsilon)
                && equals_epsilon(self.y, right.y, relative_epsilon, absolute_epsilon)
                && equals_epsilon(self.z, right.z, relative_epsilon, absolute_epsilon)
                && equals_epsilon(self.w, right.w, relative_epsilon, absolute_epsilon);
        return res;
    }
}
