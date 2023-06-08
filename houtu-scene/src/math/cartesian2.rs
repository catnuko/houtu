use std::{
    f64::consts::PI,
    fmt::{Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use bevy::math::{DVec2, DVec4, Vec2};

use crate::{ellipsoid::Ellipsoid, math::*};
pub trait Cartesian2 {
    const ZERO: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    const ONE: Vec2 = Vec2 { x: 1.0, y: 1.0 };
    const UNIT_X: Vec2 = Vec2 { x: 1.0, y: 0.0 };
    const UNIT_Y: Vec2 = Vec2 { x: 0.0, y: 1.0 };
    const UNIT_Z: Vec2 = Vec2 { x: 0.0, y: 0.0 };
    fn equals(&self, right: Vec2) -> bool;
    fn equals_epsilon(
        &self,
        right: Vec2,
        relative_epsilon: Option<f32>,
        absolute_epsilon: Option<f32>,
    ) -> bool;
    fn multiply_by_scalar(&self, scalar: f32) -> Vec2;
}
impl Cartesian2 for Vec2 {
    fn equals(&self, right: Vec2) -> bool {
        return self.eq(&right);
    }
    fn equals_epsilon(
        &self,
        right: Vec2,
        relative_epsilon: Option<f32>,
        absolute_epsilon: Option<f32>,
    ) -> bool {
        let res = self.equals(right)
            || equals_epsilon_f32(self.x, right.x, relative_epsilon, absolute_epsilon)
                && equals_epsilon_f32(self.y, right.y, relative_epsilon, absolute_epsilon);
        return res;
    }
    fn multiply_by_scalar(&self, scalar: f32) -> Vec2 {
        return Vec2::new(self.x * scalar, self.y * scalar);
    }
}
