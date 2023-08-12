use crate::{
    BoundingSphere, Cartesian3, Cartographic, Ellipsoid, OrientedBoundingBox, Plane, Projection,
    Ray, Rectangle,
};
use bevy::math::DVec3;

use crate::IntersectionTests;
#[derive(Default, Clone, Debug)]
pub struct TileBoundingRegion {
    pub rectangle: Rectangle,
    pub minimum_height: f64,
    pub maximum_height: f64,
    pub south_west_corner_cartesian: DVec3,
    pub north_east_corner_cartesian: DVec3,
    pub west_normal: DVec3,
    pub south_normal: DVec3,
    pub east_normal: DVec3,
    pub north_normal: DVec3,
    pub oriented_bounding_box: Option<OrientedBoundingBox>,
    pub bounding_sphere: Option<BoundingSphere>,
}
impl TileBoundingRegion {
    pub fn new(
        rectangle: &Rectangle,
        minimum_height: Option<f64>,
        maximum_height: Option<f64>,
        ellipsoid: Option<&Ellipsoid>,
        compute_bounding_volumes: Option<bool>,
    ) -> Self {
        let ellipsoid = ellipsoid.unwrap_or(&Ellipsoid::WGS84);
        let mut me = Self {
            rectangle: rectangle.clone(),
            minimum_height: minimum_height.unwrap_or(0.0),
            maximum_height: maximum_height.unwrap_or(0.0),
            south_west_corner_cartesian: DVec3::ZERO,
            north_east_corner_cartesian: DVec3::ZERO,
            west_normal: DVec3::ZERO,
            south_normal: DVec3::ZERO,
            east_normal: DVec3::ZERO,
            north_normal: DVec3::ZERO,
            oriented_bounding_box: None,
            bounding_sphere: None,
        };
        me.compute_box(&rectangle, &ellipsoid);
        if let Some(v) = compute_bounding_volumes {
            if v == true {
                me.compute_bounding_volumes(ellipsoid);
            }
        }
        return me;
    }
    pub fn distance_to_camera<P: Projection>(
        &self,
        position_wc: &DVec3,
        position_cartographic: &Cartographic,
        projection: &P,
    ) -> f64 {
        let region_result =
            self.distance_to_camera_region(position_wc, position_cartographic, projection);
        if let Some(v) = &self.oriented_bounding_box {
            let obb_result = v.distance_squared_to(&position_wc).sqrt();
            return region_result.max(obb_result);
        }
        return region_result;
    }
    pub fn distance_to_camera_region<P: Projection>(
        &self,
        position_wc: &DVec3,
        position_cartographic: &Cartographic,
        projection: &P,
    ) -> f64 {
        let mut result = 0.0;
        if !self.rectangle.contains(position_cartographic) {
            let south_west_corner_cartesian = self.south_west_corner_cartesian;
            let north_east_corner_cartesian = self.north_east_corner_cartesian;
            let west_normal = self.west_normal;
            let south_normal = self.south_normal;
            let east_normal = self.east_normal;
            let north_normal = self.north_normal;
            let vector_from_south_west_corner = position_wc.subtract(south_west_corner_cartesian);
            let distance_to_west_plane = vector_from_south_west_corner.dot(west_normal);
            let distance_to_south_plane = vector_from_south_west_corner.dot(south_normal);

            let vector_from_north_east_corner = position_wc.subtract(north_east_corner_cartesian);
            let distance_to_east_plane = vector_from_north_east_corner.dot(east_normal);
            let distance_to_north_plane = vector_from_north_east_corner.dot(north_normal);

            if distance_to_west_plane > 0.0 {
                result += distance_to_west_plane * distance_to_west_plane;
            } else if distance_to_east_plane > 0.0 {
                result += distance_to_east_plane * distance_to_east_plane;
            }

            if distance_to_south_plane > 0.0 {
                result += distance_to_south_plane * distance_to_south_plane;
            } else if distance_to_north_plane > 0.0 {
                result += distance_to_north_plane * distance_to_north_plane;
            }
        }

        let camera_height;
        let minimum_height;
        let maximum_height;

        camera_height = position_cartographic.height;
        minimum_height = self.minimum_height;
        maximum_height = self.maximum_height;

        if camera_height > maximum_height {
            let distance_above_top = camera_height - maximum_height;
            result += distance_above_top * distance_above_top;
        } else if camera_height < minimum_height {
            let distance_below_bottom = minimum_height - camera_height;
            result += distance_below_bottom * distance_below_bottom;
        }

        return result.sqrt();
    }
    pub fn get_bounding_volume(&self) -> Option<&OrientedBoundingBox> {
        self.oriented_bounding_box.as_ref()
    }
    pub fn get_bounding_sphere(&self) -> Option<&BoundingSphere> {
        self.bounding_sphere.as_ref()
    }
    pub fn compute_bounding_volumes(&mut self, ellipsoid: &Ellipsoid) {
        let obb = OrientedBoundingBox::from_rectangle(
            &self.rectangle,
            Some(self.minimum_height),
            Some(self.maximum_height),
            Some(ellipsoid),
        );
        self.bounding_sphere = Some(BoundingSphere::from_oriented_bouding_box(&obb));
        self.oriented_bounding_box = Some(obb);
    }
    pub fn compute_box(&mut self, rectangle: &Rectangle, ellipsoid: &Ellipsoid) {
        self.south_west_corner_cartesian =
            ellipsoid.cartographic_to_cartesian(&rectangle.south_west());
        self.north_east_corner_cartesian =
            ellipsoid.cartographic_to_cartesian(&rectangle.north_east());

        let mut cartographic_scratch = Cartographic::ZERO;
        cartographic_scratch.longitude = rectangle.west;
        cartographic_scratch.latitude = (rectangle.south + rectangle.north) * 0.5;
        cartographic_scratch.height = 0.0;
        let western_midpoint_cartesian = ellipsoid.cartographic_to_cartesian(&cartographic_scratch);

        // Compute the normal of the plane on the western edge of the tile.
        let west_normal = western_midpoint_cartesian.cross(DVec3::UNIT_Z);
        self.west_normal = west_normal.normalize();

        // The middle latitude on the eastern edge.
        cartographic_scratch.longitude = rectangle.east;
        let eastern_midpoint_cartesian = ellipsoid.cartographic_to_cartesian(&cartographic_scratch);

        // Compute the normal of the plane on the eastern edge of the tile.
        let east_normal = DVec3::ZERO.cross(eastern_midpoint_cartesian);
        self.east_normal = east_normal.normalize();

        let mut west_vector = western_midpoint_cartesian.subtract(eastern_midpoint_cartesian);

        if west_vector.magnitude() == 0.0 {
            west_vector = west_normal.clone();
        }

        let east_west_normal = west_vector.normalize();
        let mut ray_scratch = Ray::default();

        // Compute the normal of the plane bounding the southern edge of the tile.
        let south = rectangle.south;
        let south_surface_normal;

        if south > 0.0 {
            // Compute a plane that doesn't cut through the tile.
            cartographic_scratch.longitude = (rectangle.west + rectangle.east) * 0.5;
            cartographic_scratch.latitude = south;
            let south_center_cartesian = ellipsoid.cartographic_to_cartesian(&cartographic_scratch);
            ray_scratch.origin = south_center_cartesian;
            ray_scratch.direction = east_west_normal.clone();
            let west_plane =
                Plane::from_point_normal(&self.south_west_corner_cartesian, &self.west_normal);
            // Find a point that is on the west and the south planes
            self.south_west_corner_cartesian =
                IntersectionTests::rayPlane(&ray_scratch, &west_plane).unwrap();
            south_surface_normal = ellipsoid
                .geodetic_surface_normal(&south_center_cartesian)
                .unwrap();
        } else {
            south_surface_normal =
                ellipsoid.geodetic_surface_normal_cartographic(&rectangle.south_east());
        }
        let south_normal = south_surface_normal.cross(west_vector);
        self.south_normal = south_normal.normalize();

        // Compute the normal of the plane bounding the northern edge of the tile.
        let north = rectangle.north;
        let north_surface_normal;

        if north < 0.0 {
            // Compute a plane that doesn't cut through the tile.
            cartographic_scratch.longitude = (rectangle.west + rectangle.east) * 0.5;
            cartographic_scratch.latitude = north;
            let north_center_cartesian = ellipsoid.cartographic_to_cartesian(&cartographic_scratch);
            ray_scratch.origin = north_center_cartesian;
            ray_scratch.direction = east_west_normal.negate();
            let east_plane =
                Plane::from_point_normal(&self.north_east_corner_cartesian, &self.east_normal);
            // Find a point that is on the east and the north planes
            self.north_east_corner_cartesian =
                IntersectionTests::rayPlane(&ray_scratch, &east_plane).unwrap();
            north_surface_normal = ellipsoid
                .geodetic_surface_normal(&north_center_cartesian)
                .unwrap();
        } else {
            north_surface_normal =
                ellipsoid.geodetic_surface_normal_cartographic(&rectangle.north_west());
        }
        let north_normal = west_vector.cross(north_surface_normal);
        self.north_normal = north_normal.normalize();
    }
}
