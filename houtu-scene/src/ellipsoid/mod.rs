use std::f32::consts::{PI, TAU};
use std::fmt;

use crate::coord::Cartesian3;
use crate::math;
// use bevy::math::Cartesian3;
use bevy::prelude::Mesh;
use bevy::render::mesh::Indices;
use geodesy::Ellipsoid as GeodesyEllipsoid;
use wgpu::PrimitiveTopology;

// mod ellipsoid_plugin;
mod ellipsoid_shape;
pub use ellipsoid_shape::EllipsoidShape;

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
        if radiiSquared.z != 0. {
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
    pub const WGS84: Ellipsoid = Ellipsoid {
        radii: Cartesian3 {
            x: 6378137.0,
            y: 6378137.0,
            z: 6356752.3142451793,
        },
        radiiSquared: Cartesian3 {
            x: 40680631590769.0,
            y: 40680631590769.0,
            z: 40408299984661.445,
        },
        radiiToTheFourth: Cartesian3 {
            x: 1.6549137866238727e+27,
            y: 1.6549137866238727e+27,
            z: 1.63283070765039e+27,
        },
        oneOverRadii: Cartesian3 {
            x: 1.567855942887398e-7,
            y: 1.567855942887398e-7,
            z: 1.573130351105623e-7,
        },
        oneOverRadiiSquared: Cartesian3 {
            x: 2.458172257647332e-14,
            y: 2.458172257647332e-14,
            z: 2.4747391015697002e-14,
        },
        minimumRadius: 6356752.3142451793,
        maximumRadius: 6378137.0,
        centerToleranceSquared: 0.1,
        squaredXOverSquaredZ: 1.0067394967422765,
    };
    pub const UNIT_SPHERE: Ellipsoid = Ellipsoid {
        radii: Cartesian3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        radiiSquared: Cartesian3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        radiiToTheFourth: Cartesian3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        oneOverRadii: Cartesian3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        oneOverRadiiSquared: Cartesian3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        minimumRadius: 1.0,
        maximumRadius: 1.0,
        centerToleranceSquared: 0.1,
        squaredXOverSquaredZ: 1.0,
    };
    pub const MOON: Ellipsoid = Ellipsoid {
        radii: Cartesian3 {
            x: 1737400.0,
            y: 1737400.0,
            z: 1737400.0,
        },
        radiiSquared: Cartesian3 {
            x: 3018558760000.,
            y: 3018558760000.,
            z: 3018558760000.,
        },
        radiiToTheFourth: Cartesian3 {
            x: 9.111696987572739e+24,
            y: 9.111696987572739e+24,
            z: 9.111696987572739e+24,
        },
        oneOverRadii: Cartesian3 {
            x: 5.755726948313572e-7,
            y: 5.755726948313572e-7,
            z: 5.755726948313572e-7,
        },
        oneOverRadiiSquared: Cartesian3 {
            x: 3.3128392703543064e-13,
            y: 3.3128392703543064e-13,
            z: 3.3128392703543064e-13,
        },
        minimumRadius: 1737400.0,
        maximumRadius: 1737400.0,
        centerToleranceSquared: 0.1,
        squaredXOverSquaredZ: 1.0,
    };
    pub fn geocentricSurfaceNormal(vec3: Cartesian3) -> Cartesian3 {
        return vec3.normalize();
    }
    pub fn semimajor_axis(&self) -> f64 {
        return self.radii.x;
    }
    pub fn semiminor_axis(&self) -> f64 {
        return self.radii.z;
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
    pub fn geodeticSurfaceNormal(&self, vec3: &Cartesian3) -> Option<Cartesian3> {
        if vec3.equals_epsilon(&Cartesian3::ZERO, Some(math::EPSILON14), None) {
            return None;
        }
        let x = vec3.x;
        let y = vec3.y;
        let z = vec3.z;
        Some(*vec3 / &self.oneOverRadiiSquared.normalize())
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
        if dRoot < 0.0 {
            return None;
        }
        dRoot = dRoot.sqrt();
        let mut n = 0.5 * (-b * delta - dRoot);
        let mut term = n / a;
        let mut root = 0.0;
        let mut summand = 0.0;
        if term > 0.0 {
            root = term.sqrt();
            position.z = oneOverRadiiSquared.z * position.z - oneOverRadiiSquared.z * c * root;
            summand = z2OverRadiiSquared + position.z * position.z;
            sum = x2OverRadiiSquared / (summand * summand)
                + y2OverRadiiSquared / (summand * summand)
                + position.z * position.z * oneOverRadiiSquared.z * oneOverRadiiSquared.z;
            n = position.x * position.x / sum;
            term = n / a;
            if term > 0.0 {
                root = term.sqrt();
                position.x = oneOverRadiiSquared.x * position.x - oneOverRadiiSquared.x * root;
            } else {
                return None;
            }
            term = n / b;
            if term > 0.0 {
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
        k = self.radiiSquared * &n;
        let gamma = n.dot(&k).sqrt();
        k = k / gamma;
        n = n * cartographic.z;
        return k + &n;
    }
    pub fn cartesianToCartographic(&self, vec3: Cartesian3) -> Option<Cartesian3> {
        if let Some(p) = self.scaleToGeodeticSurface(vec3) {
            if let Some(n) = self.geodeticSurfaceNormal(&p) {
                let h = vec3 - &p;
                let longitude = n.y.atan2(n.x);
                let latitude = n.z.asin();
                let height = (n.dot(&vec3)).sin() * h.magnitude();
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
    const spaceCartesian: Cartesian3 = Cartesian3 {
        x: 4582719.8827300891,
        y: -4582719.8827300882,
        z: 1725510.4250797231,
    };

    const spaceCartographic: Cartesian3 = Cartesian3 {
        x: -0.7853981633974483,
        y: 0.2617993877991494,
        z: 330000.0,
    };
    #[test]
    fn default() {
        let ellipsoid = Ellipsoid::default();
        // assert_eq!(ellipsoid.radii, Cartesian3::ZERO);
        // assert_eq!(ellipsoid.radiiSquared, Cartesian3::ZERO);
        // assert_eq!(ellipsoid.radiiToTheFourth, Cartesian3::ZERO);
        // assert_eq!(ellipsoid.oneOverRadii, Cartesian3::ZERO);
        // assert_eq!(ellipsoid.oneOverRadiiSquared, Cartesian3::ZERO);
        // assert_eq!(ellipsoid.minimumRadius, 0.0);
        // assert_eq!(ellipsoid.maximumRadius, 0.0);
    }
    // #[test]
    // fn cartographicToCartesian_work() {
    //     let ellipsoid = Ellipsoid::WGS84;
    //     let cartographic = Cartesian3::new(0.0, 0.0, 0.0);
    //     let result = ellipsoid.cartographicToCartesian(spaceCartographic);
    //     assert_eq!(result - spaceCartesian, math::EPSILON7);
    // }
}
