use std::{
    fmt::{Debug, Formatter},
    ops::{Add, Div, Mul, Sub},
};

use bevy::math::{DMat3, DMat4, DQuat, DVec3, DVec4};

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
    fn equals_epsilon(&self, right: &DMat3, epsilon: f64) -> bool;
    fn from_quaternion(quaternion: &DQuat) -> DMat3;
    fn from_raw_list(slice: [f64; 9]) -> DMat3;
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
    fn from_quaternion(quaternion: &DQuat) -> DMat3 {
        let x2 = quaternion.x * quaternion.x;
        let xy = quaternion.x * quaternion.y;
        let xz = quaternion.x * quaternion.z;
        let xw = quaternion.x * quaternion.w;
        let y2 = quaternion.y * quaternion.y;
        let yz = quaternion.y * quaternion.z;
        let yw = quaternion.y * quaternion.w;
        let z2 = quaternion.z * quaternion.z;
        let zw = quaternion.z * quaternion.w;
        let w2 = quaternion.w * quaternion.w;

        let m00 = x2 - y2 - z2 + w2;
        let m01 = 2.0 * (xy - zw);
        let m02 = 2.0 * (xz + yw);

        let m10 = 2.0 * (xy + zw);
        let m11 = -x2 + y2 - z2 + w2;
        let m12 = 2.0 * (yz - xw);

        let m20 = 2.0 * (xz - yw);
        let m21 = 2.0 * (yz + xw);
        let m22 = -x2 - y2 + z2 + w2;
        return Self::from_raw_list([m00, m01, m02, m10, m11, m12, m20, m21, m22]);
        // return Matrix3(m00, m01, m02, m10, m11, m12, m20, m21, m22);
    }
    fn from_raw_list(slice: [f64; 9]) -> DMat3 {
        return make_matrix3_from_raw(slice);
    }
    fn equals_epsilon(&self, right: &DMat3, epsilon: f64) -> bool {
        let mut slice: [f64; 9] = [0.; 9];
        self.write_cols_to_slice(&mut slice);

        let mut slice2: [f64; 9] = [0.; 9];
        right.write_cols_to_slice(&mut slice2);

        return (slice[0] - slice2[0]).abs() <= epsilon
            && (slice[1] - slice2[1]).abs() <= epsilon
            && (slice[2] - slice2[2]).abs() <= epsilon
            && (slice[3] - slice2[3]).abs() <= epsilon
            && (slice[4] - slice2[4]).abs() <= epsilon
            && (slice[5] - slice2[5]).abs() <= epsilon
            && (slice[6] - slice2[6]).abs() <= epsilon
            && (slice[7] - slice2[7]).abs() <= epsilon
            && (slice[8] - slice2[8]).abs() <= epsilon;
    }
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
//按行生成
fn make_matrix4_from_raw(slice: [f64; 16]) -> DMat4 {
    DMat4 {
        x_axis: [slice[0], slice[4], slice[8], slice[12]].into(),
        y_axis: [slice[1], slice[5], slice[9], slice[13]].into(),
        z_axis: [slice[2], slice[6], slice[10], slice[14]].into(),
        w_axis: [slice[3], slice[7], slice[11], slice[15]].into(),
    }
}
//按行生成
fn make_matrix3_from_raw(slice: [f64; 9]) -> DMat3 {
    DMat3 {
        x_axis: [slice[0], slice[3], slice[6]].into(),
        y_axis: [slice[1], slice[4], slice[7]].into(),
        z_axis: [slice[2], slice[5], slice[8]].into(),
    }
}

