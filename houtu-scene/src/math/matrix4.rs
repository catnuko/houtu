use std::{
    fmt::{Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use bevy::math::{DMat3, DMat4, DVec3, DVec4};

use crate::{ellipsoid::Ellipsoid, math::*, BoundingRectangle};
pub trait Matrix3 {
    const COLUMN0ROW0: usize;
    const COLUMN0ROW1: usize;
    const COLUMN0ROW2: usize;
    const COLUMN1ROW0: usize;
    const COLUMN1ROW1: usize;
    const COLUMN1ROW2: usize;
    const COLUMN2ROW0: usize;
    const COLUMN2ROW1: usize;
    const COLUMN2ROW2: usize;
    fn multiply_by_scale(&self, scale: DVec3) -> DMat3;
    fn from_scale3(scale: DVec3) -> DMat3;
    fn set_column(&mut self, index: usize, cartesian: &DVec3);
    fn multiply_by_vector(&self, cartesian: &DVec3) -> DVec3;
    fn get_column(&self, index: usize) -> DVec3;
}
impl Matrix3 for DMat3 {
    const COLUMN0ROW0: usize = 0;
    const COLUMN0ROW1: usize = 1;
    const COLUMN0ROW2: usize = 2;
    const COLUMN1ROW0: usize = 3;
    const COLUMN1ROW1: usize = 4;
    const COLUMN1ROW2: usize = 5;
    const COLUMN2ROW0: usize = 6;
    const COLUMN2ROW1: usize = 7;
    const COLUMN2ROW2: usize = 8;
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
    fn set_column(&mut self, index: usize, cartesian: &DVec3) {
        if index == 0 {
            self.x_axis = cartesian.clone();
        } else if index == 1 {
            self.y_axis = cartesian.clone();
        } else if index == 2 {
            self.z_axis = cartesian.clone();
        } else {
            panic!("index out of range")
        }
    }
    fn get_column(&self, index: usize) -> DVec3 {
        if index == 0 {
            return self.x_axis;
        } else if index == 1 {
            return self.y_axis;
        } else if index == 2 {
            return self.z_axis;
        } else {
            panic!("index out of range")
        }
    }

    fn multiply_by_vector(&self, cartesian: &DVec3) -> DVec3 {
        let mut result = DVec3::ZERO;
        let mut slice: [f64; 9] = [0.; 9];
        self.write_cols_to_slice(&mut slice);
        let vX = cartesian.x;
        let vY = cartesian.y;
        let vZ = cartesian.z;

        let x = slice[0] * vX + slice[3] * vY + slice[6] * vZ;
        let y = slice[1] * vX + slice[4] * vY + slice[7] * vZ;
        let z = slice[2] * vX + slice[5] * vY + slice[8] * vZ;

        result.x = x;
        result.y = y;
        result.z = z;
        return result;
    }
}
pub trait Matrix4 {
    fn inverse_transformation(&self) -> DMat4;
    fn multiply_by_point(&self, cartesian: &DVec3) -> DVec3;
    fn set_translation(&mut self, cartesian: &DVec3);
    fn compute_view(position: &DVec3, direction: &DVec3, up: &DVec3, right: &DVec3) -> DMat4;
    fn multiply_by_vector(&self, cartesian: &DVec4) -> DVec4;
}
impl Matrix4 for DMat4 {
    fn set_translation(&mut self, cartesian: &DVec3) {
        self.x_axis.w = cartesian.x;
        self.y_axis.w = cartesian.y;
        self.z_axis.w = cartesian.z;
    }
    fn multiply_by_vector(&self, cartesian: &DVec4) -> DVec4 {
        let mut matrix: [f64; 16] = [
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
        ];
        self.write_cols_to_slice(&mut matrix);
        let mut result = DVec4::ZERO;
        let vX = cartesian.x;
        let vY = cartesian.y;
        let vZ = cartesian.z;
        let vW = cartesian.w;

        let x = matrix[0] * vX + matrix[4] * vY + matrix[8] * vZ + matrix[12] * vW;
        let y = matrix[1] * vX + matrix[5] * vY + matrix[9] * vZ + matrix[13] * vW;
        let z = matrix[2] * vX + matrix[6] * vY + matrix[10] * vZ + matrix[14] * vW;
        let w = matrix[3] * vX + matrix[7] * vY + matrix[11] * vZ + matrix[15] * vW;

        result.x = x;
        result.y = y;
        result.z = z;
        result.w = w;
        return result;
    }
    fn inverse_transformation(&self) -> DMat4 {
        let mut slice: [f64; 16] = [
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
        ];
        self.write_cols_to_slice(&mut slice);
        let matrix0 = slice[0];
        let matrix1 = slice[1];
        let matrix2 = slice[2];
        let matrix4 = slice[4];
        let matrix5 = slice[5];
        let matrix6 = slice[6];
        let matrix8 = slice[8];
        let matrix9 = slice[9];
        let matrix10 = slice[10];

        let vX = slice[12];
        let vY = slice[13];
        let vZ = slice[14];

        let x = -matrix0 * vX - matrix1 * vY - matrix2 * vZ;
        let y = -matrix4 * vX - matrix5 * vY - matrix6 * vZ;
        let z = -matrix8 * vX - matrix9 * vY - matrix10 * vZ;
        let mut slice2: [f64; 16] = [0.; 16];
        slice2[0] = matrix0;
        slice2[1] = matrix4;
        slice2[2] = matrix8;
        slice2[3] = 0.0;
        slice2[4] = matrix1;
        slice2[5] = matrix5;
        slice2[6] = matrix9;
        slice2[7] = 0.0;
        slice2[8] = matrix2;
        slice2[9] = matrix6;
        slice2[10] = matrix10;
        slice2[11] = 0.0;
        slice2[12] = x;
        slice2[13] = y;
        slice2[14] = z;
        slice2[15] = 1.0;
        return DMat4::from_cols_array(&slice2);
    }

    fn multiply_by_point(&self, cartesian: &DVec3) -> DVec3 {
        let mut slice: [f64; 16] = [0.; 16];
        self.write_cols_to_slice(&mut slice);
        let matrix0 = slice[0];
        let matrix1 = slice[1];
        let matrix2 = slice[2];
        let matrix4 = slice[4];
        let matrix5 = slice[5];
        let matrix6 = slice[6];
        let matrix8 = slice[8];
        let matrix9 = slice[9];
        let matrix10 = slice[10];

        let vX = cartesian.x;
        let vY = cartesian.y;
        let vZ = cartesian.z;

        let x = matrix0 * vX + matrix1 * vY + matrix2 * vZ + slice[12];
        let y = matrix4 * vX + matrix5 * vY + matrix6 * vZ + slice[13];
        let z = matrix8 * vX + matrix9 * vY + matrix10 * vZ + slice[14];
        return DVec3::new(x, y, z);
    }
    fn compute_view(position: &DVec3, direction: &DVec3, up: &DVec3, right: &DVec3) -> DMat4 {
        let mut result: [f64; 16] = [0.; 16];
        result[0] = right.x;
        result[1] = up.x;
        result[2] = -direction.x;
        result[3] = 0.0;
        result[4] = right.y;
        result[5] = up.y;
        result[6] = -direction.y;
        result[7] = 0.0;
        result[8] = right.z;
        result[9] = up.z;
        result[10] = -direction.z;
        result[11] = 0.0;
        result[12] = -right.dot(*position);
        result[13] = -up.dot(*position);
        result[14] = direction.dot(*position);
        result[15] = 1.0;
        return DMat4::from_cols_array(&result);
    }
}
