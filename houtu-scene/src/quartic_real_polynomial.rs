use crate::{CubicRealPolynomial, QuadraticRealPolynomial, EPSILON14, EPSILON15};

pub struct QuarticRealPolynomial;
impl QuarticRealPolynomial {
    // pub fn computeRealRoots(a: f64, b: f64, c: f64, d: f64, e: f64) -> f64 {
    //     let a2 = a * a;
    //     let a3 = a2 * a;
    //     let b2 = b * b;
    //     let b3 = b2 * b;
    //     let c2 = c * c;
    //     let c3 = c2 * c;
    //     let d2 = d * d;
    //     let d3 = d2 * d;
    //     let e2 = e * e;
    //     let e3 = e2 * e;

    //     let discriminant = b2 * c2 * d2 - 4.0 * b3 * d3 - 4.0 * a * c3 * d2 + 18. * a * b * c * d3
    //         - 27.0 * a2 * d2 * d2
    //         + 256.0 * a3 * e3
    //         + e * (18.0 * b3 * c * d - 4.0 * b2 * c3 + 16.0 * a * c2 * c2
    //             - 80.0 * a * b * c2 * d
    //             - 6.0 * a * b2 * d2
    //             + 144.0 * a2 * c * d2)
    //         + e2 * (144.0 * a * b2 * c - 27.0 * b2 * b2 - 128.0 * a2 * c2 - 192.0 * a2 * b * d);
    //     return discriminant;
    // }
    pub fn computeRealRoots(a: f64, b: f64, c: f64, d: f64, e: f64) -> Option<Vec<f64>> {
        if (a.abs() < EPSILON15) {
            return CubicRealPolynomial::computeRealRoots(b, c, d, e);
        }
        let a3 = b / a;
        let a2 = c / a;
        let a1 = d / a;
        let a0 = e / a;

        let mut k = if a3 < 0.0 { 1 } else { 0 };
        k += if a2 < 0.0 { k + 1 } else { k };
        k += if a1 < 0.0 { k + 1 } else { k };
        k += if a0 < 0.0 { k + 1 } else { k };

        match k {
            0 => return Some(original(a3, a2, a1, a0)),
            1 => return Some(neumark(a3, a2, a1, a0)),
            2 => return Some(neumark(a3, a2, a1, a0)),
            3 => return Some(original(a3, a2, a1, a0)),
            4 => return Some(original(a3, a2, a1, a0)),
            5 => return Some(neumark(a3, a2, a1, a0)),
            6 => return Some(original(a3, a2, a1, a0)),
            7 => return Some(original(a3, a2, a1, a0)),
            8 => return Some(neumark(a3, a2, a1, a0)),
            9 => return Some(original(a3, a2, a1, a0)),
            10 => return Some(original(a3, a2, a1, a0)),
            11 => return Some(neumark(a3, a2, a1, a0)),
            12 => return Some(original(a3, a2, a1, a0)),
            13 => return Some(original(a3, a2, a1, a0)),
            14 => return Some(original(a3, a2, a1, a0)),
            15 => return Some(original(a3, a2, a1, a0)),
            _ => return None,
        }
    }
}

fn original(a3: f64, a2: f64, a1: f64, a0: f64) -> Vec<f64> {
    let a3Squared = a3 * a3;

    let p = a2 - (3.0 * a3Squared) / 8.0;
    let q = a1 - (a2 * a3) / 2.0 + (a3Squared * a3) / 8.0;
    let r = a0 - (a1 * a3) / 4.0 + (a2 * a3Squared) / 16.0 - (3.0 * a3Squared * a3Squared) / 256.0;

    // Find the roots of the cubic equations:  h^6 + 2 p h^4 + (p^2 - 4 r) h^2 - q^2 = 0.
    let cubicRoots =
        CubicRealPolynomial::computeRealRoots(1.0, 2.0 * p, p * p - 4.0 * r, -q * q).unwrap();

    if (cubicRoots.len() > 0) {
        let temp = -a3 / 4.0;

        // Use the largest positive root.
        let hSquared = cubicRoots[cubicRoots.len() - 1];

        if (hSquared.abs() < EPSILON14) {
            // y^4 + p y^2 + r = 0.
            let roots = QuadraticRealPolynomial::computeRealRoots(1.0, p, r).unwrap();

            if (roots.len() == 2) {
                let root0 = roots[0];
                let root1 = roots[1];

                let y;
                if (root0 >= 0.0 && root1 >= 0.0) {
                    let y0 = root0.sqrt();
                    let y1 = root1.sqrt();

                    return vec![temp - y1, temp - y0, temp + y0, temp + y1];
                } else if (root0 >= 0.0 && root1 < 0.0) {
                    y = root0.sqrt();
                    return vec![temp - y, temp + y];
                } else if (root0 < 0.0 && root1 >= 0.0) {
                    y = root1.sqrt();
                    return vec![temp - y, temp + y];
                }
            }
            return vec![];
        } else if (hSquared > 0.0) {
            let h = hSquared.sqrt();

            let m = (p + hSquared - q / h) / 2.0;
            let n = (p + hSquared + q / h) / 2.0;

            // Now solve the two quadratic factors:  (y^2 + h y + m)(y^2 - h y + n);
            let mut roots1 = QuadraticRealPolynomial::computeRealRoots(1.0, h, m).unwrap();
            let mut roots2 = QuadraticRealPolynomial::computeRealRoots(1.0, -h, n).unwrap();

            if (roots1.len() != 0) {
                roots1[0] += temp;
                roots1[1] += temp;

                if (roots2.len() != 0) {
                    roots2[0] += temp;
                    roots2[1] += temp;

                    if (roots1[1] <= roots2[0]) {
                        return vec![roots1[0], roots1[1], roots2[0], roots2[1]];
                    } else if (roots2[1] <= roots1[0]) {
                        return vec![roots2[0], roots2[1], roots1[0], roots1[1]];
                    } else if (roots1[0] >= roots2[0] && roots1[1] <= roots2[1]) {
                        return vec![roots2[0], roots1[0], roots1[1], roots2[1]];
                    } else if (roots2[0] >= roots1[0] && roots2[1] <= roots1[1]) {
                        return vec![roots1[0], roots2[0], roots2[1], roots1[1]];
                    } else if (roots1[0] > roots2[0] && roots1[0] < roots2[1]) {
                        return vec![roots2[0], roots1[0], roots2[1], roots1[1]];
                    }
                    return vec![roots1[0], roots2[0], roots1[1], roots2[1]];
                }
                return roots1;
            }

            if (roots2.len() != 0) {
                roots2[0] += temp;
                roots2[1] += temp;

                return roots2;
            }
            return vec![];
        }
    }
    return vec![];
}

