use std::{fmt::Formatter, ops::Add};

use crate::ellipsoid::Ellipsoid;

use super::cartesian3::Cartesian3;
#[derive(Debug, Copy, Clone, Default)]
pub struct Cartographic {
    pub longitude: f64,
    pub latitude: f64,
    pub height: f64,
}
impl Cartographic {
    pub fn from_radians(longitude: f64, latitude: f64, height: f64) -> Self {
        Cartographic {
            longitude: longitude.to_degrees(),
            latitude: latitude.to_degrees(),
            height,
        }
    }
    pub fn from_degrees(longitude: f64, latitude: f64, height: f64) -> Self {
        Cartographic {
            longitude,
            latitude,
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
    pub fn from_cartesian(cartesian: Cartesian3, ellipsoid: Option<Ellipsoid>) -> Option<Self> {
        let ellipsoid = ellipsoid.unwrap_or(Ellipsoid::WGS84);
        let one_over_radii = ellipsoid.oneOverRadii;
        let one_over_radii_squared = ellipsoid.oneOverRadiiSquared;
        let centerToleranceSquared = ellipsoid.centerToleranceSquared;
        if let Some(p) = ellipsoid.scaleToGeodeticSurface(cartesian) {
            let n = p.multiply_components(&one_over_radii_squared).normalize();
            let h = cartesian - &p;
            let longitude = n.y.atan2(n.x);
            let latitude = n.z.asin();
            let height = h.dot(&cartesian).sin() * h.magnitude();
            return Some(Cartographic {
                longitude,
                latitude,
                height,
            });
        } else {
            return None;
        }
    }
    pub fn to_cartesian(&self, ellipsoid: Option<Ellipsoid>) -> Cartesian3 {
        return Cartesian3::from_radians(self.longitude, self.latitude, Some(self.height), {
            if let Some(e) = ellipsoid {
                Some(e.radiiSquared)
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
    pub fn equals_epsilon(&self, right: &Cartographic, epsilon: f64) -> bool {
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
    use super::*;
}
