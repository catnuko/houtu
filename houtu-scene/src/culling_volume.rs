use bevy::math::DVec4;

use crate::{BoundingVolume, Intersect, Plane};

#[derive(Debug, Default, Clone)]
pub struct CullingVolume {
    pub planes: [DVec4; 6],
}
impl CullingVolume {
    pub fn new(planes: Option<[DVec4; 6]>) -> Self {
        Self {
            planes: planes.unwrap_or([DVec4::ZERO; 6]),
        }
    }
    pub fn get_planes(&self) -> &[DVec4; 6] {
        return &self.planes;
    }
    pub fn from_cartesian4() {}
    pub fn computeVisibility(&self, bounding_volume: &Box<&dyn BoundingVolume>) -> Intersect {
        let mut intersecting = false;
        for plane in self.planes.iter() {
            let result = bounding_volume.intersect_plane(&Plane::from_vec4(&plane));
            if result == Intersect::OUTSIDE {
                return Intersect::OUTSIDE;
            } else if result == Intersect::INTERSECTING {
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
