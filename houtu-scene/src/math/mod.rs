mod epsilon;
mod to_radians;
mod vec3;
use std::f64::consts::{PI, TAU};

use bevy::{
    ecs::system::Command,
    math::DMat3,
    // math::{DMat3, DVec3},
    prelude::*,
};
mod cartesian3;
mod cartographic;
mod heading_pitch_roll;
mod matrix4;
mod transform;
pub use cartesian3::*;
mod quaternion;
pub use cartographic::*;
pub use heading_pitch_roll::*;
pub use matrix4::*;
pub use quaternion::*;
pub use transform::*;

pub use epsilon::*;
pub use to_radians::*;
pub use vec3::*;

pub struct EigenDecompositionResult {
    pub unitary: DMat3,
    pub diagonal: DMat3,
}
pub fn computeEigenDecomposition(matrix: DMat3) -> EigenDecompositionResult {
    let tolerance = epsilon::EPSILON20;
    let maxSweeps = 10;

    let mut count = 0;
    let mut sweep = 0;

    let mut unitaryMatrix = DMat3::IDENTITY;
    let mut unitaryMatrix = DMat3::IDENTITY;
    let mut diagMatrix = matrix.clone();

    let epsilon = tolerance * computeFrobeniusNorm(diagMatrix);

    while sweep < maxSweeps && offDiagonalFrobeniusNorm(diagMatrix) > epsilon {
        let jMatrix = shurDecomposition(diagMatrix);
        let jMatrixTranspose = jMatrix.transpose();
        diagMatrix = diagMatrix * jMatrix;
        diagMatrix = jMatrixTranspose * diagMatrix;
        unitaryMatrix = unitaryMatrix * jMatrix;

        count += 1;
        if count > 2 {
            sweep += 1;
            count = 0;
        }
    }
    return EigenDecompositionResult {
        unitary: unitaryMatrix,
        diagonal: diagMatrix,
    };
}
pub fn computeFrobeniusNorm(matrix: DMat3) -> f64 {
    let mut slice: [f64; 9] = [0., 0., 0., 0., 0., 0., 0., 0., 0.];
    matrix.write_cols_to_slice(&mut slice);
    let mut norm = 0.0;
    for i in 0..9 {
        let temp = slice[i];
        norm += temp * temp;
    }
    return norm.sqrt();
}
pub fn offDiagonalFrobeniusNorm(matrix: DMat3) -> f64 {
    let rowVal = [1, 0, 0];
    let colVal = [2, 2, 1];
    let mut slice: [f64; 9] = [0., 0., 0., 0., 0., 0., 0., 0., 0.];
    let mut slice: [f64; 9] = [0., 0., 0., 0., 0., 0., 0., 0., 0.];
    matrix.write_cols_to_slice(&mut slice);

    let mut norm = 0.0;
    for i in 0..3 {
        let temp = slice[getElementIndex(colVal[i], rowVal[i])];
        norm += 2.0 * temp * temp;
    }

    return norm.sqrt();
}
pub fn getElementIndex(col: usize, row: usize) -> usize {
    return row + 3 * col;
}
pub fn shurDecomposition(matrix: DMat3) -> DMat3 {
    let rowVal = [1, 0, 0];
    let colVal = [2, 2, 1];

    let tolerance = epsilon::EPSILON15;
    // let mut slice: [f64; 16] = [
    // let mut slice: [f64; 16] = [
    //     0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    // ];
    let mut slice: [f64; 9] = [0., 0., 0., 0., 0., 0., 0., 0., 0.];
    let mut slice: [f64; 9] = [0., 0., 0., 0., 0., 0., 0., 0., 0.];
    matrix.write_cols_to_slice(&mut slice);
    let mut maxDiagonal = 0.0;
    let mut rotAxis = 1;

    // find pivot (rotAxis) based on max diagonal of matrix
    for i in 0..3 {
        let temp = slice[getElementIndex(colVal[i], rowVal[i])].abs();
        if temp > maxDiagonal {
            rotAxis = i;
            maxDiagonal = temp;
        }
    }
    let mut c = 1.0;
    let mut s = 0.0;

    let p = rowVal[rotAxis];
    let q = colVal[rotAxis];
    let dif = slice[getElementIndex(q, p)].abs();
    if dif > tolerance {
        let qq = slice[getElementIndex(q, q)];
        let pp = slice[getElementIndex(p, p)];
        let qp = slice[getElementIndex(q, p)];

        let tau = (qq - pp) / 2.0 / qp;
        let t;

        if tau < 0.0 {
            t = -1.0 / (-tau + (1.0 + tau * tau).sqrt());
        } else {
            t = 1.0 / (tau + (1.0 + tau * tau).sqrt());
        }

        c = 1.0 / (1.0 + t * t).sqrt();
        s = t * c;
    }
    let mut slice2: [f64; 9] = [0.; 9];
    DMat3::IDENTITY.write_cols_to_slice(&mut slice2);

    slice2[getElementIndex(p, p)] = c;
    slice2[getElementIndex(q, q)] = c;
    slice2[getElementIndex(q, p)] = s;
    slice2[getElementIndex(p, q)] = -s;
    return DMat3::from_cols_array(&slice2);
}
pub fn nagetive_pi_to_pi(angle: f64) -> f64 {
    if angle >= -PI && angle <= PI {
        return angle;
    }
    return zero_to_two_pi(angle + PI) - PI;
}
pub fn zero_to_two_pi(angle: f64) -> f64 {
    if angle >= 0. && angle <= TAU {
        return angle;
    }
    let mode = angle % TAU;
    if mode.abs() < epsilon::EPSILON14 && angle.abs() > epsilon::EPSILON14 {
        return TAU;
    }
    return mode;
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_work() {
        let a = DMat3::from_cols_array(&vec3::to_col_major(&[
            4.0, -1.0, 1.0, -1.0, 3.0, -2.0, 1.0, -2.0, 3.0,
        ]));
        let expectedDiagonal = DMat3::from_cols_array(&vec3::to_col_major(&[
            3.0, 0.0, 0.0, 0.0, 6.0, 0.0, 0.0, 0.0, 1.0,
        ]));
        let decomposition = computeEigenDecomposition(a);
        assert_eq!(
            decomposition
                .diagonal
                .abs_diff_eq(expectedDiagonal, epsilon::EPSILON14),
            true
        );
    }
}
