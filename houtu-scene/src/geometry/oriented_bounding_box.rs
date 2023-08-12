use std::f64::{
    consts::{FRAC_2_PI, PI},
    MAX,
};
use std::ops::Index;

use crate::{ellipsoid::Ellipsoid, math::*, BoundingVolume, Intersect};
use bevy::{
    math::{DMat3, DVec3},
    prelude::*,
};

use super::{Box3d, EllipsoidTangentPlane, Plane, Rectangle};
#[derive(Clone, Debug, Component, Copy)]
pub struct OrientedBoundingBox {
    pub center: DVec3,
    pub half_axes: DMat3,
}
impl Default for OrientedBoundingBox {
    fn default() -> Self {
        Self {
            center: DVec3::ZERO,
            half_axes: DMat3::ZERO,
        }
    }
}
impl OrientedBoundingBox {
    pub fn from_points(positions: &[DVec3]) -> Self {
        let mut result = Self::default();
        let length = positions.len();
        if length == 0 {
            return result;
        }

        let mut mean_point = positions[0].clone();
        for i in 1..length {
            mean_point = mean_point + positions[i];
        }
        let inv_length: f64 = 1.0 / length as f64;
        mean_point = mean_point / inv_length;

        let mut exx = 0.0;
        let mut exy = 0.0;
        let mut exz = 0.0;
        let mut eyy = 0.0;
        let mut eyz = 0.0;
        let mut ezz = 0.0;
        let mut p;
        for i in 0..length {
            p = positions[i] - mean_point;
            exx += p.x * p.x;
            exy += p.x * p.y;
            exz += p.x * p.z;
            eyy += p.y * p.y;
            eyz += p.y * p.z;
            ezz += p.z * p.z;
        }

        exx *= inv_length;
        exy *= inv_length;
        exz *= inv_length;
        eyy *= inv_length;
        eyz *= inv_length;
        ezz *= inv_length;

        let covariance_matrix_slice = [exx, exy, exz, exy, eyy, eyz, exz, eyz, ezz];
        let covariance_matrix = DMat3::from_cols_array(&covariance_matrix_slice);

        let eigen_decomposition = computeEigenDecomposition(covariance_matrix);
        let rotation = eigen_decomposition.unitary.clone();
        result.half_axes = rotation.clone();

        let mut v1: DVec3 = rotation.col(0).into();
        let mut v2: DVec3 = rotation.col(1).into();
        let mut v3: DVec3 = rotation.col(2).into();

        let mut u1 = -MAX;
        let mut u2 = -MAX;
        let mut u3 = -MAX;
        let mut l1 = MAX;
        let mut l2 = MAX;
        let mut l3 = MAX;
        for i in 0..length {
            p = positions[i];
            u1 = v1.dot(p).max(u1);
            u2 = v2.dot(p).max(u2);
            u3 = v3.dot(p).max(u3);

            l1 = v1.dot(p).min(l1);
            l2 = v2.dot(p).min(l2);
            l3 = v3.dot(p).min(l3);
        }
        v1 = v1 * 0.5 * (l1 + u1);
        v2 = v2 * 0.5 * (l2 + u2);
        v3 = v3 * 0.5 * (l3 + u3);

        result.center = v1 + v2 + v3;
        let scale = DVec3::new(u1 - l1, u2 - l2, u3 - l3) * 0.5;
        result.half_axes = result.half_axes.multiply_by_scale(scale);

        result
    }
    pub fn from_rectangle(
        rectangle: &Rectangle,
        minimum_height: Option<f64>,
        maximum_height: Option<f64>,
        ellipsoid: Option<&Ellipsoid>,
    ) -> Self {
        let minimum_height = minimum_height.unwrap_or(0.0);
        let maximum_height = maximum_height.unwrap_or(0.0);
        let ellipsoid = ellipsoid.unwrap_or(&Ellipsoid::WGS84);

        let mut min_x: f64;
        let mut max_x: f64;
        let mut min_y: f64;
        let mut max_y: f64;
        let mut min_z: f64;
        let mut max_z: f64;
        let mut plane;

        if rectangle.compute_width() <= PI {
            // The bounding box will be aligned with the tangent plane at the center of the rectangle.
            let tangent_point_cartographic = rectangle.center();

            let tangent_point = ellipsoid.cartographic_to_cartesian(&tangent_point_cartographic);
            let tangent_plane = EllipsoidTangentPlane::new(tangent_point, Some(ellipsoid));
            plane = tangent_plane.plane;

            // If the rectangle spans the equator, CW is instead aligned with the equator (because it sticks out the farthest at the equator).
            let lon_center = tangent_point_cartographic.longitude;
            let lat_center = {
                if rectangle.south < 0.0 && rectangle.north > 0.0 {
                    0.0
                } else {
                    tangent_point_cartographic.latitude
                }
            };
            // Compute XY extents using the rectangle at maximum height
            let mut perimeter_cartographic_nc =
                Cartographic::from_radians(lon_center, rectangle.north, maximum_height);
            let mut perimeter_cartographic_nw =
                Cartographic::from_radians(rectangle.west, rectangle.north, maximum_height);
            let mut perimeter_cartographic_cw =
                Cartographic::from_radians(rectangle.west, lat_center, maximum_height);
            let mut perimeter_cartographic_sw =
                Cartographic::from_radians(rectangle.west, rectangle.south, maximum_height);
            let mut perimeter_cartographic_sc =
                Cartographic::from_radians(lon_center, rectangle.south, maximum_height);

            let mut perimeter_cartesian_nc =
                ellipsoid.cartographic_to_cartesian(&perimeter_cartographic_nc);
            let mut perimeter_cartesian_nw =
                ellipsoid.cartographic_to_cartesian(&perimeter_cartographic_nw);
            let mut perimeter_cartesian_cw =
                ellipsoid.cartographic_to_cartesian(&perimeter_cartographic_cw);
            let mut perimeter_cartesian_sw =
                ellipsoid.cartographic_to_cartesian(&perimeter_cartographic_sw);
            let mut perimeter_cartesian_sc =
                ellipsoid.cartographic_to_cartesian(&perimeter_cartographic_sc);

            let perimeter_projected_nc =
                tangent_plane.project_point_to_nearest_on_plane(perimeter_cartesian_nc);
            let perimeter_projected_nw =
                tangent_plane.project_point_to_nearest_on_plane(perimeter_cartesian_nw);
            let perimeter_projected_cw =
                tangent_plane.project_point_to_nearest_on_plane(perimeter_cartesian_cw);
            let perimeter_projected_sw =
                tangent_plane.project_point_to_nearest_on_plane(perimeter_cartesian_sw);
            let perimeter_projected_sc =
                tangent_plane.project_point_to_nearest_on_plane(perimeter_cartesian_sc);

            min_x = perimeter_projected_nw
                .x
                .min(perimeter_projected_cw.x)
                .min(perimeter_projected_sw.x);
            max_x = -min_x; // symmetrical

            max_y = perimeter_projected_nw.y.max(perimeter_projected_nc.y);
            min_y = perimeter_projected_sw.y.min(perimeter_projected_sc.y);

            // Compute minimum Z using the rectangle at minimum height, since it will be deeper than the maximum height
            perimeter_cartographic_nw.height = minimum_height;
            perimeter_cartographic_sw.height = minimum_height;
            perimeter_cartesian_nw =
                ellipsoid.cartographic_to_cartesian(&perimeter_cartographic_nw);
            perimeter_cartesian_sw =
                ellipsoid.cartographic_to_cartesian(&perimeter_cartographic_sw);

            min_z = plane
                .get_point_distance(perimeter_cartesian_nw)
                .min(plane.get_point_distance(perimeter_cartesian_sw));
            max_z = maximum_height; // Since the tangent plane touches the surface at height = 0, this is okay

            return from_plane_extents(
                tangent_plane.origin,
                tangent_plane.x_axis,
                tangent_plane.y_axis,
                tangent_plane.z_axis,
                min_x,
                max_x,
                min_y,
                max_y,
                min_z,
                max_z,
            );
        }

        // Handle the case where rectangle width is greater than PI (wraps around more than half the ellipsoid).
        let fully_above_equator = rectangle.south > 0.0;
        let fully_below_equator = rectangle.north < 0.0;
        let latitude_nearest_to_equator = {
            if fully_above_equator {
                rectangle.south
            } else if fully_below_equator {
                rectangle.north
            } else {
                0.0
            }
        };
        let center_longitude = rectangle.center().longitude;

        // Plane is located at the rectangle's center longitude and the rectangle's latitude that is closest to the equator. It rotates around the Z axis.
        // This results in a better fit than the obb approach for smaller rectangles, which orients with the rectangle's center normal.
        let mut plane_origin = DVec3::from_radians(
            center_longitude,
            latitude_nearest_to_equator,
            Some(maximum_height),
            Some(ellipsoid.radii_squared),
        );
        plane_origin.z = 0.0; // center the plane on the equator to simpify plane normal calculation
        let isPole = plane_origin.x.abs() < EPSILON10 && plane_origin.y.abs() < EPSILON10;
        let plane_normal = {
            if !isPole {
                plane_origin.normalize()
            } else {
                DVec3::UNIT_X
            }
        };

        let plane_yaxis = DVec3::UNIT_Z;
        let plane_xaxis = plane_normal.cross(plane_yaxis);
        plane = Plane::from_point_normal(&plane_origin, &plane_normal);

        // Get the horizon point relative to the center. This will be the farthest extent in the plane's X dimension.
        let horizon_cartesian = DVec3::from_radians(
            center_longitude + FRAC_2_PI,
            latitude_nearest_to_equator,
            Some(maximum_height),
            Some(ellipsoid.radii_squared),
        );
        max_x = plane
            .project_point_onto_plane(horizon_cartesian)
            .dot(plane_xaxis);
        min_x = -max_x; // symmetrical

        // Get the min and max Y, using the height that will give the largest extent
        max_y = DVec3::from_radians(
            0.0,
            rectangle.north,
            {
                if fully_below_equator {
                    Some(minimum_height)
                } else {
                    Some(maximum_height)
                }
            },
            Some(ellipsoid.radii_squared),
        )
        .z;
        min_y = DVec3::from_radians(
            0.0,
            rectangle.south,
            {
                if fully_above_equator {
                    Some(minimum_height)
                } else {
                    Some(maximum_height)
                }
            },
            Some(ellipsoid.radii_squared),
        )
        .z;

        let farZ = DVec3::from_radians(
            rectangle.east,
            latitude_nearest_to_equator,
            Some(maximum_height),
            Some(ellipsoid.radii_squared),
        );
        min_z = plane.get_point_distance(farZ);
        max_z = 0.0; // plane origin starts at max_z already

        // min and max are local to the plane axes
        return from_plane_extents(
            plane_origin,
            plane_xaxis,
            plane_yaxis,
            plane_normal,
            min_x,
            max_x,
            min_y,
            max_y,
            min_z,
            max_z,
        );
    }
    pub fn distance_squared_to(&self, cartesian: &DVec3) -> f64 {
        let offset = cartesian.subtract(self.center);

        let half_axes = self.half_axes;
        let mut u = half_axes.get_column(0);
        let mut v = half_axes.get_column(1);
        let mut w = half_axes.get_column(2);

        let u_half = u.magnitude();
        let v_half = v.magnitude();
        let w_half = w.magnitude();

        let mut u_valid = true;
        let mut v_valid = true;
        let mut w_valid = true;

        if u_half > 0. {
            u = u.divide_by_scalar(u_half);
        } else {
            u_valid = false;
        }

        if v_half > 0. {
            v = v.divide_by_scalar(v_half);
        } else {
            v_valid = false;
        }

        if w_half > 0. {
            w = w.divide_by_scalar(w_half);
        } else {
            w_valid = false;
        }

        let number_of_degenerate_axes = (!u_valid as i8) + (!v_valid as i8) + (!w_valid as i8);
        let mut valid_axis1;
        let mut valid_axis2;
        let mut valid_axis3;

        if number_of_degenerate_axes == 1 {
            let mut degenerate_axis = u;
            valid_axis1 = v;
            valid_axis2 = w;
            if !v_valid {
                degenerate_axis = v;
                valid_axis1 = u;
            } else if !w_valid {
                degenerate_axis = w;
                valid_axis2 = u;
            }

            valid_axis3 = valid_axis1.cross(valid_axis2);

            if degenerate_axis == u {
                u = valid_axis3;
            } else if degenerate_axis == v {
                v = valid_axis3;
            } else if degenerate_axis == w {
                w = valid_axis3;
            }
        } else if number_of_degenerate_axes == 2 {
            valid_axis1 = u;
            if v_valid {
                valid_axis1 = v;
            } else if w_valid {
                valid_axis1 = w;
            }

            let mut cross_vector = DVec3::UNIT_Y;
            if cross_vector.equals_epsilon(valid_axis1, Some(EPSILON3), None) {
                cross_vector = DVec3::UNIT_X;
            }

            valid_axis2 = valid_axis1.cross(cross_vector);
            valid_axis2.normalize();
            valid_axis3 = valid_axis1.cross(valid_axis2);
            valid_axis3.normalize();

            if valid_axis1 == u {
                v = valid_axis2;
                w = valid_axis3;
            } else if valid_axis1 == v {
                w = valid_axis2;
                u = valid_axis3;
            } else if valid_axis1 == w {
                u = valid_axis2;
                v = valid_axis3;
            }
        } else if number_of_degenerate_axes == 3 {
            u = DVec3::UNIT_X;
            v = DVec3::UNIT_Y;
            w = DVec3::UNIT_Z;
        }

        let mut p_prime = DVec3::ZERO;
        p_prime.x = offset.dot(u);
        p_prime.y = offset.dot(v);
        p_prime.z = offset.dot(w);

        let mut distance_squared = 0.0;
        let mut d;

        if p_prime.x < -u_half {
            d = p_prime.x + u_half;
            distance_squared += d * d;
        } else if p_prime.x > u_half {
            d = p_prime.x - u_half;
            distance_squared += d * d;
        }

        if p_prime.y < -v_half {
            d = p_prime.y + v_half;
            distance_squared += d * d;
        } else if p_prime.y > v_half {
            d = p_prime.y - v_half;
            distance_squared += d * d;
        }

        if p_prime.z < -w_half {
            d = p_prime.z + w_half;
            distance_squared += d * d;
        } else if p_prime.z > w_half {
            d = p_prime.z - w_half;
            distance_squared += d * d;
        }

        return distance_squared;
    }
    pub fn intersect_plane(&self, plane: &Plane) -> Intersect {
        let center = self.center;
        let normal = plane.normal;
        let half_axes = self.half_axes;
        let normal_x = normal.x;
        let normal_y = normal.y;
        let normal_z = normal.z;
        let slice = half_axes.to_cols_array();
        // plane is used as if it is its normal; the first three components are assumed to be normalized
        let rad_effective = (normal_x * slice[DMat3::COLUMN0ROW0]
            + normal_y * slice[DMat3::COLUMN0ROW1]
            + normal_z * slice[DMat3::COLUMN0ROW2])
            .abs()
            + (normal_x * slice[DMat3::COLUMN1ROW0]
                + normal_y * slice[DMat3::COLUMN1ROW1]
                + normal_z * slice[DMat3::COLUMN1ROW2])
                .abs()
            + (normal_x * slice[DMat3::COLUMN2ROW0]
                + normal_y * slice[DMat3::COLUMN2ROW1]
                + normal_z * slice[DMat3::COLUMN2ROW2])
                .abs();
        let distance_to_plane = normal.dot(center) + plane.distance;

        if distance_to_plane <= -rad_effective {
            // The entire box is on the negative side of the plane normal
            return Intersect::OUTSIDE;
        } else if distance_to_plane >= rad_effective {
            // The entire box is on the positive side of the plane normal
            return Intersect::INSIDE;
        }
        return Intersect::INTERSECTING;
    }
}
impl BoundingVolume for OrientedBoundingBox {
    fn intersect_plane(&self, plane: &Plane) -> Intersect {
        return self.intersect_plane(plane);
    }
}
pub fn from_plane_extents(
    plane_origin: DVec3,
    plane_xaxis: DVec3,
    plane_yaxis: DVec3,
    plane_zaxis: DVec3,
    minimum_x: f64,
    maximum_x: f64,
    minimum_y: f64,
    maximum_y: f64,
    minimum_z: f64,
    maximum_z: f64,
) -> OrientedBoundingBox {
    let mut result = OrientedBoundingBox::default();

    let mut half_axes = result.half_axes;
    half_axes.set_column(0, &plane_xaxis);
    half_axes.set_column(1, &plane_yaxis);
    half_axes.set_column(2, &plane_zaxis);

    let mut center_offset = DVec3::default();
    center_offset.x = (minimum_x + maximum_x) / 2.0;
    center_offset.y = (minimum_y + maximum_y) / 2.0;
    center_offset.z = (minimum_z + maximum_z) / 2.0;

    let mut scale = DVec3::default();
    scale.x = (maximum_x - minimum_x) / 2.0;
    scale.y = (maximum_y - minimum_y) / 2.0;
    scale.z = (maximum_z - minimum_z) / 2.0;

    let center = result.center;
    center_offset = half_axes.multiply_by_vector(&center_offset);
    result.center = plane_origin + center_offset;
    result.half_axes = half_axes.multiply_by_scale(scale);
    return result;
}

