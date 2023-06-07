use std::f64::consts::PI;

pub trait ToRadians {
    fn to_radians(&self) -> f64;
    fn get_mod(&self, n: f64) -> f64;
}
impl ToRadians for f64 {
    fn to_radians(&self) -> f64 {
        self * (PI / 180.0)
    }
    fn get_mod(&self, n: f64) -> f64 {
        if self.signum() == n.signum() && self.abs() < n.abs() {
            return self.clone();
        }
        return ((self % n) + n) % n;
    }
}
