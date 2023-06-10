use bevy::math::DVec3;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Plane {
    pub normal: DVec3,
    pub distance: f64,
}
impl Plane {
    pub fn fromPointNormal(point: &DVec3, normal: &DVec3) -> Self {
        let distance = -normal.dot(*point);
        Self {
            normal: normal.clone(),
            distance,
        }
    }
    pub fn getPointDistance(&self, point: DVec3) -> f64 {
        return self.normal.dot(point) + self.distance;
    }
    pub fn projectPointOntoPlane(&self, point: DVec3) -> DVec3 {
        return point - self.normal * self.getPointDistance(point);
    }
}
