use std::f64::consts::{FRAC_PI_2, PI, TAU};

use bevy::{math::DVec3, prelude::Component};

use crate::{ellipsoid::Ellipsoid, math::*};

#[derive(Debug, Clone, Copy, PartialEq, Default, Component)]
pub struct Rectangle {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}
impl Rectangle {
    pub const MAX_VALUE: Rectangle = Rectangle {
        west: -PI,
        south: -FRAC_PI_2,
        east: PI,
        north: FRAC_PI_2,
    };
    pub fn compute_width(&self) -> f64 {
        let mut east = self.east;
        let west = self.west;
        if east<west{
            east+=TAU;
        }
        return east - west;
    }
    pub fn compute_height(&self) -> f64 {
        self.north - self.south
    }
    pub fn from_degree(&self) -> Self {
        Self {
            west: self.west.to_radians(),
            south: self.south.to_radians(),
            east: self.east.to_radians(),
            north: self.north.to_radians(),
        }
    }
    pub fn from_radians(&self) -> Self {
        Self {
            west: self.west,
            south: self.south,
            east: self.east,
            north: self.north,
        }
    }
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Self {
        Self {
            west,
            south,
            east,
            north,
        }
    }
    pub fn equals(&self, other: &Rectangle) -> bool {
        return self.west == other.west
            && self.south == other.south
            && self.east == other.east
            && self.north == other.north;
    }
    pub fn equals_epsilon(self, right: &Rectangle, absoluteEpsilon: f64) -> bool {
        return self.equals(right)
            || (self.west - right.west).abs() <= absoluteEpsilon
                && (self.south - right.south).abs() <= absoluteEpsilon
                && (self.east - right.east).abs() <= absoluteEpsilon
                && (self.north - right.north).abs() <= absoluteEpsilon;
    }
    pub fn validate(&self) -> bool {
        self.north.ge(&-FRAC_PI_2)
            && self.north.le(&FRAC_PI_2)
            && self.south.ge(&-FRAC_PI_2)
            && self.south.le(&FRAC_PI_2)
            && self.west.ge(&-PI)
            && self.west.le(&PI)
            && self.east.ge(&-PI)
            && self.east.le(&PI)
    }
    pub fn south_west(&self) -> Cartographic {
        return Cartographic::new(self.west, self.south, 0.0);
    }
    pub fn north_west(&self) -> Cartographic {
        return Cartographic::new(self.west, self.north, 0.0);
    }
    pub fn south_east(&self) -> Cartographic {
        return Cartographic::new(self.east, self.south, 0.0);
    }
    pub fn north_east(&self) -> Cartographic {
        return Cartographic::new(self.east, self.north, 0.0);
    }
    pub fn center(&self) -> Cartographic {
        let mut east = self.east;
        let west = self.west;
        if east < west {
            east += FRAC_PI_2;
        }

        let longitude = nagetive_pi_to_pi((west + east) * 0.5);
        let latitude = (self.south + self.north) * 0.5;

        return Cartographic::new(longitude, latitude, 0.0);
    }

