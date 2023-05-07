use bevy::math::DMat4;

use crate::{coord::Cartesian3, ellipsoid::Ellipsoid, geometry::Rectangle};

#[derive(Debug, Clone, PartialEq)]
pub struct BoundingSphere {
    center: Cartesian3,
    radius: f64,
}
impl Default for BoundingSphere {
    fn default() -> Self {
        Self {
            center: Cartesian3::ZERO,
            radius: 0.0,
        }
    }
}
impl BoundingSphere {
    pub fn new(center: Cartesian3, radius: f64) -> Self {
        Self { center, radius }
    }
    // pub fn fromRectangle3D(rectangle,ellipsoid:Ellipsoid,surfaceHeight,)->Self{
    //     ellipsoid = defaultValue(ellipsoid, Ellipsoid.WGS84);
    //     surfaceHeight = defaultValue(surfaceHeight, 0.0);

    //     if (!defined(result)) {
    //       result = new BoundingSphere();
    //     }

    //     if (!defined(rectangle)) {
    //       result.center = Cartesian3.clone(Cartesian3.ZERO, result.center);
    //       result.radius = 0.0;
    //       return result;
    //     }

    //     const positions = Rectangle.subsample(
    //       rectangle,
    //       ellipsoid,
    //       surfaceHeight,
    //       fromRectangle3DScratch
    //     );
    //     return BoundingSphere.from_points(positions, result);
    // }
    pub fn union(&self, right: &BoundingSphere) -> Self {
        let leftCenter = self.center;
        let leftRadius = self.radius;
        let rightCenter = right.center;
        let rightRadius = right.radius;

        let toRightCenter = rightCenter.subtract(&leftCenter);
        let centerSeparation = toRightCenter.magnitude();

        if (leftRadius >= centerSeparation + rightRadius) {
            // Left sphere wins.
            return self.clone();
        }

        if (rightRadius >= centerSeparation + leftRadius) {
            // Right sphere wins.
            return right.clone();
        }

        // There are two tangent points, one on far side of each sphere.
        let halfDistanceBetweenTangentPoints = (leftRadius + centerSeparation + rightRadius) * 0.5;

        // Compute the center point halfway between the two tangent points.
        let mut center = toRightCenter.multiply_by_scalar(
            (-leftRadius + halfDistanceBetweenTangentPoints) / centerSeparation,
        );
        center = center + leftCenter;
        return Self {
            center,
            radius: halfDistanceBetweenTangentPoints,
        };
    }
    pub fn from_bounding_spheres(boundingSpheres: Vec<&BoundingSphere>) -> Self {
        if boundingSpheres.len() == 0 {
            return Self::default();
        }

        let length = boundingSpheres.len();
        if (length == 1) {
            return boundingSpheres[0].clone();
        }

        if (length == 2) {
            return BoundingSphere::union(boundingSpheres[0], boundingSpheres[1]);
        }

        let mut positions: Vec<&Cartesian3> = vec![];
        for i in 0..length {
            positions.push(&boundingSpheres[i].center);
        }
        let result = BoundingSphere::from_points(positions);

        let center = result.center;
        let mut radius = result.radius;
        for i in 0..length {
            let tmp = boundingSpheres[i];
            radius = radius.max(center.distance(&tmp.center) + tmp.radius)
        }
        return Self { center, radius };
    }
    pub fn from_corner_points(corner: Cartesian3, oppositeCorner: Cartesian3) -> Self {
        let center = (corner + oppositeCorner) * 0.5;
        let radius = center.distance(&oppositeCorner);
        Self { center, radius }
    }
    pub fn from_ellipsoid(ellipsoid: &Ellipsoid) -> Self {
        let center = Cartesian3::ZERO;
        let radius = ellipsoid.maximumRadius;
        Self { center, radius }
    }
    pub fn from_points(positions: Vec<&Cartesian3>) -> Self {
        if (positions.len() == 0) {
            return Self::default();
        }
        let currentPos = positions[0].clone();

        let mut xMin = currentPos.clone();
        let mut yMin = currentPos.clone();
        let mut zMin = currentPos.clone();

        let mut xMax = currentPos.clone();
        let mut yMax = currentPos.clone();
        let mut zMax = currentPos.clone();

        let numPositions = positions.len();
        for i in 1..numPositions {
            let currentPos = positions[i].clone();
            let x = currentPos.x;
            let y = currentPos.y;
            let z = currentPos.z;

            // Store points containing the the smallest and largest components
            if x < xMin.x {
                xMin = currentPos.clone();
            }

            if x > xMax.x {
                xMax = currentPos.clone();
            }

            if y < yMin.y {
                yMin = currentPos.clone();
            }

            if y > yMax.y {
                yMax = currentPos.clone();
            }

            if z < zMin.z {
                zMin = currentPos.clone();
            }

            if z > zMax.z {
                zMax = currentPos.clone();
            }
        }

        // Compute x-, y-, and z-spans (Squared distances b/n each component's min. and max.).
        let mut xSpan = (xMax - xMin).magnitude_squared();
        let mut ySpan = (yMax - yMin).magnitude_squared();
        let mut zSpan = (zMax - zMin).magnitude_squared();

        // Set the diameter endpoints to the largest span.
        let mut diameter1 = xMin;
        let mut diameter2 = xMax;
        let mut maxSpan = xSpan;
        if (ySpan > maxSpan) {
            maxSpan = ySpan;
            diameter1 = yMin;
            diameter2 = yMax;
        }
        if (zSpan > maxSpan) {
            maxSpan = zSpan;
            diameter1 = zMin;
            diameter2 = zMax;
        }

        // Calculate the center of the initial sphere found by Ritter's algorithm
        let mut ritterCenter = Cartesian3::ZERO;
        ritterCenter.x = (diameter1.x + diameter2.x) * 0.5;
        ritterCenter.y = (diameter1.y + diameter2.y) * 0.5;
        ritterCenter.z = (diameter1.z + diameter2.z) * 0.5;

        // Calculate the radius of the initial sphere found by Ritter's algorithm
        let mut radiusSquared = (diameter2 - ritterCenter).magnitude_squared();
        let mut ritterRadius = radiusSquared.sqrt();

        // Find the center of the sphere found using the Naive method.
        let mut minBoxPt = Cartesian3::ZERO;
        minBoxPt.x = xMin.x;
        minBoxPt.y = yMin.y;
        minBoxPt.z = zMin.z;

        let mut maxBoxPt = Cartesian3::ZERO;
        maxBoxPt.x = xMax.x;
        maxBoxPt.y = yMax.y;
        maxBoxPt.z = zMax.z;

        let mut naiveCenter = (minBoxPt + maxBoxPt) * 0.5;

        // Begin 2nd pass to find naive radius and modify the ritter sphere.
        let mut naiveRadius: f64 = 0.;
        for i in 0..numPositions {
            let currentPos = positions[i].clone();

            // Find the furthest point from the naive center to calculate the naive radius.
            let r = (currentPos - naiveCenter).magnitude();
            if r > naiveRadius {
                naiveRadius = r;
            }

            // Make adjustments to the Ritter Sphere to include all points.
            let oldCenterToPointSquared = (currentPos - ritterCenter).magnitude_squared();
            if oldCenterToPointSquared > radiusSquared {
                let oldCenterToPoint = oldCenterToPointSquared.sqrt();
                // Calculate new radius to include the point that lies outside
                ritterRadius = (ritterRadius + oldCenterToPoint) * 0.5;
                radiusSquared = ritterRadius * ritterRadius;
                // Calculate center of new Ritter sphere
                let oldToNew = oldCenterToPoint - ritterRadius;
                ritterCenter.x =
                    (ritterRadius * ritterCenter.x + oldToNew * currentPos.x) / oldCenterToPoint;
                ritterCenter.y =
                    (ritterRadius * ritterCenter.y + oldToNew * currentPos.y) / oldCenterToPoint;
                ritterCenter.z =
                    (ritterRadius * ritterCenter.z + oldToNew * currentPos.z) / oldCenterToPoint;
            }
        }
        let mut result = Self::default();
        if ritterRadius < naiveRadius {
            result.center = ritterCenter;
            result.radius = ritterRadius;
        } else {
            result.center = naiveCenter;
            result.radius = naiveRadius;
        }
        return result;
    }
    pub fn equal(&self, other: &BoundingSphere) -> bool {
        return self.center == other.center && self.radius == other.radius;
    }
    pub fn expand(&self, point: &Cartesian3) -> Self {
        let radius = point.subtract(&self.center).magnitude();
        if radius > self.radius {
            return Self {
                center: self.center.clone(),
                radius: radius,
            };
        } else {
            return sphere.clone();
        }
    }
    pub fn transform(&self, transform: DMat4) -> Self {
        let result = BoundingSphere::default();
        result.center = transform.multiply_point(&self.center);
        result.radius = self.radius * transform.get_maximum_scale();
        return result;
    }
    pub fn from_rectangle_3d(
        &self,
        rectangle: &Rectangle,
        ellipsoid: Option<&Ellipsoid>,
        surfaceHeight: Option<f64>,
    ) -> Self {
        let positions = rectangle.subsample(ellipsoid, surfaceHeight);
        return Self::from_points(&positions);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const positionsCenter: Cartesian3 = Cartesian3::new(10000001.0, 0.0, 0.0);
    const center: Cartesian3 = Cartesian3::new(10000000.0, 0.0, 0.0);
    const positionsRadius: f64 = 1.0;
    fn getPositions() -> Vec<Cartesian3> {
        return vec![
            (center + Cartesian3::new(1.0, 0.0, 0.0)),
            (center + Cartesian3::new(2.0, 0.0, 0.0)),
            (center + Cartesian3::new(0.0, 0.0, 0.0)),
            (center + Cartesian3::new(1.0, 1.0, 0.0)),
            (center + Cartesian3::new(1.0, -1.0, 0.0)),
            (center + Cartesian3::new(1.0, 0.0, 1.0)),
            (center + Cartesian3::new(1.0, 0.0, -1.0)),
        ];
    }

    #[test]
    fn test_bounding_sphere() {
        let positions = vec![
            Cartesian3::new(0.0, 0.0, 0.0),
            Cartesian3::new(1.0, 0.0, 0.0),
            Cartesian3::new(0.0, 1.0, 0.0),
            Cartesian3::new(0.0, 0.0, 1.0),
            Cartesian3::new(1.0, 1.0, 1.0),
        ];
        let bounding_sphere = BoundingSphere::from_points(positions.iter().map(|x| x).collect());
        assert_eq!(bounding_sphere.center, Cartesian3::new(0.5, 0.5, 0.5));
        assert_eq!(bounding_sphere.radius, 0.8660254037844386);
    }
    #[test]
    fn from_points_work_with_empty_points() {
        let positions = vec![];
        let bounding_sphere = BoundingSphere::from_points(positions);
        assert_eq!(bounding_sphere.center, Cartesian3::ZERO);
        assert_eq!(bounding_sphere.radius, 0.0);
    }
    #[test]
    fn from_points_work_with_one_point() {
        let expectedCenter = Cartesian3::new(1.0, 2.0, 3.0);
        let sphere = BoundingSphere::from_points(vec![&expectedCenter]);
        assert_eq!(sphere.center, expectedCenter);
        assert_eq!(sphere.radius, 0.0);
    }
    #[test]
    fn from_points_compute_center_from_points() {
        let sphere = BoundingSphere::from_points(getPositions().iter().map(|x| x).collect());
        assert_eq!(sphere.center, positionsCenter);
        assert_eq!(sphere.radius, 1.0);
    }
    #[test]
    fn test_union_left_enclosed_right() {
        let bs1 = BoundingSphere::new(Cartesian3::ZERO, 3.0);
        let bs2 = BoundingSphere::new(Cartesian3::UNIT_X, 1.0);
        let res = bs1.union(&bs2);
        assert!(res.equal(&bs1));
    }
    #[test]
    fn test_union_right_encloded_left() {
        let bs1 = BoundingSphere::new(Cartesian3::ZERO, 1.0);
        let bs2 = BoundingSphere::new(Cartesian3::UNIT_X, 3.0);
        let res = bs1.union(&bs2);
        assert!(res.equal(&bs2));
    }
    #[test]
    fn test_union_return_tight_fit() {
        let bs1 = BoundingSphere::new(Cartesian3::UNIT_X.negate().multiply_by_scalar(3.0), 3.0);
        let bs2 = BoundingSphere::new(Cartesian3::UNIT_X, 1.0);
        let expected =
            BoundingSphere::new(Cartesian3::UNIT_X.negate().multiply_by_scalar(2.0), 4.0);
        let actual = bs1.union(&bs2);
        assert!(actual.equal(&expected));
    }
}
