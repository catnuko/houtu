use crate::QuadraticRealPolynomial;

pub struct CubicRealPolynomial;
impl CubicRealPolynomial {
    pub fn computeRealRoots(a: f64, b: f64, c: f64, d: f64) -> Option<Vec<f64>> {
        let roots;
        let ratio;
        if (a == 0.0) {
            // Quadratic function: b * x^2 + c * x + d = 0.
            return QuadraticRealPolynomial::computeRealRoots(b, c, d);
        } else if (b == 0.0) {
            if (c == 0.0) {
                if (d == 0.0) {
                    // 3rd order monomial: a * x^3 = 0.
                    return Some(vec![0.0, 0.0, 0.0]);
                }

                // a * x^3 + d = 0
                ratio = -d / a;
                let root = if ratio < 0.0 {
                    -(-ratio).powf(1.0 / 3.0)
                } else {
                    ratio.powf(1.0 / 3.0)
                };
                return Some(vec![root, root, root]);
            } else if (d == 0.0) {
                // x * (a * x^2 + c) = 0.
                roots = QuadraticRealPolynomial::computeRealRoots(a, 0., c);

                // Return the roots in ascending order.
                if roots.is_some() {
                    let roots = roots.unwrap();
                    if roots.len() == 0 {
                        return Some(vec![0.0]);
                    } else {
                        return Some(vec![roots[0], 0.0, roots[1]]);
                    }
                } else {
                    return None;
                }
            }

            // Deflated cubic polynomial: a * x^3 + c * x + d= 0.
            return computeRealRoots(a, 0., c, d);
        } else if (c == 0.0) {
            if (d == 0.0) {
                // x^2 * (a * x + b) = 0.
                ratio = -b / a;
                if (ratio < 0.0) {
                    return Some(vec![ratio, 0.0, 0.0]);
                }
                return Some(vec![0.0, 0.0, ratio]);
            }
            // a * x^3 + b * x^2 + d = 0.
            return computeRealRoots(a, b, 0., d);
        } else if (d == 0.0) {
            // x * (a * x^2 + b * x + c) = 0
            roots = QuadraticRealPolynomial::computeRealRoots(a, b, c);
            if roots.is_some() {
                let roots = roots.unwrap();
                if (roots.len() == 0) {
                    return Some(vec![0.0]);
                } else if (roots[1] <= 0.0) {
                    return Some(vec![roots[0], roots[1], 0.0]);
                } else if (roots[0] >= 0.0) {
                    return Some(vec![0.0, roots[0], roots[1]]);
                }
                return Some(vec![roots[0], 0.0, roots[1]]);
            } else {
                return None;
            }
        }

        return computeRealRoots(a, b, c, d);
    }
}

fn computeRealRoots(a: f64, b: f64, c: f64, d: f64) -> Option<Vec<f64>> {
    let A = a;
    let B = b / 3.0;
    let C = c / 3.0;
    let D = d;

    let AC = A * C;
    let BD = B * D;
    let B2 = B * B;
    let C2 = C * C;
    let delta1 = A * C - B2;
    let delta2 = A * D - B * C;
    let delta3 = B * D - C2;

    let discriminant = 4.0 * delta1 * delta3 - delta2 * delta2;
    let mut temp;
    let mut temp1;

    if (discriminant < 0.0) {
        let ABar;
        let CBar;
        let DBar;

        if (B2 * BD >= AC * C2) {
            ABar = A;
            CBar = delta1;
            DBar = -2.0 * B * delta1 + A * delta2;
        } else {
            ABar = D;
            CBar = delta3;
            DBar = -D * delta2 + 2.0 * C * delta3;
        }

        let s = if DBar < 0.0 { -1.0 } else { 1.0 }; // This is not Math.Sign()!
        let temp0 = -s * ABar.abs() * (-discriminant).sqrt();
        temp1 = -DBar + temp0;

        let x = temp1 / 2.0;
        let p = if x < 0.0 {
            -(-x).powf(1.0 / 3.0)
        } else {
            x.powf(1.0 / 3.0)
        };
        let q = if temp1 == temp0 { -p } else { -CBar / p };

        temp = if CBar <= 0.0 {
            p + q
        } else {
            -DBar / (p * p + q * q + CBar)
        };

        if (B2 * BD >= AC * C2) {
            return Some(vec![(temp - B) / A]);
        }

        return Some(vec![-D / (temp + C)]);
    }

    let CBarA = delta1;
    let DBarA = -2.0 * B * delta1 + A * delta2;

    let CBarD = delta3;
    let DBarD = -D * delta2 + 2.0 * C * delta3;

    let squareRootOfDiscriminant = discriminant.sqrt();
    let halfSquareRootOf3 = (3.0 as f64).sqrt() / 2.0;
    let a: f64 = 3.0;
    let mut theta = ((A * squareRootOfDiscriminant).atan2(-DBarA) / 3.0).abs();
    temp = 2.0 * (-CBarA).sqrt();
    let mut cosine = theta.cos();
    temp1 = temp * cosine;
    let mut temp3 = temp * (-cosine / 2.0 - halfSquareRootOf3 * theta.sin());

    let numeratorLarge = if temp1 + temp3 > 2.0 * B {
        temp1 - B
    } else {
        temp3 - B
    };
    let denominatorLarge = A;

    let root1 = numeratorLarge / denominatorLarge;

    theta = ((D * squareRootOfDiscriminant).atan2(-DBarD) / 3.0).abs();
    temp = 2.0 * (-CBarD).sqrt();
    cosine = theta.cos();
    temp1 = temp * cosine;
    temp3 = temp * (-cosine / 2.0 - halfSquareRootOf3 * theta.sin());

    let numeratorSmall = -D;
    let denominatorSmall = if temp1 + temp3 < 2.0 * C {
        temp1 + C
    } else {
        temp3 + C
    };

    let root3 = numeratorSmall / denominatorSmall;

    let E = denominatorLarge * denominatorSmall;
    let F = -numeratorLarge * denominatorSmall - denominatorLarge * numeratorSmall;
    let G = numeratorLarge * numeratorSmall;

    let root2 = (C * F - B * G) / (-B * F + C * E);

    if (root1 <= root2) {
        if (root1 <= root3) {
            if (root2 <= root3) {
                return Some(vec![root1, root2, root3]);
            }
            return Some(vec![root1, root3, root2]);
        }
        return Some(vec![root3, root1, root2]);
    }
    if (root1 <= root3) {
        return Some(vec![root2, root1, root3]);
    }
    if (root2 <= root3) {
        return Some(vec![root2, root3, root1]);
    }
    return Some(vec![root3, root2, root1]);
}
