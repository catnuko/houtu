use bevy::prelude::Vec3;
use geodesy::{Coord, Ellipsoid};

use crate::projection::Projection;

pub struct GeographicProjection {
    ellipsoid: Ellipsoid,
    semimajor_axis: f64,
    one_over_semimajor_axis: f64,
}
impl Default for GeographicProjection {
    fn default() -> Self {
        let e = Ellipsoid::named("WGS84");
        Self {
            ellipsoid: e,
            semimajor_axis: e.semimajor_axis(),
            one_over_semimajor_axis: 1.0 / e.semimajor_axis(),
        }
    }
}
impl Projection for GeographicProjection {
    fn project(&self, coord: Coord) -> Vec3 {
        let semimajorAxis = self.semimajor_axis;
        let x = coord.first() * semimajorAxis;
        let y = coord.second() * semimajorAxis;
        let z = coord.third();
        Vec3::new(x, y, z)
    }
    fn un_project(&self, vec: Vec3) -> Coord {
        let one_over_semimajor_axis = self.one_over_semimajor_axis;
        let x = vec.x * one_over_semimajor_axis;
        let y = vec.y * one_over_semimajor_axis;
        let z = vec.z;
        Coord::gis(x, y, z, 0.)
    }
}
