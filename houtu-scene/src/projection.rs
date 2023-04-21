use bevy::prelude::*;
use geodesy::preamble::*;

pub trait Projection {
    fn project(&self, coord: Coord) -> Vec3;
    fn un_project(&self, vec: Vec3) -> Coord;
    fn from_ellipsoid(&self, ellipsoid: Ellipsoid) -> Self {
        Self {
            ellipsoid: ellipsoid,
            semimajor_axis: ellipsoid.semimajor_axis(),
            one_over_semimajor_axis: 1.0 / ellipsoid.semimajor_axis(),
        }
    }
}
