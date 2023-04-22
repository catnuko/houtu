use bevy::prelude::{Mat3, Vec3};

pub fn multiplyByScalar(m: Mat3, v: Vec3) -> Mat3 {
    Mat3::from_cols(m.x_axis * v.x, m.y_axis * v.y, m.z_axis * v.z)
}
// pub fn equalEpsilon(m: Mat3, e: f32) -> bool {
//     return m.abs_diff_eq(rhs, max_abs_diff)
// }
pub fn to_col_major(v: &[f32; 9]) -> [f32; 9] {
    let mut result = [0.0; 9];
    result[0] = v[0];
    result[1] = v[3];
    result[2] = v[6];
    result[3] = v[1];
    result[4] = v[4];
    result[5] = v[7];
    result[6] = v[2];
    result[7] = v[5];
    result[8] = v[8];
    return result;
}
