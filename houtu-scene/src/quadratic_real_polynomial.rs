use crate::EPSILON14;

pub struct QuadraticRealPolynomial;
impl QuadraticRealPolynomial {
    pub fn compute_real_roots(a: f64, b: f64, c: f64) -> Option<Vec<f64>> {
        let ratio;
        if a == 0.0 {
            if b == 0.0 {
                // letant function: c = 0.
                return Some(vec![]);
            }

            // Linear function: b * x + c = 0.
            return Some(vec![-c / b]);
        } else if b == 0.0 {
            if c == 0.0 {
                // 2nd order monomial: a * x^2 = 0.
                return Some(vec![0.0, 0.0]);
            }

            let c_magnitude = c.abs();
            let a_magnitude = a.abs();

            if c_magnitude < a_magnitude && c_magnitude / a_magnitude < EPSILON14 {
                // c ~= 0.0.
                // 2nd order monomial: a * x^2 = 0.
                return Some(vec![0.0, 0.0]);
            } else if c_magnitude > a_magnitude && a_magnitude / c_magnitude < EPSILON14 {
                // a ~= 0.0.
                // letant function: c = 0.
                return Some(vec![]);
            }

            // a * x^2 + c = 0
            ratio = -c / a;

            if ratio < 0.0 {
                // Both roots are complex.
                return Some(vec![]);
            }

            // Both roots are real.
            let root = ratio.sqrt();
            return Some(vec![-root, root]);
        } else if c == 0.0 {
            // a * x^2 + b * x = 0
            ratio = -b / a;
            if ratio < 0.0 {
                return Some(vec![ratio, 0.0]);
            }

            return Some(vec![0.0, ratio]);
        } else {
            return None;
        }
    }
}
