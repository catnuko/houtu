use std::{
    f64::consts::PI,
    fmt::{Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use bevy::math::{DMat3, DVec3};

use crate::{ellipsoid::Ellipsoid, math::*};
pub trait Matrix3 {
    fn multiply_by_scale(&self, scale: DVec3) -> DMat3;
    fn from_scale3(scale: DVec3) -> DMat3;
}
impl Matrix3 for DMat3 {
    fn multiply_by_scale(&self, scale: DVec3) -> DMat3 {
        let mut result = self.clone();
        result.x_axis = result.x_axis * scale.x;
        result.y_axis = result.y_axis * scale.y;
        result.z_axis = result.z_axis * scale.z;
        result
    }
    fn from_scale3(scale: DVec3) -> DMat3 {
        DMat3::from_cols(
            DVec3::new(scale.x, 0.0, 0.0),
            DVec3::new(0.0, scale.y, 0.0),
            DVec3::new(0.0, 0.0, scale.z),
        )
    }
}
