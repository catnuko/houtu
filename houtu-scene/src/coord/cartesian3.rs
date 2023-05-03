use std::ops::Add;

pub struct Cartesian3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Default for Cartesian3 {
    fn default() -> Self {
        return Cartesian3::ZERO;
    }
}
impl Cartesian3 {
    // pub fn fromSpherical(spherical: Spherical) -> Self {
    //     let x = spherical.radius * spherical.sinTheta * spherical.cosPhi;
    //     let y = spherical.radius * spherical.sinTheta * spherical.sinPhi;
    //     let z = spherical.radius * spherical.cosTheta;
    //     return Cartesian3::new(x, y, z);
    // }
    const ZERO: Cartesian3 = Cartesian3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Cartesian3 { x, y, z }
    }
    pub fn from_vec3(vec3: DVec3) -> Self {
        Cartesian3 {
            x: vec3.x,
            y: vec3.y,
            z: vec3.z,
        }
    }
}
impl Add for Cartesian3 {
    type Output = Cartesian3;

    fn add(self, other: Cartesian3) -> Cartesian3 {
        Cartesian3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}
