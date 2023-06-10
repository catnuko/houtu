use std::{
    f64::consts::PI,
    fmt::{Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use bevy::math::{DVec2, DVec4};

use crate::{ellipsoid::Ellipsoid, math::*};
pub trait Cartesian2 {
    const ZERO: DVec2 = DVec2 { x: 0.0, y: 0.0 };
    const ONE: DVec2 = DVec2 { x: 1.0, y: 1.0 };
    const UNIT_X: DVec2 = DVec2 { x: 1.0, y: 0.0 };
    const UNIT_Y: DVec2 = DVec2 { x: 0.0, y: 1.0 };
    const UNIT_Z: DVec2 = DVec2 { x: 0.0, y: 0.0 };
    fn equals(&self, right: DVec2) -> bool;
    fn equals_epsilon(
        &self,
        right: DVec2,
        relative_epsilon: Option<f64>,
        absolute_epsilon: Option<f64>,
    ) -> bool;
    fn multiply_by_scalar(&self, scalar: f64) -> DVec2;
}
impl Cartesian2 for DVec2 {
    fn equals(&self, right: DVec2) -> bool {
        return self.eq(&right);
    }
    fn equals_epsilon(
        &self,
        right: DVec2,
        relative_epsilon: Option<f64>,
        absolute_epsilon: Option<f64>,
    ) -> bool {
        let res = self.equals(right)
            || equals_epsilon(self.x, right.x, relative_epsilon, absolute_epsilon)
                && equals_epsilon(self.y, right.y, relative_epsilon, absolute_epsilon);
        return res;
    }
    fn multiply_by_scalar(&self, scalar: f64) -> DVec2 {
        return DVec2::new(self.x * scalar, self.y * scalar);
    }
}
