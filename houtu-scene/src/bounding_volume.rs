use crate::{Intersect, Plane};

pub trait BoundingVolume {
    fn intersect_plane(&self, plane: &Plane) -> Intersect;
}
