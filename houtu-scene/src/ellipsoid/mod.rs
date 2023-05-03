use std::f32::consts::{PI, TAU};
use std::fmt;

use bevy::math::DVec3;
use bevy::prelude::Mesh;
use bevy::render::mesh::Indices;
use wgpu::PrimitiveTopology;

// mod ellipsoid_plugin;
// mod ellipsoid_shape;

pub struct Ellipsoid {
    pub radii: DVec3,
    pub radiiSquared: DVec3,
    pub radiiToTheFourth: DVec3,
    pub oneOverRadii: DVec3,
    pub oneOverRadiiSquared: DVec3,
    pub minimumRadius: f64,
    pub maximumRadius: f64,
    pub centerToleranceSquared: f64,
    pub squaredXOverSquaredZ: f64,
}
impl Default for Ellipsoid {
    fn default() -> Self {
        let radii = DVec3::ZERO;
        return Ellipsoid::from_vec3(radii);
    }
}
impl Clone for Ellipsoid {
    fn clone(&self) -> Self {
        return Ellipsoid::from_vec3(self.radii);
    }
}
impl Ellipsoid {
    pub fn from_vec3(radii: DVec3) -> Self {
        return Ellipsoid::new(radii.x, radii.y, radii.z);
    }
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        let radii = DVec3::new(x, y, z);
        let radiiSquared = DVec3::new(x * x, y * y, z * z);
        let radiiToTheFourth = DVec3::new(x * x * x * x, y * y * y * y, z * z * z * z);
        let oneOverRadii = DVec3::new(1.0 / x, 1.0 / y, 1.0 / z);
        let oneOverRadiiSquared = DVec3::new(1.0 / (x * x), 1.0 / (y * y), 1.0 / (z * z));
        let minimumRadius = x.min(y).min(z);
        let maximumRadius = x.max(y).max(z);
        let centerToleranceSquared = houtu_math::epsilon::EPSILON1 as f64;
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
    const WGS84: Ellipsoid = Ellipsoid::new(6378137.0, 6378137.0, 6356752.3142451793);
    const UNIT_SPHERE: Ellipsoid = Ellipsoid::new(1.0, 1.0, 1.0);
    const MOON: Ellipsoid = Ellipsoid::new(
        houtu_math::epsilon::LUNAR_RADIUS,
        houtu_math::epsilon::LUNAR_RADIUS,
        houtu_math::epsilon::LUNAR_RADIUS,
    );
    pub fn geocentricSurfaceNormal(vec3: DVec3) -> DVec3 {
        return vec3.normalize();
    }
    pub fn geodeticSurfaceNormalCartographic(&self, cartographic: DVec3) -> DVec3 {
        let longitude = cartographic.x;
        let latitude = cartographic.y;
        let cosLatitude = latitude.cos();
        let x = cosLatitude * longitude.cos();
        let y = cosLatitude * longitude.sin();
        let z = latitude.sin();
        return DVec3::new(x, y, z).normalize();
    }
    pub fn geodeticSurfaceNormal(&self, vec3: DVec3) -> Option<DVec3> {
        if vec3.abs_diff_eq(DVec3::ZERO, houtu_math::epsilon::EPSILON14) {
            return None;
        }
        let x = vec3.x;
        let y = vec3.y;
        let z = vec3.z;
        Some(vec3 / self.oneOverRadiiSquared.normalize())
    }
    pub fn scaleToGeodeticSurface(&self, vec3: DVec3) -> Option<DVec3> {
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
    pub fn cartographicToCartesian(&self, cartographic: DVec3) -> DVec3 {
        let mut n = DVec3::ZERO;
        let mut k = DVec3::ZERO;
        n = self.geodeticSurfaceNormalCartographic(cartographic);
        k = self.radiiSquared * n;
        let mut gamma = n.dot(k).sqrt();
        k = k / gamma;
        n = n * cartographic.z;
        return k + n;
    }
    pub fn cartesianToCartographic(&self, vec3: DVec3) -> Option<DVec3> {
        if let Some(mut p) = self.scaleToGeodeticSurface(vec3) {
            if let Some(mut n) = self.geodeticSurfaceNormal(p) {
                let mut h = vec3 - p;
                let mut longitude = n.y.atan2(n.x);
                let mut latitude = n.z.asin();
                let mut height = (n.dot(vec3)).sin() * h.length();
                return Some(DVec3::new(longitude, latitude, height));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
}
fn negate(vec: DVec3) -> DVec3 {
    DVec3::new(-vec.x, -vec.y, -vec.z)
}

#[cfg(test)]
mod tests {
    use super::*;
    const spaceCartesian: DVec3 =
        DVec3::new(4582719.8827300891, -4582719.8827300882, 1725510.4250797231);

    const spaceCartographic: DVec3 = DVec3::new(-45.0.to_radians(), 15.0.to_radians(), 330000.0);
    #[test]
    fn default() {
        let ellipsoid = Ellipsoid::default();
        assert_eq!(ellipsoid.radii, DVec3::ZERO);
        assert_eq!(ellipsoid.radiiSquared, DVec3::ZERO);
        assert_eq!(ellipsoid.radiiToTheFourth, DVec3::ZERO);
        assert_eq!(ellipsoid.oneOverRadii, DVec3::ZERO);
        assert_eq!(ellipsoid.oneOverRadiiSquared, DVec3::ZERO);
        assert_eq!(ellipsoid.minimumRadius, 0.0);
        assert_eq!(ellipsoid.maximumRadius, 0.0);
    }
    // #[test]
    // fn cartographicToCartesian_work() {
    //     let ellipsoid = Ellipsoid::WGS84;
    //     let cartographic = DVec3::new(0.0, 0.0, 0.0);
    //     let result = ellipsoid.cartographicToCartesian(spaceCartographic);
    //     assert_eq!(result - spaceCartesian, houtu_math::epsilon::EPSILON7);
    // }
}
