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
    pub southwestCornerCartesian: DVec3,
    pub northeastCornerCartesian: DVec3,
    pub westNormal: DVec3,
    pub southNormal: DVec3,
    pub eastNormal: DVec3,
    pub northNormal: DVec3,
    pub oriented_bounding_box: Option<OrientedBoundingBox>,
    pub boundingSphere: Option<BoundingSphere>,
}
impl TileBoundingRegion {
    pub fn new(
        rectangle: &Rectangle,
        minimum_height: Option<f64>,
        maximum_height: Option<f64>,
        ellipsoid: Option<&Ellipsoid>,
        computeBoundingVolumes: Option<bool>,
    ) -> Self {
        let ellipsoid = ellipsoid.unwrap_or(&Ellipsoid::WGS84);
        let mut me = Self {
            rectangle: rectangle.clone(),
            minimum_height: minimum_height.unwrap_or(0.0),
            maximum_height: maximum_height.unwrap_or(0.0),
            southwestCornerCartesian: DVec3::ZERO,
            northeastCornerCartesian: DVec3::ZERO,
            westNormal: DVec3::ZERO,
            southNormal: DVec3::ZERO,
            eastNormal: DVec3::ZERO,
            northNormal: DVec3::ZERO,
            oriented_bounding_box: None,
            boundingSphere: None,
        };
        me.computeBox(&rectangle, &ellipsoid);
        if let Some(v) = computeBoundingVolumes {
            if v == true {
                me.computeBoundingVolumes(ellipsoid);
            }
        }
        return me;
    }
    pub fn distanceToCamera<P: Projection>(
        &self,
        positionWC: &DVec3,
        positionCartographic: &Cartographic,
        projection: &P,
    ) -> f64 {
        let regionResult =
            self.distanceToCameraRegion(positionWC, positionCartographic, projection);
        if let Some(v) = &self.oriented_bounding_box {
            let obbResult = v.distanceSquaredTo(&positionWC).sqrt();
            return regionResult.max(obbResult);
        }
        return regionResult;
    }
    pub fn distanceToCameraRegion<P: Projection>(
        &self,
        positionWC: &DVec3,
        positionCartographic: &Cartographic,
        projection: &P,
    ) -> f64 {
        let mut result = 0.0;
        if !self.rectangle.contains(positionCartographic) {
            let southwestCornerCartesian = self.southwestCornerCartesian;
            let northeastCornerCartesian = self.northeastCornerCartesian;
            let westNormal = self.westNormal;
            let southNormal = self.southNormal;
            let eastNormal = self.eastNormal;
            let northNormal = self.northNormal;
            let vectorFromSouthwestCorner = positionWC.subtract(southwestCornerCartesian);
            let distanceToWestPlane = vectorFromSouthwestCorner.dot(westNormal);
            let distanceToSouthPlane = vectorFromSouthwestCorner.dot(southNormal);

            let vectorFromNortheastCorner = positionWC.subtract(northeastCornerCartesian);
            let distanceToEastPlane = vectorFromNortheastCorner.dot(eastNormal);
            let distanceToNorthPlane = vectorFromNortheastCorner.dot(northNormal);

            if distanceToWestPlane > 0.0 {
                result += distanceToWestPlane * distanceToWestPlane;
            } else if distanceToEastPlane > 0.0 {
                result += distanceToEastPlane * distanceToEastPlane;
            }

            if distanceToSouthPlane > 0.0 {
                result += distanceToSouthPlane * distanceToSouthPlane;
            } else if distanceToNorthPlane > 0.0 {
                result += distanceToNorthPlane * distanceToNorthPlane;
            }
        }

        let cameraHeight;
        let minimum_height;
        let maximum_height;

        cameraHeight = positionCartographic.height;
        minimum_height = self.minimum_height;
        maximum_height = self.maximum_height;

        if cameraHeight > maximum_height {
            let distanceAboveTop = cameraHeight - maximum_height;
            result += distanceAboveTop * distanceAboveTop;
        } else if cameraHeight < minimum_height {
            let distanceBelowBottom = minimum_height - cameraHeight;
            result += distanceBelowBottom * distanceBelowBottom;
        }

        return result.sqrt();
    }
    pub fn get_bounding_volume(&self) -> Option<&OrientedBoundingBox> {
        self.oriented_bounding_box.as_ref()
    }
    pub fn get_bounding_sphere(&self) -> Option<&BoundingSphere> {
        self.boundingSphere.as_ref()
    }
    pub fn computeBoundingVolumes(&mut self, ellipsoid: &Ellipsoid) {
        let obb = OrientedBoundingBox::fromRectangle(
            &self.rectangle,
            Some(self.minimum_height),
            Some(self.maximum_height),
            Some(ellipsoid),
        );
        self.boundingSphere = Some(BoundingSphere::from_oriented_bouding_box(&obb));
        self.oriented_bounding_box = Some(obb);
    }
    pub fn computeBox(&mut self, rectangle: &Rectangle, ellipsoid: &Ellipsoid) {
        self.southwestCornerCartesian = ellipsoid.cartographicToCartesian(&rectangle.south_west());
        self.northeastCornerCartesian = ellipsoid.cartographicToCartesian(&rectangle.north_east());

        let mut cartographicScratch = Cartographic::ZERO;
        cartographicScratch.longitude = rectangle.west;
        cartographicScratch.latitude = (rectangle.south + rectangle.north) * 0.5;
        cartographicScratch.height = 0.0;
        let westernMidpointCartesian = ellipsoid.cartographicToCartesian(&cartographicScratch);

        // Compute the normal of the plane on the western edge of the tile.
        let westNormal = westernMidpointCartesian.cross(DVec3::UNIT_Z);
        self.westNormal = westNormal.normalize();

        // The middle latitude on the eastern edge.
        cartographicScratch.longitude = rectangle.east;
        let easternMidpointCartesian = ellipsoid.cartographicToCartesian(&cartographicScratch);

        // Compute the normal of the plane on the eastern edge of the tile.
        let eastNormal = DVec3::ZERO.cross(easternMidpointCartesian);
        self.eastNormal = eastNormal.normalize();

        let mut westVector = westernMidpointCartesian.subtract(easternMidpointCartesian);

        if westVector.magnitude() == 0.0 {
            westVector = westNormal.clone();
        }

        let eastWestNormal = westVector.normalize();
        let mut rayScratch = Ray::default();

        // Compute the normal of the plane bounding the southern edge of the tile.
        let south = rectangle.south;
        let southSurfaceNormal;

        if south > 0.0 {
            // Compute a plane that doesn't cut through the tile.
            cartographicScratch.longitude = (rectangle.west + rectangle.east) * 0.5;
            cartographicScratch.latitude = south;
            let southCenterCartesian = ellipsoid.cartographicToCartesian(&cartographicScratch);
            rayScratch.origin = southCenterCartesian;
            rayScratch.direction = eastWestNormal.clone();
            let westPlane =
                Plane::fromPointNormal(&self.southwestCornerCartesian, &self.westNormal);
            // Find a point that is on the west and the south planes
            self.southwestCornerCartesian =
                IntersectionTests::rayPlane(&rayScratch, &westPlane).unwrap();
            southSurfaceNormal = ellipsoid
                .geodeticSurfaceNormal(&southCenterCartesian)
                .unwrap();
        } else {
            southSurfaceNormal =
                ellipsoid.geodeticSurfaceNormalCartographic(&rectangle.south_east());
        }
        let southNormal = southSurfaceNormal.cross(westVector);
        self.southNormal = southNormal.normalize();

        // Compute the normal of the plane bounding the northern edge of the tile.
        let north = rectangle.north;
        let northSurfaceNormal;

        if north < 0.0 {
            // Compute a plane that doesn't cut through the tile.
            cartographicScratch.longitude = (rectangle.west + rectangle.east) * 0.5;
            cartographicScratch.latitude = north;
            let northCenterCartesian = ellipsoid.cartographicToCartesian(&cartographicScratch);
            rayScratch.origin = northCenterCartesian;
            rayScratch.direction = eastWestNormal.negate();
            let eastPlane =
                Plane::fromPointNormal(&self.northeastCornerCartesian, &self.eastNormal);
            // Find a point that is on the east and the north planes
            self.northeastCornerCartesian =
                IntersectionTests::rayPlane(&rayScratch, &eastPlane).unwrap();
            northSurfaceNormal = ellipsoid
                .geodeticSurfaceNormal(&northCenterCartesian)
                .unwrap();
        } else {
            northSurfaceNormal =
                ellipsoid.geodeticSurfaceNormalCartographic(&rectangle.north_west());
        }
        let northNormal = westVector.cross(northSurfaceNormal);
        self.northNormal = northNormal.normalize();
    }
}
