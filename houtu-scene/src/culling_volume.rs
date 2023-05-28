use crate::{BoundingSphere, BoundingVolume, Intersect, Plane};

#[derive(Debug, Default)]
pub struct CullingVolume {
    pub planes: Vec<Plane>,
}
impl CullingVolume {
    pub fn new(planes: Option<Vec<Plane>>) -> Self {
        Self {
            planes: planes.unwrap_or(Vec::new()),
        }
    }
    pub fn from_cartesian4() {}
    pub fn computeVisibility(&self, boundingVolume: &Box<&dyn BoundingVolume>) -> Intersect {
        let mut intersecting = false;
        for plane in self.planes.iter() {
            let result = boundingVolume.intersect_plane(&plane);
            if (result == Intersect::OUTSIDE) {
                return Intersect::OUTSIDE;
            } else if (result == Intersect::INTERSECTING) {
                intersecting = true;
            }
        }
        return {
            if intersecting {
                Intersect::INTERSECTING
            } else {
                Intersect::INSIDE
            }
        };
    }
}
