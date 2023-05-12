use std::f64::{
    consts::{FRAC_2_PI, PI},
    MAX,
};

use crate::{ellipsoid::Ellipsoid, math::*};
use bevy::{
    math::{DMat3, DVec3},
    prelude::*,
};

use super::{Box3d, EllipsoidTangentPlane, Plane, Rectangle};
#[derive(Component)]
pub struct OrientedBoundingBox {
    pub center: DVec3,
    pub halfAxes: DMat3,
}
impl Default for OrientedBoundingBox {
    fn default() -> Self {
        Self {
            center: DVec3::ZERO,
            halfAxes: DMat3::ZERO,
        }
    }
}
impl OrientedBoundingBox {
    pub fn fromPoints(positions: &[DVec3]) -> Self {
        let mut result = Self::default();
        let length = positions.len();
        if length == 0 {
            return result;
        }

        let mut meanPoint = positions[0].clone();
        for i in 1..length {
            meanPoint = meanPoint + positions[i];
        }
        let invLength: f64 = 1.0 / length as f64;
        meanPoint = meanPoint / invLength;

        let mut exx = 0.0;
        let mut exy = 0.0;
        let mut exz = 0.0;
        let mut eyy = 0.0;
        let mut eyz = 0.0;
        let mut ezz = 0.0;
        let mut p;
        for i in 0..length {
            p = positions[i] - meanPoint;
            exx += p.x * p.x;
            exy += p.x * p.y;
            exz += p.x * p.z;
            eyy += p.y * p.y;
            eyz += p.y * p.z;
            ezz += p.z * p.z;
        }

        exx *= invLength;
        exy *= invLength;
        exz *= invLength;
        eyy *= invLength;
        eyz *= invLength;
        ezz *= invLength;

        let covarianceMatrixSlice = [exx, exy, exz, exy, eyy, eyz, exz, eyz, ezz];
        let covarianceMatrix = DMat3::from_cols_array(&covarianceMatrixSlice);

        let eigenDecomposition = computeEigenDecomposition(covarianceMatrix);
        let rotation = eigenDecomposition.unitary.clone();
        result.halfAxes = rotation.clone();

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
        result.halfAxes = result.halfAxes.multiply_by_scale(scale);

        result
    }
    pub fn fromRectangle(
        rectangle: &Rectangle,
        minimumHeight: Option<f64>,
        maximumHeight: Option<f64>,
        ellipsoid: Option<Ellipsoid>,
    ) -> Self {
        let minimumHeight = minimumHeight.unwrap_or(0.0);
        let maximumHeight = maximumHeight.unwrap_or(0.0);
        let ellipsoid = ellipsoid.unwrap_or(Ellipsoid::WGS84);

        let mut minX: f64;
        let mut maxX: f64;
        let mut minY: f64;
        let mut maxY: f64;
        let mut minZ: f64;
        let mut maxZ: f64;
        let mut plane;

        if rectangle.computeWidth() <= PI {
            // The bounding box will be aligned with the tangent plane at the center of the rectangle.
            let tangentPointCartographic = rectangle.center();

            let tangentPoint = ellipsoid.cartographicToCartesian(&tangentPointCartographic);
            let tangentPlane = EllipsoidTangentPlane::new(tangentPoint, Some(ellipsoid));
            plane = tangentPlane.plane;

            // If the rectangle spans the equator, CW is instead aligned with the equator (because it sticks out the farthest at the equator).
            let lonCenter = tangentPointCartographic.longitude;
            let latCenter = {
                if rectangle.south < 0.0 && rectangle.north > 0.0 {
                    0.0
                } else {
                    tangentPointCartographic.latitude
                }
            };
            // Compute XY extents using the rectangle at maximum height
            let mut perimeterCartographicNC =
                Cartographic::from_radians(lonCenter, rectangle.north, maximumHeight);
            let mut perimeterCartographicNW =
                Cartographic::from_radians(rectangle.west, rectangle.north, maximumHeight);
            let mut perimeterCartographicCW =
                Cartographic::from_radians(rectangle.west, latCenter, maximumHeight);
            let mut perimeterCartographicSW =
                Cartographic::from_radians(rectangle.west, rectangle.south, maximumHeight);
            let mut perimeterCartographicSC =
                Cartographic::from_radians(lonCenter, rectangle.south, maximumHeight);

            let mut perimeterCartesianNC =
                ellipsoid.cartographicToCartesian(&perimeterCartographicNC);
            let mut perimeterCartesianNW =
                ellipsoid.cartographicToCartesian(&perimeterCartographicNW);
            let mut perimeterCartesianCW =
                ellipsoid.cartographicToCartesian(&perimeterCartographicCW);
            let mut perimeterCartesianSW =
                ellipsoid.cartographicToCartesian(&perimeterCartographicSW);
            let mut perimeterCartesianSC =
                ellipsoid.cartographicToCartesian(&perimeterCartographicSC);

            let perimeterProjectedNC =
                tangentPlane.projectPointToNearestOnPlane(perimeterCartesianNC);
            let perimeterProjectedNW =
                tangentPlane.projectPointToNearestOnPlane(perimeterCartesianNW);
            let perimeterProjectedCW =
                tangentPlane.projectPointToNearestOnPlane(perimeterCartesianCW);
            let perimeterProjectedSW =
                tangentPlane.projectPointToNearestOnPlane(perimeterCartesianSW);
            let perimeterProjectedSC =
                tangentPlane.projectPointToNearestOnPlane(perimeterCartesianSC);

            minX = perimeterProjectedNW
                .x
                .min(perimeterProjectedCW.x)
                .min(perimeterProjectedSW.x);
            maxX = -minX; // symmetrical

            maxY = perimeterProjectedNW.y.max(perimeterProjectedNC.y);
            minY = perimeterProjectedSW.y.min(perimeterProjectedSC.y);

            // Compute minimum Z using the rectangle at minimum height, since it will be deeper than the maximum height
            perimeterCartographicNW.height = minimumHeight;
            perimeterCartographicSW.height = minimumHeight;
            perimeterCartesianNW = ellipsoid.cartographicToCartesian(&perimeterCartographicNW);
            perimeterCartesianSW = ellipsoid.cartographicToCartesian(&perimeterCartographicSW);

            minZ = plane
                .getPointDistance(perimeterCartesianNW)
                .min(plane.getPointDistance(perimeterCartesianSW));
            maxZ = maximumHeight; // Since the tangent plane touches the surface at height = 0, this is okay

            return fromPlaneExtents(
                tangentPlane.origin,
                tangentPlane.xAxis,
                tangentPlane.yAxis,
                tangentPlane.zAxis,
                minX,
                maxX,
                minY,
                maxY,
                minZ,
                maxZ,
            );
        }

        // Handle the case where rectangle width is greater than PI (wraps around more than half the ellipsoid).
        let fullyAboveEquator = rectangle.south > 0.0;
        let fullyBelowEquator = rectangle.north < 0.0;
        let latitudeNearestToEquator = {
            if fullyAboveEquator {
                rectangle.south
            } else if fullyBelowEquator {
                rectangle.north
            } else {
                0.0
            }
        };
        let centerLongitude = rectangle.center().longitude;

        // Plane is located at the rectangle's center longitude and the rectangle's latitude that is closest to the equator. It rotates around the Z axis.
        // This results in a better fit than the obb approach for smaller rectangles, which orients with the rectangle's center normal.
        let mut planeOrigin = DVec3::from_radians(
            centerLongitude,
            latitudeNearestToEquator,
            Some(maximumHeight),
            Some(ellipsoid.radiiSquared),
        );
        planeOrigin.z = 0.0; // center the plane on the equator to simpify plane normal calculation
        let isPole = planeOrigin.x.abs() < EPSILON10 && planeOrigin.y.abs() < EPSILON10;
        let planeNormal = {
            if !isPole {
                planeOrigin.normalize()
            } else {
                DVec3::UNIT_X
            }
        };

        let planeYAxis = DVec3::UNIT_Z;
        let planeXAxis = planeNormal.cross(planeYAxis);
        plane = Plane::fromPointNormal(planeOrigin, planeNormal);

        // Get the horizon point relative to the center. This will be the farthest extent in the plane's X dimension.
        let horizonCartesian = DVec3::from_radians(
            centerLongitude + FRAC_2_PI,
            latitudeNearestToEquator,
            Some(maximumHeight),
            Some(ellipsoid.radiiSquared),
        );
        maxX = plane
            .projectPointOntoPlane(horizonCartesian)
            .dot(planeXAxis);
        minX = -maxX; // symmetrical

        // Get the min and max Y, using the height that will give the largest extent
        maxY = DVec3::from_radians(
            0.0,
            rectangle.north,
            {
                if fullyBelowEquator {
                    Some(minimumHeight)
                } else {
                    Some(maximumHeight)
                }
            },
            Some(ellipsoid.radiiSquared),
        )
        .z;
        minY = DVec3::from_radians(
            0.0,
            rectangle.south,
            {
                if fullyAboveEquator {
                    Some(minimumHeight)
                } else {
                    Some(maximumHeight)
                }
            },
            Some(ellipsoid.radiiSquared),
        )
        .z;

        let farZ = DVec3::from_radians(
            rectangle.east,
            latitudeNearestToEquator,
            Some(maximumHeight),
            Some(ellipsoid.radiiSquared),
        );
        minZ = plane.getPointDistance(farZ);
        maxZ = 0.0; // plane origin starts at maxZ already

        // min and max are local to the plane axes
        return fromPlaneExtents(
            planeOrigin,
            planeXAxis,
            planeYAxis,
            planeNormal,
            minX,
            maxX,
            minY,
            maxY,
            minZ,
            maxZ,
        );
    }
}
pub fn fromPlaneExtents(
    planeOrigin: DVec3,
    planeXAxis: DVec3,
    planeYAxis: DVec3,
    planeZAxis: DVec3,
    minimumX: f64,
    maximumX: f64,
    minimumY: f64,
    maximumY: f64,
    minimumZ: f64,
    maximumZ: f64,
) -> OrientedBoundingBox {
    let mut result = OrientedBoundingBox::default();

    let mut halfAxes = result.halfAxes;
    halfAxes.set_column(0, &planeXAxis);
    halfAxes.set_column(1, &planeYAxis);
    halfAxes.set_column(2, &planeZAxis);

    let mut centerOffset = DVec3::default();
    centerOffset.x = (minimumX + maximumX) / 2.0;
    centerOffset.y = (minimumY + maximumY) / 2.0;
    centerOffset.z = (minimumZ + maximumZ) / 2.0;

    let mut scale = DVec3::default();
    scale.x = (maximumX - minimumX) / 2.0;
    scale.y = (maximumY - minimumY) / 2.0;
    scale.z = (maximumZ - minimumZ) / 2.0;

    let center = result.center;
    centerOffset = halfAxes.multiply_by_vector(&centerOffset);
    result.center = planeOrigin + centerOffset;
    result.halfAxes = halfAxes.multiply_by_scale(scale);
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
//             mesh: meshes.add(Box3d::from_center_halfaxes(obb.center, obb.halfAxes).into()),
//             material: materials.add(Color::BLACK.into()),
//             ..Default::default()
//         });
//     }
// }
#[cfg(test)]
mod tests {
    use bevy::math::DVec3;

    use super::*;
    const positions: [DVec3; 6] = [
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
        assert_eq!(obb.halfAxes, DMat3::ZERO);
    }
    #[test]
    fn empty_points_work() {
        let points = vec![];
        let obb = OrientedBoundingBox::fromPoints(&points);
        assert_eq!(obb.center, DVec3::ZERO);
        assert_eq!(obb.halfAxes, DMat3::ZERO);
    }
    #[test]
    fn fromPointsCorrectScale() {
        let obb = OrientedBoundingBox::fromPoints(&positions);
        assert_eq!(obb.halfAxes, DMat3::from_scale3(DVec3::new(2.0, 3.0, 4.0)));
        assert_eq!(obb.center, DVec3::ZERO);
    }
}
