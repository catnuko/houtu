use std::f32::consts::{PI, TAU};
use std::fmt;

use crate::coord::Cartesian3;
use crate::math;
// use bevy::math::Cartesian3;
use bevy::prelude::Mesh;
use bevy::render::mesh::Indices;
use wgpu::PrimitiveTopology;

// mod ellipsoid_plugin;
// mod ellipsoid_shape;

pub struct Ellipsoid {
    pub radii: Cartesian3,
    pub radiiSquared: Cartesian3,
    pub radiiToTheFourth: Cartesian3,
    pub oneOverRadii: Cartesian3,
    pub oneOverRadiiSquared: Cartesian3,
    pub minimumRadius: f64,
    pub maximumRadius: f64,
    pub centerToleranceSquared: f64,
    pub squaredXOverSquaredZ: f64,
}
impl Default for Ellipsoid {
    fn default() -> Self {
        let radii = Cartesian3::ZERO;
        return Ellipsoid::from_vec3(radii);
    }
}
impl Clone for Ellipsoid {
    fn clone(&self) -> Self {
        return Ellipsoid::from_vec3(self.radii);
    }
}
impl Ellipsoid {
    pub fn from_vec3(radii: Cartesian3) -> Self {
        return Ellipsoid::new(radii.x, radii.y, radii.z);
    }
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        let radii = Cartesian3::new(x, y, z);
        let radiiSquared = Cartesian3::new(x * x, y * y, z * z);
        let radiiToTheFourth = Cartesian3::new(x * x * x * x, y * y * y * y, z * z * z * z);
        let oneOverRadii = Cartesian3::new(1.0 / x, 1.0 / y, 1.0 / z);
        let oneOverRadiiSquared = Cartesian3::new(1.0 / (x * x), 1.0 / (y * y), 1.0 / (z * z));
        let minimumRadius = x.min(y).min(z);
        let maximumRadius = x.max(y).max(z);
        let centerToleranceSquared = math::EPSILON1 as f64;
        let mut squaredXOverSquaredZ = 0.0;
        if (radiiSquared.z != 0) {
            squaredXOverSquaredZ = radiiSquared.x / radiiSquared.z;
        }
        Ellipsoid {
            radii,
            radiiSquared,
            radiiToTheFourth,
            oneOverRadii,
            oneOverRadiiSquared,
            minimumRadius,
            maximumRadius,
            centerToleranceSquared,
            squaredXOverSquaredZ,
        }
    }
    pub const WGS84: Ellipsoid = Ellipsoid::new(6378137.0, 6378137.0, 6356752.3142451793);
    pub const UNIT_SPHERE: Ellipsoid = Ellipsoid::new(1.0, 1.0, 1.0);
    pub const MOON: Ellipsoid =
        Ellipsoid::new(math::LUNAR_RADIUS, math::LUNAR_RADIUS, math::LUNAR_RADIUS);
    pub fn geocentricSurfaceNormal(vec3: Cartesian3) -> Cartesian3 {
        return vec3.normalize();
    }
    pub fn geodeticSurfaceNormalCartographic(&self, cartographic: Cartesian3) -> Cartesian3 {
        let longitude = cartographic.x;
        let latitude = cartographic.y;
        let cosLatitude = latitude.cos();
        let x = cosLatitude * longitude.cos();
        let y = cosLatitude * longitude.sin();
        let z = latitude.sin();
        return Cartesian3::new(x, y, z).normalize();
    }
    pub fn geodeticSurfaceNormal(&self, vec3: Cartesian3) -> Option<Cartesian3> {
        if vec3.abs_diff_eq(Cartesian3::ZERO, math::EPSILON14) {
            return None;
        }
        let x = vec3.x;
        let y = vec3.y;
        let z = vec3.z;
        Some(vec3 / self.oneOverRadiiSquared.normalize())
    }
    pub fn scaleToGeodeticSurface(&self, vec3: Cartesian3) -> Option<Cartesian3> {
        let mut position = vec3;
        let mut positionX2 = position.x * position.x;
        let mut positionY2 = position.y * position.y;
        let mut positionZ2 = position.z * position.z;
        let mut oneOverRadiiSquared = self.oneOverRadiiSquared;
        let mut x2OverRadiiSquared = positionX2 * oneOverRadiiSquared.x;
        let mut y2OverRadiiSquared = positionY2 * oneOverRadiiSquared.y;
        let mut z2OverRadiiSquared = positionZ2 * oneOverRadiiSquared.z;
        let mut sum = x2OverRadiiSquared + y2OverRadiiSquared + z2OverRadiiSquared;
        let mut a = 1.0;
        let mut b = 0.5;
        let mut c = 0.0625;
        let mut d = 0.03125;
        let mut x2y2 = positionX2 + positionY2;
        let mut x2z2 = positionX2 + positionZ2;
        let mut y2z2 = positionY2 + positionZ2;
        let mut alpha = 1.0
            - (b * y2z2 + c * z2OverRadiiSquared) * position.y * position.y * oneOverRadiiSquared.y;
        let mut beta =
            (b * x2z2 + c * z2OverRadiiSquared) * position.x * position.x * oneOverRadiiSquared.x;
        let mut gamma =
            (b * x2y2 + c * y2OverRadiiSquared) * position.z * position.z * oneOverRadiiSquared.z;
        let mut delta = 2.0
            * (a * (x2OverRadiiSquared + y2OverRadiiSquared + z2OverRadiiSquared)
                + beta * position.x * position.y
                + gamma * position.z * position.z
                - c * oneOverRadiiSquared.z * position.z * position.z * position.z * position.z)
            - 1.0;
        let mut dRoot = (b * b - 4.0 * a * c) * delta;
        if (dRoot < 0.0) {
            return None;
        }
        dRoot = dRoot.sqrt();
        let mut n = 0.5 * (-b * delta - dRoot);
        let mut term = n / a;
        let mut root = 0.0;
        let mut summand = 0.0;
        if (term > 0.0) {
            root = term.sqrt();
            position.z = oneOverRadiiSquared.z * position.z - oneOverRadiiSquared.z * c * root;
            summand = z2OverRadiiSquared + position.z * position.z;
            sum = x2OverRadiiSquared / (summand * summand)
                + y2OverRadiiSquared / (summand * summand)
                + position.z * position.z * oneOverRadiiSquared.z * oneOverRadiiSquared.z;
            n = position.x * position.x / sum;
            term = n / a;
            if (term > 0.0) {
                root = term.sqrt();
                position.x = oneOverRadiiSquared.x * position.x - oneOverRadiiSquared.x * root;
            } else {
                return None;
            }
            term = n / b;
            if (term > 0.0) {
                root = term.sqrt();
                position.y = oneOverRadiiSquared.y * position.y - oneOverRadiiSquared.y * root;
            } else {
                return None;
            }
        } else {
            return None;
        }
        return Some(position);
    }
    pub fn cartographicToCartesian(&self, cartographic: Cartesian3) -> Cartesian3 {
        let mut n = Cartesian3::ZERO;
        let mut k = Cartesian3::ZERO;
        n = self.geodeticSurfaceNormalCartographic(cartographic);
        k = self.radiiSquared * n;
        let mut gamma = n.dot(k).sqrt();
        k = k / gamma;
        n = n * cartographic.z;
        return k + n;
    }
    pub fn cartesianToCartographic(&self, vec3: Cartesian3) -> Option<Cartesian3> {
        if let Some(mut p) = self.scaleToGeodeticSurface(vec3) {
            if let Some(mut n) = self.geodeticSurfaceNormal(p) {
                let mut h = vec3 - p;
                let mut longitude = n.y.atan2(n.x);
                let mut latitude = n.z.asin();
                let mut height = (n.dot(vec3)).sin() * h.length();
                return Some(Cartesian3::new(longitude, latitude, height));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}
fn negate(vec: Cartesian3) -> Cartesian3 {
    Cartesian3::new(-vec.x, -vec.y, -vec.z)
}

#[cfg(test)]
mod tests {
    use super::*;
    const spaceCartesian: Cartesian3 =
        Cartesian3::new(4582719.8827300891, -4582719.8827300882, 1725510.4250797231);

    const spaceCartographic: Cartesian3 = Cartesian3::new(
        (-45.0 as f64).to_radians(),
        (15.0 as f64).to_radians(),
        330000.0,
    );
    #[test]
    fn default() {
        let ellipsoid = Ellipsoid::default();
        assert_eq!(ellipsoid.radii, Cartesian3::ZERO);
        assert_eq!(ellipsoid.radiiSquared, Cartesian3::ZERO);
        assert_eq!(ellipsoid.radiiToTheFourth, Cartesian3::ZERO);
        assert_eq!(ellipsoid.oneOverRadii, Cartesian3::ZERO);
        assert_eq!(ellipsoid.oneOverRadiiSquared, Cartesian3::ZERO);
        assert_eq!(ellipsoid.minimumRadius, 0.0);
        assert_eq!(ellipsoid.maximumRadius, 0.0);
    }
    // #[test]
    // fn cartographicToCartesian_work() {
    //     let ellipsoid = Ellipsoid::WGS84;
    //     let cartographic = Cartesian3::new(0.0, 0.0, 0.0);
    //     let result = ellipsoid.cartographicToCartesian(spaceCartographic);
    //     assert_eq!(result - spaceCartesian, math::EPSILON7);
    // }
}