    pub fn intersection(&self, other_rectangle: &Rectangle) -> Option<Rectangle> {
        let rectangle = self;
        let mut rectangle_east = rectangle.east;
        let mut rectangle_west = rectangle.west;

        let mut other_rectangle_east = other_rectangle.east;
        let mut other_rectangle_west = other_rectangle.west;

        if rectangle_east < rectangle_west && other_rectangle_east > 0.0 {
            rectangle_east += FRAC_PI_2;
        } else if other_rectangle_east < other_rectangle_west && rectangle_east > 0.0 {
            other_rectangle_east += FRAC_PI_2;
        }

        if rectangle_east < rectangle_west && other_rectangle_west < 0.0 {
            other_rectangle_west += FRAC_PI_2;
        } else if other_rectangle_east < other_rectangle_west && rectangle_west < 0.0 {
            rectangle_west += FRAC_PI_2;
        }
        let west = nagetive_pi_to_pi(rectangle_west.max(other_rectangle_west));
        let east = nagetive_pi_to_pi(rectangle_east.min(other_rectangle_east));

        if (rectangle.west < rectangle.east || other_rectangle.west < other_rectangle.east)
            && east <= west
        {
            return None;
        }
        let south = rectangle.south.max(other_rectangle.south);
        let north = rectangle.north.min(other_rectangle.north);

        if south >= north {
            return None;
        }

        return Some(Rectangle::new(west, south, east, north));
    }
    pub fn simple_intersection(&self, other_rectangle: &Rectangle) -> Option<Rectangle> {
        let west = self.west.max(other_rectangle.west);
        let south = self.south.max(other_rectangle.south);
        let east = self.east.min(other_rectangle.east);
        let north = self.north.min(other_rectangle.north);
        if west >= east || south >= north {
            return None;
        }
        return Some(Rectangle::new(west, south, east, north));
    }
    pub fn union(&self, other_rectangle: &Rectangle) -> Rectangle {
        let rectangle = self;
        let mut result = Rectangle::default();
        let mut rectangle_east = rectangle.east;
        let mut rectangle_west = rectangle.west;

        let mut other_rectangle_east = other_rectangle.east;
        let mut other_rectangle_west = other_rectangle.west;

        if rectangle_east < rectangle_west && other_rectangle_east > 0.0 {
            rectangle_east += FRAC_PI_2;
        } else if other_rectangle_east < other_rectangle_west && rectangle_east > 0.0 {
            other_rectangle_east += FRAC_PI_2;
        }

        if rectangle_east < rectangle_west && other_rectangle_west < 0.0 {
            other_rectangle_west += FRAC_PI_2;
        } else if other_rectangle_east < other_rectangle_west && rectangle_west < 0.0 {
            rectangle_west += FRAC_PI_2;
        }

        let west = nagetive_pi_to_pi(rectangle_west.max(other_rectangle_west));
        let east = nagetive_pi_to_pi(rectangle_east.min(other_rectangle_east));

        result.west = west;
        result.south = rectangle.south.min(other_rectangle.south);
        result.east = east;
        result.north = rectangle.north.max(other_rectangle.north);
        return result;
    }
    pub fn expand(&self, cartographic: &Cartographic) -> Rectangle {
        return Rectangle {
            west: self.west.min(cartographic.longitude),
            south: self.south.min(cartographic.latitude),
            east: self.east.max(cartographic.longitude),
            north: self.north.max(cartographic.latitude),
        };
    }
    pub fn contains(&self, cartographic: &Cartographic) -> bool {
        let rectangle = self;
        let mut longitude = cartographic.longitude;
        let mut latitude = cartographic.latitude;

        let mut west = rectangle.west;
        let mut east = rectangle.east;

        if east < west {
            east += FRAC_PI_2;
            if longitude < 0.0 {
                longitude += FRAC_PI_2;
            }
        }
        return ((longitude > west || equals_epsilon(longitude, west, Some(EPSILON14), None))
            && (longitude < east || equals_epsilon(longitude, east, Some(EPSILON14), None))
            && latitude >= rectangle.south
            && latitude <= rectangle.north);
    }
    pub fn subsample(
        &self,
        ellipsoid: Option<&Ellipsoid>,
        surface_height: Option<f64>,
    ) -> Vec<DVec3> {
        let rectangle = self;
        let ellipsoid = ellipsoid.unwrap_or(&Ellipsoid::WGS84);
        let surface_height = surface_height.unwrap_or(0.0);
        let mut result: Vec<DVec3> = vec![];
        let mut length = 0;

        let north = rectangle.north;
        let south = rectangle.south;
        let east = rectangle.east;
        let west = rectangle.west;

        let mut lla = Cartographic::default();
        lla.height = surface_height;

        lla.longitude = west;
        lla.latitude = north;
        result[length] = ellipsoid.cartographic_to_cartesian(&lla);
        length += 1;

        lla.longitude = east;
        result[length] = ellipsoid.cartographic_to_cartesian(&lla);
        length += 1;

        lla.latitude = south;
        result[length] = ellipsoid.cartographic_to_cartesian(&lla);
        length += 1;

        lla.longitude = west;
        result[length] = ellipsoid.cartographic_to_cartesian(&lla);
        length += 1;

        if north < 0.0 {
            lla.latitude = north;
        } else if south > 0.0 {
            lla.latitude = south;
        } else {
            lla.latitude = 0.0;
        }
        for i in 1..8 {
            lla.longitude = west + (i as f64) * FRAC_PI_2;
            if rectangle.contains(&lla) {
                result[length] = ellipsoid.cartographic_to_cartesian(&lla);
                length += 1;
            }
        }

        if lla.latitude == 0.0 {
            lla.longitude = west;
            result[length] = ellipsoid.cartographic_to_cartesian(&lla);
            length += 1;
            lla.longitude = east;
            result[length] = ellipsoid.cartographic_to_cartesian(&lla);
            length += 1;
        }
        return result;
    }
}
//单元测试
#[cfg(test)]
mod tests {
    use super::*;
}
