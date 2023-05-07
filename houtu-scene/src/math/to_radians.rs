use std::f64::consts::PI;

pub trait ToRadians {
    fn to_radians(&self) -> f64;
}
impl ToRadians for f64 {
    fn to_radians(&self) -> f64 {
        self * (PI / 180.0)
    }
}