fn neumark(a3: f64, a2: f64, a1: f64, a0: f64) -> Vec<f64> {
    let a1Squared = a1 * a1;
    let a2Squared = a2 * a2;
    let a3Squared = a3 * a3;

    let p = -2.0 * a2;
    let q = a1 * a3 + a2Squared - 4.0 * a0;
    let r = a3Squared * a0 - a1 * a2 * a3 + a1Squared;

    let cubicRoots = CubicRealPolynomial::computeRealRoots(1.0, p, q, r).unwrap();

    if (cubicRoots.len() > 0) {
        // Use the most positive root
        let y = cubicRoots[0];

        let temp = a2 - y;
        let tempSquared = temp * temp;

        let g1 = a3 / 2.0;
        let h1 = temp / 2.0;

        let m = tempSquared - 4.0 * a0;
        let mError = tempSquared + 4.0 * a0.abs();

        let n = a3Squared - 4.0 * y;
        let nError = a3Squared + 4.0 * y.abs();

        let mut g2;
        let mut h2;

        if (y < 0.0 || m * nError < n * mError) {
            let squareRootOfN = n.sqrt();
            g2 = squareRootOfN / 2.0;
            h2 = if squareRootOfN == 0.0 {
                0.0
            } else {
                (a3 * h1 - a1) / squareRootOfN
            };
        } else {
            let squareRootOfM = m.sqrt();
            g2 = if squareRootOfM == 0.0 {
                0.0
            } else {
                (a3 * h1 - a1) / squareRootOfM
            };
            h2 = squareRootOfM / 2.0;
        }

        let mut G;
        let mut g;
        if (g1 == 0.0 && g2 == 0.0) {
            G = 0.0;
            g = 0.0;
        } else if (g1.signum() == g2.signum()) {
            G = g1 + g2;
            g = y / G;
        } else {
            g = g1 - g2;
            G = y / g;
        }

        let mut H;
        let mut h;
        if (h1 == 0.0 && h2 == 0.0) {
            H = 0.0;
            h = 0.0;
        } else if (h1.signum() == h2.signum()) {
            H = h1 + h2;
            h = a0 / H;
        } else {
            h = h1 - h2;
            H = a0 / h;
        }

        // Now solve the two quadratic factors:  (y^2 + G y + H)(y^2 + g y + h);
        let roots1 = QuadraticRealPolynomial::computeRealRoots(1.0, G, H).unwrap();
        let roots2 = QuadraticRealPolynomial::computeRealRoots(1.0, g, h).unwrap();

        if (roots1.len() != 0) {
            if (roots2.len() != 0) {
                if (roots1[1] <= roots2[0]) {
                    return vec![roots1[0], roots1[1], roots2[0], roots2[1]];
                } else if (roots2[1] <= roots1[0]) {
                    return vec![roots2[0], roots2[1], roots1[0], roots1[1]];
                } else if (roots1[0] >= roots2[0] && roots1[1] <= roots2[1]) {
                    return vec![roots2[0], roots1[0], roots1[1], roots2[1]];
                } else if (roots2[0] >= roots1[0] && roots2[1] <= roots1[1]) {
                    return vec![roots1[0], roots2[0], roots2[1], roots1[1]];
                } else if (roots1[0] > roots2[0] && roots1[0] < roots2[1]) {
                    return vec![roots2[0], roots1[0], roots2[1], roots1[1]];
                }
                return vec![roots1[0], roots2[0], roots1[1], roots2[1]];
            }
            return roots1;
        }
        if (roots2.len() != 0) {
            return roots2;
        }
    }
    return vec![];
}