// #[derive(Bundle)]
// pub struct OrientedBoundingBoxBundle {
//     pub obb: OrientedBoundingBox,
//     pub visibility: Visibility,
// }
// pub struct OrientedBoundingBoxPlugin;
// impl Default for OrientedBoundingBoxPlugin {
//     fn default() -> Self {
//         Self {}
//     }
// }

// impl bevy::app::Plugin for OrientedBoundingBoxPlugin {
//     fn build(&self, app: &mut bevy::app::App) {
//         // app.add_startup_system(setup);
//     }
// }
// fn setup(
//     mut commands: bevy::ecs::system::Commands,
//     mut query: Query<&mut OrientedBoundingBox>,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut materials: ResMut<Assets<StandardMaterial>>,
// ) {
//     for (mut obb) in query.iter_mut() {
//         commands.spawn(PbrBundle {
//             mesh: meshes.add(Box3d::from_center_halfaxes(obb.center, obb.half_axes).into()),
//             material: materials.add(Color::BLACK.into()),
//             ..Default::default()
//         });
//     }
// }
#[cfg(test)]
mod tests {
    use bevy::math::DVec3;

    use super::*;
    const POSITIONS: [DVec3; 6] = [
        DVec3::new(2.0, 0.0, 0.0),
        DVec3::new(0.0, 3.0, 0.0),
        DVec3::new(0.0, 0.0, 4.0),
        DVec3::new(-2.0, 0.0, 0.0),
        DVec3::new(0.0, -3.0, 0.0),
        DVec3::new(0.0, 0.0, -4.0),
    ];

    #[test]
    fn init_work() {
        let obb = OrientedBoundingBox::default();
        assert_eq!(obb.center, DVec3::ZERO);
        assert_eq!(obb.half_axes, DMat3::ZERO);
    }
    #[test]
    fn empty_points_work() {
        let points = vec![];
        let obb = OrientedBoundingBox::from_points(&points);
        assert_eq!(obb.center, DVec3::ZERO);
        assert_eq!(obb.half_axes, DMat3::ZERO);
    }
    #[test]
    fn fromPointsCorrectScale() {
        let obb = OrientedBoundingBox::from_points(&POSITIONS);
        assert_eq!(obb.half_axes, DMat3::from_scale3(DVec3::new(2.0, 3.0, 4.0)));
        assert_eq!(obb.center, DVec3::ZERO);
    }
}
