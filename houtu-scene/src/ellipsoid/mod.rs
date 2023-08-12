use std::f32::consts::{PI, TAU};
use std::fmt;
use std::ops::Sub;

use crate::math::*;
use bevy::math::DVec3;
// use bevy::DVec3;
use bevy::prelude::{Mesh, Resource};
use bevy::render::mesh::Indices;
#[derive(Debug, Clone, Copy, PartialEq, Resource)]
pub struct Ellipsoid {
    pub radii: DVec3,
    pub radii_squared: DVec3,
    pub radii_to_the_fourth: DVec3,
    pub one_over_radii: DVec3,
    pub one_over_radii_squared: DVec3,
    pub minimum_radius: f64,
    pub maximum_radius: f64,
    pub center_tolerance_squared: f64,
    pub squared_xover_squared_z: f64,
}
impl Default for Ellipsoid {
    fn default() -> Self {
        let radii = DVec3::ZERO;
        return Ellipsoid::from_vec3(radii);
    }
}
impl Ellipsoid {
    pub fn from_vec3(radii: DVec3) -> Self {
        return Ellipsoid::new(radii.x, radii.y, radii.z);
    }
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        let radii = DVec3::new(x, y, z);
        let radii_squared = DVec3::new(x * x, y * y, z * z);
        let radii_to_the_fourth = DVec3::new(x * x * x * x, y * y * y * y, z * z * z * z);
        let one_over_radii = DVec3::new(1.0 / x, 1.0 / y, 1.0 / z);
        let one_over_radii_squared = DVec3::new(1.0 / (x * x), 1.0 / (y * y), 1.0 / (z * z));
        let minimum_radius = x.min(y).min(z);
        let maximum_radius = x.max(y).max(z);
        let center_tolerance_squared = EPSILON1;
        let mut squared_xover_squared_z = 0.0;
        if radii_squared.z != 0. {
            squared_xover_squared_z = radii_squared.x / radii_squared.z;
        }
        Ellipsoid {
            radii,
            radii_squared,
            radii_to_the_fourth,
            one_over_radii,
            one_over_radii_squared,
            minimum_radius,
            maximum_radius,
            center_tolerance_squared,
            squared_xover_squared_z,
        }
    }
    pub const WGS84: Ellipsoid = Ellipsoid {
        radii: DVec3 {
            x: 6378137.0,
            y: 6378137.0,
            z: 6356752.3142451793,
        },
        radii_squared: DVec3 {
            x: 40680631590769.0,
            y: 40680631590769.0,
            z: 40408299984661.445,
        },
        radii_to_the_fourth: DVec3 {
            x: 1.6549137866238727e+27,
            y: 1.6549137866238727e+27,
            z: 1.63283070765039e+27,
        },
        one_over_radii: DVec3 {
            x: 1.567855942887398e-7,
            y: 1.567855942887398e-7,
            z: 1.573130351105623e-7,
        },
        one_over_radii_squared: DVec3 {
            x: 2.458172257647332e-14,
            y: 2.458172257647332e-14,
            z: 2.4747391015697002e-14,
        },
        minimum_radius: 6356752.3142451793,
        maximum_radius: 6378137.0,
        center_tolerance_squared: 0.1,
        squared_xover_squared_z: 1.0067394967422765,
    };
    pub const UNIT_SPHERE: Ellipsoid = Ellipsoid {
        radii: DVec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        radii_squared: DVec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        radii_to_the_fourth: DVec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        one_over_radii: DVec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        one_over_radii_squared: DVec3 {
            x: 1.0,
            y: 1.0,
            z: 1.0,
        },
        minimum_radius: 1.0,
        maximum_radius: 1.0,
        center_tolerance_squared: 0.1,
        squared_xover_squared_z: 1.0,
    };
    pub const MOON: Ellipsoid = Ellipsoid {
        radii: DVec3 {
            x: 1737400.0,
            y: 1737400.0,
            z: 1737400.0,
        },
        radii_squared: DVec3 {
            x: 3018558760000.,
            y: 3018558760000.,
            z: 3018558760000.,
        },
        radii_to_the_fourth: DVec3 {
            x: 9.111696987572739e+24,
            y: 9.111696987572739e+24,
            z: 9.111696987572739e+24,
        },
        one_over_radii: DVec3 {
            x: 5.755726948313572e-7,
            y: 5.755726948313572e-7,
            z: 5.755726948313572e-7,
        },
        one_over_radii_squared: DVec3 {
            x: 3.3128392703543064e-13,
            y: 3.3128392703543064e-13,
            z: 3.3128392703543064e-13,
        },
        minimum_radius: 1737400.0,
        maximum_radius: 1737400.0,
        center_tolerance_squared: 0.1,
        squared_xover_squared_z: 1.0,
    };
    pub fn geocentric_surface_normal(vec3: &DVec3) -> DVec3 {
        return vec3.normalize();
    }
    pub fn semimajor_axis(&self) -> f64 {
        return self.radii.x;
    }
    pub fn semiminor_axis(&self) -> f64 {
        return self.radii.z;
    }
    pub fn geodetic_surface_normal_cartographic(&self, cartographic: &Cartographic) -> DVec3 {
        let longitude = cartographic.longitude;
        let latitude = cartographic.latitude;
        let cos_latitude = latitude.cos();
        let x = cos_latitude * longitude.cos();
        let y = cos_latitude * longitude.sin();
        let z = latitude.sin();
        return DVec3::new(x, y, z).normalize();
    }
    pub fn geodetic_surface_normal(&self, vec3: &DVec3) -> Option<DVec3> {
        if vec3.equals_epsilon(DVec3::ZERO, Some(EPSILON14), None) {
            return None;
        }
        Some(
            vec3.multiply_components(&self.one_over_radii_squared)
                .normalize(),
        )
    }
    pub fn scale_to_geodetic_surface(&self, cartesian: &DVec3) -> Option<DVec3> {
        let one_over_radii = self.one_over_radii;
        let one_over_radii_squared = self.one_over_radii_squared;
        let center_tolerance_squared = self.center_tolerance_squared;
        let position_x = cartesian.x;
        let position_y = cartesian.y;
        let position_z = cartesian.z;
        let one_over_radii_x = one_over_radii.x;
        let one_over_radii_y = one_over_radii.y;
        let one_over_radii_z = one_over_radii.z;

        let x2 = position_x * position_x * one_over_radii_x * one_over_radii_x;
        let y2 = position_y * position_y * one_over_radii_y * one_over_radii_y;
        let z2 = position_z * position_z * one_over_radii_z * one_over_radii_z;
        let mut squared_norm = x2 + y2 + z2;

        // Compute the squared ellipsoid norm.

        let ratio = (1.0 / squared_norm).sqrt();

        // As an initial approximation, assume that the radial intersection is the projection point.
        let intersection = cartesian.multiply_by_scalar(ratio);
        // If the position is near the center, the iteration will not converge.
        if squared_norm < center_tolerance_squared {
            if !ratio.is_finite() {
                return None;
            } else {
                return Some(intersection);
            }
        }

        let one_over_radii_squared_x = one_over_radii_squared.x;
        let one_over_radii_squared_y = one_over_radii_squared.y;
        let one_over_radii_squared_z = one_over_radii_squared.z;

        // Use the gradient at the intersection point in place of the true unit normal.
        // The difference in magnitude will be absorbed in the multiplier.
        let mut gradient = intersection;
        gradient.x = intersection.x * one_over_radii_squared_x * 2.0;
        gradient.y = intersection.y * one_over_radii_squared_y * 2.0;
        gradient.z = intersection.z * one_over_radii_squared_z * 2.0;

        // Compute the initial guess at the normal vector multiplier, lambda.
        let mut lambda = ((1.0 - ratio) * cartesian.magnitude()) / (0.5 * gradient.magnitude());
        let mut correction = 0.0;

        let mut func: f64 = 0.;
        let mut denominator: f64 = 0.;
        let mut x_multiplier: f64 = 0.;
        let mut y_multiplier: f64 = 0.;
        let mut z_multiplier: f64 = 0.;
        let mut x_multiplier2: f64 = 0.;
        let mut y_multiplier2: f64 = 0.;
        let mut z_multiplier2: f64 = 0.;
        let mut x_multiplier3: f64 = 0.;
        let mut y_multiplier3: f64 = 0.;
        let mut z_multiplier3: f64 = 0.;
        loop {
            lambda -= correction;

            x_multiplier = 1.0 / (1.0 + lambda * one_over_radii_squared_x);
            y_multiplier = 1.0 / (1.0 + lambda * one_over_radii_squared_y);
            z_multiplier = 1.0 / (1.0 + lambda * one_over_radii_squared_z);

            x_multiplier2 = x_multiplier * x_multiplier;
            y_multiplier2 = y_multiplier * y_multiplier;
            z_multiplier2 = z_multiplier * z_multiplier;

            x_multiplier3 = x_multiplier2 * x_multiplier;
            y_multiplier3 = y_multiplier2 * y_multiplier;
            z_multiplier3 = z_multiplier2 * z_multiplier;

            func = x2 * x_multiplier2 + y2 * y_multiplier2 + z2 * z_multiplier2 - 1.0;

            // "denominator" here refers to the use of this expression in the velocity and acceleration
            // computations in the sections to follow.
            denominator = x2 * x_multiplier3 * one_over_radii_squared_x
                + y2 * y_multiplier3 * one_over_radii_squared_y
                + z2 * z_multiplier3 * one_over_radii_squared_z;

            let derivative = -2.0 * denominator;

            correction = func / derivative;
            if func.abs() < EPSILON12 {
                break;
            }
        }

        return Some(DVec3::new(
            position_x * x_multiplier,
            position_y * y_multiplier,
            position_z * z_multiplier,
        ));
    }
    pub fn cartographic_to_cartesian(&self, cartographic: &Cartographic) -> DVec3 {
        let mut n = DVec3::ZERO;
        let mut k = DVec3::ZERO;
        n = self.geodetic_surface_normal_cartographic(cartographic);
        k = self.radii_squared * n;
        let gamma = n.dot(k).sqrt();
        k = k / gamma;
        n = n * cartographic.height;
        return k + n;
    }
    pub fn cartesian_to_cartographic(&self, vec3: &DVec3) -> Option<Cartographic> {
        if let Some(p) = self.scale_to_geodetic_surface(&vec3) {
            if let Some(n) = self.geodetic_surface_normal(&p) {
                let h = vec3.subtract(p);
                let longitude = n.y.atan2(n.x);
                let latitude = n.z.asin();
                let b = h.dot(*vec3);
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
    pub fn cartographic_array_to_cartesian_array(
        &self,
        cartographics: Vec<&Cartographic>,
    ) -> Vec<DVec3> {
        let mut result = Vec::with_capacity(cartographics.len());
        for cartographic in cartographics {
            result.push(self.cartographic_to_cartesian(cartographic));
        }
        return result;
    }
    pub fn cartesian_array_to_cartographic_array(
        &self,
        cartesians: Vec<DVec3>,
    ) -> Vec<Cartographic> {
        let mut result = Vec::with_capacity(cartesians.len());
        for cartesian in cartesians {
            if let Some(cartographic) = self.cartesian_to_cartographic(&cartesian) {
                result.push(cartographic);
            }
        }
        return result;
    }
    pub fn transform_position_to_scaled_space(&self, position: &DVec3) -> DVec3 {
        return position.multiply_components(&self.one_over_radii);
    }
    pub fn scale_to_geocentric_surface(&self, cartesian: &DVec3) -> DVec3 {
        let position_x = cartesian.x;
        let position_y = cartesian.y;
        let position_z = cartesian.z;
        let one_over_radii_squared = self.one_over_radii_squared;

        let beta = 1.0
            / (position_x * position_x * one_over_radii_squared.x
                + position_y * position_y * one_over_radii_squared.y
                + position_z * position_z * one_over_radii_squared.z)
                .sqrt();
        return cartesian.clone() * beta;
    }
}
#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::*;
    const SPACE_CARTESIAN: DVec3 = DVec3 {
        x: 4582719.8827300891,
        y: -4582719.8827300882,
        z: 1725510.4250797231,
    };

    const SPACE_CARTOGRAPHIC: Cartographic = Cartographic {
        longitude: -0.7853981633974483,
        latitude: 0.2617993877991494,
        height: 330000.0,
    };
    const SURFACE_CARTOGRAPHIC: Cartographic = Cartographic {
        longitude: 25.0 * PI / 180.0,
        latitude: 45.0 * PI / 180.0,
        height: 0.0,
    };

    const SURFACE_CARTESIAN: DVec3 = DVec3 {
        x: 4094327.7921465295,
        y: 1909216.4044747739,
        z: 4487348.4088659193,
    };
    #[test]
    fn test_cartographic_to_cartesian() {
        let ellipsoid = Ellipsoid::WGS84;
        let cartographic = DVec3::new(0.0, 0.0, 0.0);
        let result = ellipsoid.cartographic_to_cartesian(&SPACE_CARTOGRAPHIC);
        assert_eq!(
            result.equals_epsilon(SPACE_CARTESIAN, Some(EPSILON7), None),
            true
        );
    }
    #[test]
    fn test_cartographic_array_to_cartesian_array() {
        let ellipsoid = Ellipsoid::WGS84;
        let cartographic = DVec3::new(0.0, 0.0, 0.0);
        let result = ellipsoid.cartographic_array_to_cartesian_array(vec![
            &SPACE_CARTOGRAPHIC,
            &SURFACE_CARTOGRAPHIC,
        ]);

        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].equals_epsilon(SPACE_CARTESIAN, Some(EPSILON7), None),
            true
        );
        assert_eq!(
            result[1].equals_epsilon(SURFACE_CARTESIAN, Some(EPSILON7), None),
            true
        );
    }
    #[test]
    fn test_cartesian_to_cartographic() {
        let ellipsoid = Ellipsoid::WGS84;
        let result = ellipsoid.cartesian_to_cartographic(&SURFACE_CARTESIAN);
        assert_eq!(
            result
                .unwrap()
                .equals_epsilon(SURFACE_CARTOGRAPHIC, EPSILON8),
            true
        );
    }
    #[test]
    fn test_cartesian_array_to_cartographic_array() {
        let ellipsoid = Ellipsoid::WGS84;
        let result = ellipsoid
            .cartesian_array_to_cartographic_array(vec![SURFACE_CARTESIAN, SPACE_CARTESIAN]);
        assert_eq!(result.len(), 2);
        assert_eq!(
            result[0].equals_epsilon(SURFACE_CARTOGRAPHIC, EPSILON7),
            true
        );
        assert_eq!(result[1].equals_epsilon(SPACE_CARTOGRAPHIC, EPSILON7), true);
    }
    #[test]
    fn test_cartesian_to_cartographic_close_to_center() {
        let ellipsoid = Ellipsoid::WGS84;
        let expected = Cartographic::new(9.999999999999999e-11, 1.0067394967422763e-20, -6378137.0);
        let result = ellipsoid.cartesian_to_cartographic(&DVec3::new(1e-50, 1e-60, 1e-70));
        assert_eq!(result.is_none(), true);
    }
}
