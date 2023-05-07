use std::f32::consts::{PI, TAU};
use std::fmt;
use std::ops::Sub;

use crate::coord::{Cartesian3, Cartographic};
use crate::math;
use crate::math::ToRadians;
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
    pub fn geocentricSurfaceNormal(vec3: &Cartesian3) -> Cartesian3 {
        return vec3.normalize();
    }
    pub fn semimajor_axis(&self) -> f64 {
        return self.radii.x;
    }
    pub fn semiminor_axis(&self) -> f64 {
        return self.radii.z;
    }
    pub fn geodeticSurfaceNormalCartographic(&self, cartographic: &Cartographic) -> Cartesian3 {
        let longitude = cartographic.longitude;
        let latitude = cartographic.latitude;
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
        Some(
            vec3.multiply_components(&self.oneOverRadiiSquared)
                .normalize(),
        )
    }
    pub fn scaleToGeodeticSurface(&self, cartesian: &Cartesian3) -> Option<Cartesian3> {
        let oneOverRadii = self.oneOverRadii;
        let oneOverRadiiSquared = self.oneOverRadiiSquared;
        let centerToleranceSquared = self.centerToleranceSquared;
        let positionX = cartesian.x;
        let positionY = cartesian.y;
        let positionZ = cartesian.z;
        let oneOverRadiiX = oneOverRadii.x;
        let oneOverRadiiY = oneOverRadii.y;
        let oneOverRadiiZ = oneOverRadii.z;

        let x2 = positionX * positionX * oneOverRadiiX * oneOverRadiiX;
        let y2 = positionY * positionY * oneOverRadiiY * oneOverRadiiY;
        let z2 = positionZ * positionZ * oneOverRadiiZ * oneOverRadiiZ;
        let mut squaredNorm = x2 + y2 + z2;

        // Compute the squared ellipsoid norm.

        let ratio = (1.0 / squaredNorm).sqrt();

        // As an initial approximation, assume that the radial intersection is the projection point.
        let intersection = cartesian.multiply_by_scalar(ratio);
        // If the position is near the center, the iteration will not converge.
        if (squaredNorm < centerToleranceSquared) {
            if !ratio.is_infinite() {
                return None;
            } else {
                return Some(intersection);
            }
        }

        let oneOverRadiiSquaredX = oneOverRadiiSquared.x;
        let oneOverRadiiSquaredY = oneOverRadiiSquared.y;
        let oneOverRadiiSquaredZ = oneOverRadiiSquared.z;

        // Use the gradient at the intersection point in place of the true unit normal.
        // The difference in magnitude will be absorbed in the multiplier.
        let mut gradient = intersection;
        gradient.x = intersection.x * oneOverRadiiSquaredX * 2.0;
        gradient.y = intersection.y * oneOverRadiiSquaredY * 2.0;
        gradient.z = intersection.z * oneOverRadiiSquaredZ * 2.0;

        // Compute the initial guess at the normal vector multiplier, lambda.
        let mut lambda = ((1.0 - ratio) * cartesian.magnitude()) / (0.5 * gradient.magnitude());
        let mut correction = 0.0;

        let mut func: f64 = 0.;
        let mut denominator: f64 = 0.;
        let mut xMultiplier: f64 = 0.;
        let mut yMultiplier: f64 = 0.;
        let mut zMultiplier: f64 = 0.;
        let mut xMultiplier2: f64 = 0.;
        let mut yMultiplier2: f64 = 0.;
        let mut zMultiplier2: f64 = 0.;
        let mut xMultiplier3: f64 = 0.;
        let mut yMultiplier3: f64 = 0.;
        let mut zMultiplier3: f64 = 0.;
        loop {
            lambda -= correction;

            xMultiplier = 1.0 / (1.0 + lambda * oneOverRadiiSquaredX);
            yMultiplier = 1.0 / (1.0 + lambda * oneOverRadiiSquaredY);
            zMultiplier = 1.0 / (1.0 + lambda * oneOverRadiiSquaredZ);

            xMultiplier2 = xMultiplier * xMultiplier;
            yMultiplier2 = yMultiplier * yMultiplier;
            zMultiplier2 = zMultiplier * zMultiplier;

            xMultiplier3 = xMultiplier2 * xMultiplier;
            yMultiplier3 = yMultiplier2 * yMultiplier;
            zMultiplier3 = zMultiplier2 * zMultiplier;

            func = x2 * xMultiplier2 + y2 * yMultiplier2 + z2 * zMultiplier2 - 1.0;

            // "denominator" here refers to the use of this expression in the velocity and acceleration
            // computations in the sections to follow.
            denominator = x2 * xMultiplier3 * oneOverRadiiSquaredX
                + y2 * yMultiplier3 * oneOverRadiiSquaredY
                + z2 * zMultiplier3 * oneOverRadiiSquaredZ;

            let derivative = -2.0 * denominator;

            correction = func / derivative;
            if func.abs() < math::EPSILON12 {
                break;
            }
        }

        return Some(Cartesian3::new(
            positionX * xMultiplier,
            positionY * yMultiplier,
            positionZ * zMultiplier,
        ));
    }
    pub fn cartographicToCartesian(&self, cartographic: &Cartographic) -> Cartesian3 {
        let mut n = Cartesian3::ZERO;
        let mut k = Cartesian3::ZERO;
        n = self.geodeticSurfaceNormalCartographic(cartographic);
        k = self.radiiSquared * &n;
        let gamma = n.dot(&k).sqrt();
        k = k / gamma;
        n = n * cartographic.height;
        return k + &n;
    }
    pub fn cartesianToCartographic(&self, vec3: &Cartesian3) -> Option<Cartographic> {
        if let Some(p) = self.scaleToGeodeticSurface(vec3) {
            if let Some(n) = self.geodeticSurfaceNormal(&p) {
                let h = vec3.subtract(&p);
                let longitude = n.y.atan2(n.x);
                let latitude = n.z.asin();
                let b = h.dot(&vec3);
                let c = b.signum();
                let d = c * h.magnitude();
                let height = d;
                return Some(Cartographic::from_radians(longitude, latitude, height));
            } else {
                return None;
            }
        } else {
            return None;
        }
    }
    pub fn cartographicArrayToCartesianArray(
        &self,
        cartographics: Vec<&Cartographic>,
    ) -> Vec<Cartesian3> {
        let mut result = Vec::with_capacity(cartographics.len());
        for cartographic in cartographics {
            result.push(self.cartographicToCartesian(cartographic));
        }
        return result;
    }
    pub fn cartesianArrayToCartographicArray(
        &self,
        cartesians: Vec<&Cartesian3>,
    ) -> Vec<Cartographic> {
        let mut result = Vec::with_capacity(cartesians.len());
        for cartesian in cartesians {
            if let Some(cartographic) = self.cartesianToCartographic(cartesian) {
                result.push(cartographic);
            }
        }
        return result;
    }
}
fn negate(vec: Cartesian3) -> Cartesian3 {
    Cartesian3::new(-vec.x, -vec.y, -vec.z)
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::*;
    const spaceCartesian: Cartesian3 = Cartesian3 {
        x: 4582719.8827300891,
        y: -4582719.8827300882,
        z: 1725510.4250797231,
    };

    const spaceCartographic: Cartographic = Cartographic {
        longitude: -0.7853981633974483,
        latitude: 0.2617993877991494,
        height: 330000.0,
    };
    const surfaceCartographic: Cartographic = Cartographic {
        longitude: 25.0 * PI / 180.0,
        latitude: 45.0 * PI / 180.0,
        height: 0.0,
    };

    const surfaceCartesian: Cartesian3 = Cartesian3 {
        x: 4094327.7921465295,
        y: 1909216.4044747739,
        z: 4487348.4088659193,
    };
    #[test]
    fn test_cartographicToCartesian() {
        let ellipsoid = Ellipsoid::WGS84;
        let cartographic = Cartesian3::new(0.0, 0.0, 0.0);
        let result = ellipsoid.cartographicToCartesian(&spaceCartographic);
        assert_eq!(
            result.equals_epsilon(&spaceCartesian, Some(math::EPSILON7), None),
            true
        );
    }
    #[test]
    fn test_cartographicArrayToCartesianArray() {
        let ellipsoid = Ellipsoid::WGS84;
        let cartographic = Cartesian3::new(0.0, 0.0, 0.0);
        let result = ellipsoid
            .cartographicArrayToCartesianArray(vec![&spaceCartographic, &surfaceCartographic]);

        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].equals_epsilon(&spaceCartesian, Some(math::EPSILON7), None),
            true
        );
        assert_eq!(
            result[1].equals_epsilon(&surfaceCartesian, Some(math::EPSILON7), None),
            true
        );
    }
    #[test]
    fn test_cartesianToCartographic() {
        let ellipsoid = Ellipsoid::WGS84;
        let result = ellipsoid.cartesianToCartographic(&surfaceCartesian);
        assert_eq!(
            result
                .unwrap()
                .equals_epsilon(&surfaceCartographic, math::EPSILON8),
            true
        );
        let result = ellipsoid.cartesianToCartographic(&spaceCartesian);
        assert_eq!(
            result.unwrap().equals_epsilon(&spaceCartographic, 1.0),
            true
        );
    }
    #[test]
    fn test_cartesianArrayToCartographicArray() {
        let ellipsoid = Ellipsoid::WGS84;
        let result =
            ellipsoid.cartesianArrayToCartographicArray(vec![&surfaceCartesian, &spaceCartesian]);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].equals_epsilon(&surfaceCartographic, math::EPSILON7),
            true
        );
        assert_eq!(
            result[1].equals_epsilon(&spaceCartographic, math::EPSILON7),
            true
        );
    }
    #[test]
    fn test_cartesianToCartographic_close_to_center() {
        let ellipsoid = Ellipsoid::WGS84;
        let expected = Cartographic::new(9.999999999999999e-11, 1.0067394967422763e-20, -6378137.0);
        let result = ellipsoid.cartesianToCartographic(&Cartesian3::new(1e-50, 1e-60, 1e-70));
        assert_eq!(result.is_none(), true);
    }
}
