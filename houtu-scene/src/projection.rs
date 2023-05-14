use bevy::{math::DVec3, prelude::*};

use crate::{ellipsoid::Ellipsoid, math::Cartographic};

pub trait Projection {
    type Output;
    fn project(&self, coord: &Cartographic) -> DVec3;
    fn un_project(&self, vec: &DVec3) -> Cartographic;
    fn from_ellipsoid(ellipsoid: &Ellipsoid) -> Self::Output;
}
