use bevy::math::{DMat3, DVec3};

pub fn multiplyByScalar(m: DMat3, v: DVec3) -> DMat3 {
    DMat3::from_cols(m.x_axis * v.x, m.y_axis * v.y, m.z_axis * v.z)
}
// pub fn equalEpsilon(m: DMat3, e: f64) -> bool {
//     return m.abs_diff_eq(rhs, max_abs_diff)
// }
pub fn to_col_major(v: &[f64; 9]) -> [f64; 9] {
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
pub fn equals_epsilon(
    left: f64,
    right: f64,
    relative_epsilon: Option<f64>,
    absolute_epsilon: Option<f64>,
) -> bool {
    let relative_epsilon = relative_epsilon.unwrap_or(0.0);
    let absolute_epsilon = absolute_epsilon.unwrap_or(relative_epsilon);
    let diff = (left - right).abs();
    return diff <= absolute_epsilon || diff <= relative_epsilon * left.abs();
}
pub fn less_than(left: f64, right: f64, absolute_epsilon: f64) -> bool {
    return right - left < -absolute_epsilon;
}
pub fn less_than_equals(left: f64, right: f64, absolute_epsilon: f64) -> bool {
    return right - left < absolute_epsilon;
}
pub fn greater_than(left: f64, right: f64, absolute_epsilon: f64) -> bool {
    return left - right > absolute_epsilon;
}
pub fn greater_than_equals(left: f64, right: f64, absolute_epsilon: f64) -> bool {
    return left - right > -absolute_epsilon;
}

pub fn factorial(n: f64) -> f64 {
    let mut factorials: Vec<f64> = vec![1.];
    let mut func = |n: f64| -> f64 {
        if n >= factorials.len() as f64 {
            let mut sum = factorials[factorials.len() - 1];
            for i in factorials.len()..n as usize {
                sum *= i as f64;
                factorials.push(sum);
            }
            return sum;
        } else {
            return factorials[n as usize];
        }
    };
    return func(n);
}
