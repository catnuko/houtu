pub mod epsilon;
pub mod vec3;
use bevy::{ecs::system::Command, prelude::*};

pub struct EigenDecompositionResult {
    pub unitary: Mat3,
    pub diagonal: Mat3,
}
pub fn computeEigenDecomposition(matrix: Mat3) -> EigenDecompositionResult {
    let tolerance = epsilon::EPSILON20;
    let maxSweeps = 10;

    let mut count = 0;
    let mut sweep = 0;

    let mut unitaryMatrix = Mat3::IDENTITY;
    let mut diagMatrix = matrix.clone();

    let epsilon = tolerance * computeFrobeniusNorm(diagMatrix);

    while (sweep < maxSweeps && offDiagonalFrobeniusNorm(diagMatrix) > epsilon) {
        let jMatrix = shurDecomposition(diagMatrix);
        let jMatrixTranspose = jMatrix.transpose();
        diagMatrix = diagMatrix * jMatrix;
        diagMatrix = jMatrixTranspose * diagMatrix;
        unitaryMatrix = unitaryMatrix * jMatrix;

        count += 1;
        if (count > 2) {
            sweep += 1;
            count = 0;
        }
    }
    return EigenDecompositionResult {
        unitary: unitaryMatrix,
        diagonal: diagMatrix,
    };
}
pub fn computeFrobeniusNorm(matrix: Mat3) -> f32 {
    let mut slice: [f32; 9] = [0., 0., 0., 0., 0., 0., 0., 0., 0.];
    matrix.write_cols_to_slice(&mut slice);
    let mut norm = 0.0;
    for i in 0..9 {
        let temp = slice[i];
        norm += temp * temp;
    }
    return norm.sqrt();
}

pub fn offDiagonalFrobeniusNorm(matrix: Mat3) -> f32 {
    let rowVal = [1, 0, 0];
    let colVal = [2, 2, 1];
    let mut slice: [f32; 9] = [0., 0., 0., 0., 0., 0., 0., 0., 0.];
    matrix.write_cols_to_slice(&mut slice);

    let mut norm = 0.0;
    for i in 0..3 {
        let temp = slice[getElementIndex(colVal[i], rowVal[i])];
        norm += 2.0 * temp * temp;
    }

    return norm.sqrt();
}
pub fn getElementIndex(row: usize, col: usize) -> usize {
    return row + 3 * col;
}
pub fn shurDecomposition(matrix: Mat3) -> Mat3 {
    let rowVal = [1, 0, 0];
    let colVal = [2, 2, 1];

    let tolerance = epsilon::EPSILON15;
    // let mut slice: [f32; 16] = [
    //     0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    // ];
    let mut slice: [f32; 9] = [0., 0., 0., 0., 0., 0., 0., 0., 0.];
    matrix.write_cols_to_slice(&mut slice);
    let mut maxDiagonal = 0.0;
    let mut rotAxis = 1;

    // find pivot (rotAxis) based on max diagonal of matrix
    for i in 0..3 {
        let temp = slice[getElementIndex(i, i)].abs();
        if temp > maxDiagonal {
            rotAxis = i;
            maxDiagonal = temp;
        }
    }
    let mut c = 1.0;
    let mut s = 0.0;

    let p = rowVal[rotAxis];
    let q = colVal[rotAxis];

    if ((slice[getElementIndex(q, p)]).abs() > tolerance) {
        let qq = slice[getElementIndex(q, q)];
        let pp = slice[getElementIndex(p, p)];
        let qp = slice[getElementIndex(q, p)];

        let tau = (qq - pp) / 2.0 / qp;
        let t;

        if (tau < 0.0) {
            t = -1.0 / (-tau + (1.0 + tau * tau).sqrt());
        } else {
            t = 1.0 / (tau + (1.0 + tau * tau).sqrt());
        }

        c = 1.0 / (1.0 + t * t).sqrt();
        s = t * c;
    }
    let mut slice: [f32; 9] = [0., 0., 0., 0., 0., 0., 0., 0., 0.];

    slice[getElementIndex(p, p)] = c;
    slice[getElementIndex(q, q)] = c;
    slice[getElementIndex(q, p)] = s;
    slice[getElementIndex(p, q)] = -s;
    return Mat3::from_cols_array(&slice);
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_work() {
        let a = Mat3::from_cols_array(&vec3::to_col_major(&[
            4.0, -1.0, 1.0, -1.0, 3.0, -2.0, 1.0, -2.0, 3.0,
        ]));
        let expectedDiagonal = Mat3::from_cols_array(&vec3::to_col_major(&[
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
