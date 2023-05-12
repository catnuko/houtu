use bevy::math::DVec3;

use crate::math::Cartesian3;
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Ray {
    pub origin: DVec3,
    pub direction: DVec3,
}
impl Ray {
    pub fn new(origin: DVec3, direction: DVec3) -> Self {
        Self { origin, direction }
    }
    pub fn getPoint(&self, t: f64) -> DVec3 {
        let temp = self.direction.multiply_by_scalar(t);
        return self.origin + temp;
    }
}