pub trait Matrix4 {
    fn inverse_transformation(&self) -> DMat4;
    fn multiply_by_point(&self, cartesian: &DVec3) -> DVec3;
    fn set_translation(&mut self, cartesian: &DVec3);
    fn get_translation(&self) -> DVec3;
    fn compute_view(position: &DVec3, direction: &DVec3, up: &DVec3, right: &DVec3) -> DMat4;
    fn multiply_by_vector(&self, cartesian: &DVec4) -> DVec4;
    fn multiply_by_point_as_vector(&self, cartesian: &DVec3) -> DVec3;
    fn from_raw_list(slice: [f64; 16]) -> DMat4;
    fn to_mat4_32(&self) -> Mat4;
    fn compute_perspective_off_center(
        left: f64,
        right: f64,
        bottom: f64,
        top: f64,
        near: f64,
        far: f64,
    ) -> DMat4;
    fn compute_orthographic_off_center(
        left: f64,
        right: f64,
        bottom: f64,
        top: f64,
        near: f64,
        far: f64,
    ) -> DMat4;
    fn compute_infinite_perspective_off_center(
        left: f64,
        right: f64,
        bottom: f64,
        top: f64,
        near: f64,
    ) -> DMat4;
}
impl Matrix4 for DMat4 {
    fn compute_infinite_perspective_off_center(
        left: f64,
        right: f64,
        bottom: f64,
        top: f64,
        near: f64,
    ) -> DMat4 {
        let mut result: [f64; 16] = [0.; 16];

        let column0Row0 = (2.0 * near) / (right - left);
        let column1Row1 = (2.0 * near) / (top - bottom);
        let column2Row0 = (right + left) / (right - left);
        let column2Row1 = (top + bottom) / (top - bottom);
        let column2Row2 = -1.0;
        let column2Row3 = -1.0;
        let column3Row2 = -2.0 * near;

        result[0] = column0Row0;
        result[1] = 0.0;
        result[2] = 0.0;
        result[3] = 0.0;
        result[4] = 0.0;
        result[5] = column1Row1;
        result[6] = 0.0;
        result[7] = 0.0;
        result[8] = column2Row0;
        result[9] = column2Row1;
        result[10] = column2Row2;
        result[11] = column2Row3;
        result[12] = 0.0;
        result[13] = 0.0;
        result[14] = column3Row2;
        result[15] = 0.0;
        return DMat4::from_cols_array(&result);
    }
    fn compute_perspective_off_center(
        left: f64,
        right: f64,
        bottom: f64,
        top: f64,
        near: f64,
        far: f64,
    ) -> DMat4 {
        let mut result: [f64; 16] = [0.; 16];
        let column0Row0 = (2.0 * near) / (right - left);
        let column1Row1 = (2.0 * near) / (top - bottom);
        let column2Row0 = (right + left) / (right - left);
        let column2Row1 = (top + bottom) / (top - bottom);
        let column2Row2 = -(far + near) / (far - near);
        let column2Row3 = -1.0;
        let column3Row2 = (-2.0 * far * near) / (far - near);

        result[0] = column0Row0;
        result[1] = 0.0;
        result[2] = 0.0;
        result[3] = 0.0;
        result[4] = 0.0;
        result[5] = column1Row1;
        result[6] = 0.0;
        result[7] = 0.0;
        result[8] = column2Row0;
        result[9] = column2Row1;
        result[10] = column2Row2;
        result[11] = column2Row3;
        result[12] = 0.0;
        result[13] = 0.0;
        result[14] = column3Row2;
        result[15] = 0.0;
        return DMat4::from_cols_array(&result);
    }
    fn from_raw_list(slice: [f64; 16]) -> DMat4 {
        return make_matrix4_from_raw(slice);
    }
    fn compute_orthographic_off_center(
        left: f64,
        right: f64,
        bottom: f64,
        top: f64,
        near: f64,
        far: f64,
    ) -> DMat4 {
        let mut matrix: [f64; 16] = [0.; 16];
        let mut a = 1.0 / (right - left);
        let mut b = 1.0 / (top - bottom);
        let mut c = 1.0 / (far - near);

        let tx = -(right + left) * a;
        let ty = -(top + bottom) * b;
        let tz = -(far + near) * c;
        a *= 2.0;
        b *= 2.0;
        c *= -2.0;

        matrix[0] = a;
        matrix[1] = 0.0;
        matrix[2] = 0.0;
        matrix[3] = 0.0;
        matrix[4] = 0.0;
        matrix[5] = b;
        matrix[6] = 0.0;
        matrix[7] = 0.0;
        matrix[8] = 0.0;
        matrix[9] = 0.0;
        matrix[10] = c;
        matrix[11] = 0.0;
        matrix[12] = tx;
        matrix[13] = ty;
        matrix[14] = tz;
        matrix[15] = 1.0;
        return DMat4::from_cols_array(&matrix);
    }
    fn set_translation(&mut self, cartesian: &DVec3) {
        self.x_axis.w = cartesian.x;
        self.y_axis.w = cartesian.y;
        self.z_axis.w = cartesian.z;
    }
    fn get_translation(&self) -> DVec3 {
        DVec3 {
            x: self.x_axis.w,
            y: self.y_axis.w,
            z: self.z_axis.w,
        }
    }
    fn multiply_by_point_as_vector(&self, cartesian: &DVec3) -> DVec3 {
        let mut matrix: [f64; 16] = [
            0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
        ];
        self.write_cols_to_slice(&mut matrix);
        let vX = cartesian.x;
        let vY = cartesian.y;
        let vZ = cartesian.z;

        let x = matrix[0] * vX + matrix[4] * vY + matrix[8] * vZ;
        let y = matrix[1] * vX + matrix[5] * vY + matrix[9] * vZ;
        let z = matrix[2] * vX + matrix[6] * vY + matrix[10] * vZ;

        return DVec3::new(x, y, z);
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

        let vX = cartesian.x;
        let vY = cartesian.y;
        let vZ = cartesian.z;

        let x = slice[0] * vX + slice[4] * vY + slice[8] * vZ + slice[12];
        let y = slice[1] * vX + slice[5] * vY + slice[9] * vZ + slice[13];
        let z = slice[2] * vX + slice[6] * vY + slice[10] * vZ + slice[14];
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
    fn to_mat4_32(&self) -> Mat4 {
        to_mat4_32(self)
    }
}
pub fn to_mat4_64(mat4: &Mat4) -> DMat4 {
    let mut matrix: [f32; 16] = [
        0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    ];
    mat4.write_cols_to_slice(&mut matrix);
    let mut new_matrix: [f64; 16] = [0.; 16];
    matrix
        .iter()
        .enumerate()
        .for_each(|(i, x)| new_matrix[i] = x.clone() as f64);
    return DMat4::from_cols_array(&new_matrix);
}
pub fn to_mat4_32(mat4: &DMat4) -> Mat4 {
    let mut matrix: [f64; 16] = [
        0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    ];
    mat4.write_cols_to_slice(&mut matrix);
    let mut new_matrix: [f32; 16] = [0.; 16];
    matrix
        .iter()
        .enumerate()
        .for_each(|(i, x)| new_matrix[i] = x.clone() as f32);
    return Mat4::from_cols_array(&new_matrix);
}
