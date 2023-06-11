use flo_curves::{Coordinate, Coordinate3D, BezierCurve};

use std::collections::HashMap;
use std::ops::{Add, Sub, Mul, Div};
use smallvec::SmallVec;

use amethyst::{
    core::{
        math::{Unit, UnitQuaternion, Vector3, Point3},
        transform::{Transform, TransformBundle},
        Time,
    },
    ecs::prelude::*,
};

use log::warn;

#[derive(PartialEq, Copy, Clone)]
pub struct Coord3 (pub f64, pub f64, pub f64);

impl Coord3 {
    pub fn normalize(self) -> Coord3 {
        self / f64::sqrt(self.0*self.0 + self.1*self.1 + self.2*self.2)
    }

    pub fn cross(self, rhs: &Coord3) -> Coord3 {
        Coord3(
            self.1*rhs.2 - self.2*rhs.1,
            self.2*rhs.0 - self.0*rhs.2,
            self.0*rhs.1 - self.1*rhs.0,
        )
    }
}

impl Into<Point3<f32>> for Coord3 {
    fn into(self) -> Point3<f32> {
        Point3::new(self.0 as f32, self.1  as f32, self.2  as f32)
    }
}
impl Into<Vector3<f32>> for Coord3 {
     fn into(self) -> Vector3<f32> {
        Vector3::new(self.0 as f32, self.1  as f32, self.2  as f32)
    }
}


impl Coordinate3D for Coord3 {
    ///
    /// X component of this coordinate
    /// 
    #[inline]
    fn x(&self) -> f64 {
        self.0
    }

    ///
    /// Y component of this coordinate
    /// 
    #[inline]
    fn y(&self) -> f64 {
        self.1
    }

    ///
    /// Z component of this coordinate
    /// 
    #[inline]
    fn z(&self) -> f64 {
        self.2
    }
    
}

impl Add<Coord3> for Coord3 {
    type Output=Coord3;

    #[inline]
    fn add(self, rhs: Coord3) -> Coord3 {
        Coord3(self.0 + rhs.0, self.1 + rhs.1, self.2 + rhs.2)
    }
}

impl Sub<Coord3> for Coord3 {
    type Output=Coord3;

    #[inline]
    fn sub(self, rhs: Coord3) -> Coord3 {
        Coord3(self.0 - rhs.0, self.1 - rhs.1, self.2 - rhs.2)
    }
}

impl Mul<f64> for Coord3 {
    type Output=Coord3;

    #[inline]
    fn mul(self, rhs: f64) -> Coord3 {
        Coord3(self.0 * rhs, self.1 * rhs, self.2 * rhs)
    }
}
impl Div<f64> for Coord3 {
    type Output=Coord3;

    #[inline]
    fn div(self, rhs: f64) -> Coord3 {
        Coord3(self.0 / rhs, self.1 / rhs, self.2 / rhs)
    }
}

impl Coordinate for Coord3 {
    #[inline]
    fn from_components(components: &[f64]) -> Coord3 {
        Coord3(components[0], components[1], components[2])
    }

    #[inline]
    fn origin() -> Coord3 {
        Coord3(0.0, 0.0, 0.0)
    }

    #[inline]
    fn len() -> usize { 3 }

    #[inline]
    fn get(&self, index: usize) -> f64 { 
        match index {
            0 => self.0,
            1 => self.1,
            2 => self.2,
            _ => panic!("Coord3 only has three components")
        }
    }

    fn from_biggest_components(p1: Coord3, p2: Coord3) -> Coord3 {
        Coord3(f64::from_biggest_components(p1.0, p2.0), f64::from_biggest_components(p1.1, p2.1), f64::from_biggest_components(p1.2, p2.2))
    }

    fn from_smallest_components(p1: Coord3, p2: Coord3) -> Coord3 {
        Coord3(f64::from_smallest_components(p1.0, p2.0), f64::from_smallest_components(p1.1, p2.1), f64::from_smallest_components(p1.2, p2.2))
    }

    #[inline]
    fn distance_to(&self, target: &Coord3) -> f64 {
        let dist_x = target.0-self.0;
        let dist_y = target.1-self.1;
        let dist_z = target.2-self.2;

        f64::sqrt(dist_x*dist_x + dist_y*dist_y + dist_z*dist_z)
    }

    #[inline]
    fn dot(&self, target: &Self) -> f64 {
        self.0*target.0 + self.1*target.1 + self.2*target.2
    }
}


pub struct Overfly {
    pub curves: Vec<flo_curves::bezier::Curve<Coord3>>,
    pub time_scale: f64,
}

impl Component for Overfly {
    type Storage = DenseVecStorage<Self>;
}

#[derive(Default)]
pub struct OverflySystem {
    entites: HashMap<Entity, f64>,
}

impl<'a> System<'a> for OverflySystem {
    type SystemData = (
        Entities<'a>,
        Read<'a, Time>,
        ReadStorage<'a, Overfly>,
        WriteStorage<'a, Transform>,
    );

    fn run(&mut self, (entities, time, overflies, mut transforms): Self::SystemData) {
        for (e, overfly, mut transform) in (&*entities, &overflies, &mut transforms).join() {
            let up : Vector3<f32> = Vector3::y();
            let ct = self.entites.entry(e).or_insert(0.0);

            let index = ct.floor() as usize % overfly.curves.len();
            let curve = overfly.curves[index];

            let t = *ct - ct.floor();
            let control_points = curve.control_points();
            let position = curve.point_at_pos(t);


            // Calculate Frenet Normal
            let dx = flo_curves::bezier::derivative4(curve.start_point(), control_points.0, control_points.1, curve.end_point());
            let dxx = flo_curves::bezier::derivative3(dx.0, dx.1, dx.2);
            let a = flo_curves::bezier::de_casteljau3(t, dx.0, dx.1, dx.2).normalize();

            let t = (position + a).normalize();
            let tangent: Vector3<f32> = a.into();
            let b = t.cross(&position).normalize();
            let bitangent: Vector3<f32> = b.into();

            let normal: Vector3<f32> = b.cross(&position).normalize().into();

            transform.set_translation(position.into());
            transform.face_towards((position + a).into(), up);
            let axis = Unit::new_normalize(tangent);
            let angle = up.dot(&tangent);
            let rotation = UnitQuaternion::from_axis_angle(&axis, angle);
            // transform.set_rotation(rotation);

            *ct += time.fixed_seconds() as f64 * overfly.time_scale;
        }
    }
}