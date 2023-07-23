use std::{fmt::Formatter, ops::Add};

use bevy::math::DVec3;

use crate::ellipsoid::Ellipsoid;
use crate::math::*;
#[derive(Debug, Copy, Clone, Default, PartialEq)]
pub struct Cartographic {
    pub longitude: f64,
    pub latitude: f64,
    pub height: f64,
}
impl Cartographic {
    pub fn new(longitude: f64, latitude: f64, height: f64) -> Self {
        Cartographic::from_radians(longitude, latitude, height)
    }
    pub fn from_radians(longitude: f64, latitude: f64, height: f64) -> Self {
        Cartographic {
            longitude: longitude,
            latitude: latitude,
            height,
        }
    }
    pub fn from_degrees(longitude: f64, latitude: f64, height: f64) -> Self {
        Cartographic {
            longitude: longitude.to_radians(),
            latitude: latitude.to_radians(),
            height,
        }
    }
    pub fn to_radians(&self) -> Self {
        Cartographic {
            longitude: self.longitude.to_radians(),
            latitude: self.latitude.to_radians(),
            height: self.height,
        }
    }
    pub fn to_degrees(&self) -> Self {
        Cartographic {
            longitude: self.longitude.to_degrees(),
            latitude: self.latitude.to_degrees(),
            height: self.height,
        }
    }
    pub fn from_cartesian(cartesian: DVec3, ellipsoid: Option<&Ellipsoid>) -> Option<Self> {
        let ellipsoid = ellipsoid.unwrap_or(&Ellipsoid::WGS84);
        let one_over_radii = ellipsoid.oneOverRadii;
        let one_over_radii_squared = ellipsoid.oneOverRadiiSquared;
        let centerToleranceSquared = ellipsoid.centerToleranceSquared;
        if let Some(p) = ellipsoid.scaleToGeodeticSurface(&cartesian) {
            let n = p * one_over_radii_squared.normalize();
            let h = cartesian - p;
            let longitude = n.y.atan2(n.x);
            let latitude = n.z.asin();
            let height = h.dot(cartesian).sin() * h.magnitude();
            return Some(Cartographic {
                longitude,
                latitude,
                height,
            });
        } else {
            return None;
        }
    }
    pub fn to_cartesian(&self, ellipsoid: Option<Ellipsoid>) -> DVec3 {
        return DVec3::from_radians(self.longitude, self.latitude, Some(self.height), {
            if let Some(e) = ellipsoid {
                Some(e.radii_squared)
            } else {
                None
            }
        });
    }
    pub fn equals(&self, right: &Cartographic) -> bool {
        return self.longitude == right.longitude
            && self.latitude == right.latitude
            && self.height == right.height;
    }
    pub fn equals_epsilon(self, right: Cartographic, epsilon: f64) -> bool {
        return (self.longitude - right.longitude).abs() <= epsilon
            && (self.latitude - right.latitude).abs() <= epsilon
            && (self.height - right.height).abs() <= epsilon;
    }
    pub const ZERO: Cartographic = Cartographic {
        longitude: 0.0,
        latitude: 0.0,
        height: 0.0,
    };
}
impl ToString for Cartographic {
    fn to_string(&self) -> String {
        return format!(
            "Cartographic {{ longitude: {}, latitude: {}, height: {} }}",
            self.longitude, self.latitude, self.height
        );
    }
}

#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use super::*;
    const surfaceCartographic: Cartographic = Cartographic {
        longitude: 25.0 * PI / 180.0,
        latitude: 45.0 * PI / 180.0,
        height: 0.0,
    };

    const surfaceCartesian: DVec3 = DVec3 {
        x: 4094327.7921465295,
        y: 1909216.4044747739,
        z: 4487348.4088659193,
    };
    #[test]
    fn test_to_cartesian() {
        let lon = 150.0.to_radians();
        let lat = -40.0.to_radians();
        let height = 100000.0;
        let ellipsoid = Ellipsoid::WGS84;
        let actual = Cartographic::new(lon, lat, height).to_cartesian(None);
        let expected = ellipsoid.cartographicToCartesian(&Cartographic::new(lon, lat, height));
        assert!(actual.equals(expected));
    }
    #[test]
    fn test_from_cartesian() {
        let ellipsoid = Ellipsoid::WGS84;
        let c = Cartographic::from_cartesian(surfaceCartesian, None).unwrap();
        assert!(c.equals_epsilon(surfaceCartographic, 1e-5));
    }
}
