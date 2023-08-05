use crate::{
    ellipsoid::Ellipsoid, geometry::Rectangle, math::*, BoundingVolume, Intersect,
    OrientedBoundingBox, Plane,
};
use bevy::math::{DMat4, DVec3};

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct BoundingSphere {
    pub center: DVec3,
    pub radius: f64,
}
impl Default for BoundingSphere {
    fn default() -> Self {
        Self {
            center: DVec3::ZERO,
            radius: 0.0,
        }
    }
}
impl BoundingSphere {
    pub fn new(center: DVec3, radius: f64) -> Self {
        Self { center, radius }
    }
    // pub fn fromRectangle3D(rectangle,ellipsoid:Ellipsoid,surfaceHeight,)->Self{
    //     ellipsoid = defaultValue(ellipsoid, Ellipsoid.WGS84);
    //     surfaceHeight = defaultValue(surfaceHeight, 0.0);

    //     if !defined(result) {
    //       result = new BoundingSphere();
    //     }

    //     if !defined(rectangle) {
    //       result.center = DVec3.clone(DVec3.ZERO, result.center);
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

        let toRightCenter = rightCenter.subtract(leftCenter);
        let centerSeparation = toRightCenter.magnitude();

        if leftRadius >= centerSeparation + rightRadius {
            // Left sphere wins.
            return self.clone();
        }

        if rightRadius >= centerSeparation + leftRadius {
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
        if length == 1 {
            return boundingSpheres[0].clone();
        }

        if length == 2 {
            return BoundingSphere::union(boundingSpheres[0], boundingSpheres[1]);
        }

        let mut positions: Vec<DVec3> = vec![];
        for i in 0..length {
            positions.push(boundingSpheres[i].center);
        }
        let result = BoundingSphere::from_points(&positions);

        let center = result.center;
        let mut radius = result.radius;
        for i in 0..length {
            let tmp = boundingSpheres[i];
            radius = radius.max(center.distance(tmp.center) + tmp.radius)
        }
        return Self { center, radius };
    }
    pub fn from_corner_points(corner: DVec3, oppositeCorner: DVec3) -> Self {
        let center = (corner + oppositeCorner) * 0.5;
        let radius = center.distance(oppositeCorner);
        Self { center, radius }
    }
    pub fn from_ellipsoid(ellipsoid: &Ellipsoid) -> Self {
        let center = DVec3::ZERO;
        let radius = ellipsoid.maximum_radius;
        Self { center, radius }
    }
    pub fn from_points(positions: &Vec<DVec3>) -> Self {
        if positions.len() == 0 {
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
        if ySpan > maxSpan {
            maxSpan = ySpan;
            diameter1 = yMin;
            diameter2 = yMax;
        }
        if zSpan > maxSpan {
            maxSpan = zSpan;
            diameter1 = zMin;
            diameter2 = zMax;
        }

        // Calculate the center of the initial sphere found by Ritter's algorithm
        let mut ritterCenter = DVec3::ZERO;
        ritterCenter.x = (diameter1.x + diameter2.x) * 0.5;
        ritterCenter.y = (diameter1.y + diameter2.y) * 0.5;
        ritterCenter.z = (diameter1.z + diameter2.z) * 0.5;

        // Calculate the radius of the initial sphere found by Ritter's algorithm
        let mut radiusSquared = (diameter2 - ritterCenter).magnitude_squared();
        let mut ritterRadius = radiusSquared.sqrt();

        // Find the center of the sphere found using the Naive method.
        let mut minBoxPt = DVec3::ZERO;
        minBoxPt.x = xMin.x;
        minBoxPt.y = yMin.y;
        minBoxPt.z = zMin.z;

        let mut maxBoxPt = DVec3::ZERO;
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
    pub fn expand(&self, point: &DVec3) -> Self {
        let radius = point.subtract(self.center).magnitude();
        if radius > self.radius {
            return Self {
                center: self.center.clone(),
                radius: radius,
            };
        } else {
            return self.clone();
        }
    }
    // pub fn transform(&self, transform: DMat4) -> Self {
    //     let result = BoundingSphere::default();
    //     result.center = transform.multiply_point(&self.center);
    //     result.radius = self.radius * transform.get_maximum_scale();
    //     return result;
    // }
    pub fn from_rectangle_3d(
        &self,
        rectangle: &Rectangle,
        ellipsoid: Option<&Ellipsoid>,
        surfaceHeight: Option<f64>,
    ) -> Self {
        let positions = rectangle.subsample(ellipsoid, surfaceHeight);
        return Self::from_points(&positions);
    }

    pub fn from_oriented_bouding_box(oriented_bounding_box: &OrientedBoundingBox) -> Self {
        let halfAxes = oriented_bounding_box.halfAxes;
        let mut u = halfAxes.get_column(0);
        let v = halfAxes.get_column(1);
        let w = halfAxes.get_column(2);
        u = u + v;
        u = u + w;

        return Self {
            center: oriented_bounding_box.center.clone(),
            radius: u.magnitude(),
        };
    }
    pub fn intersectPlane(&self, plane: &Plane) -> Intersect {
        let center = self.center;
        let radius = self.radius;
        let normal = plane.normal;
        let distanceToPlane = normal.dot(center) + plane.distance;

        if distanceToPlane < -radius {
            // The center point is negative side of the plane normal
            return Intersect::OUTSIDE;
        } else if distanceToPlane < radius {
            // The center point is positive side of the plane, but radius extends beyond it; partial overlap
            return Intersect::INTERSECTING;
        }
        return Intersect::INSIDE;
    }
    pub fn from_vertices(positions: Vec<f64>) -> Self {
        let mut me = Self::new(DVec3::ZERO, 0.0);
        if positions.len() == 0 {
            return me;
        }
        let center = DVec3::ZERO;
        let stride = 3;

        let mut currentPos = DVec3::ZERO;
        currentPos.x = positions[0] + center.x;
        currentPos.y = positions[1] + center.y;
        currentPos.z = positions[2] + center.z;

        let mut xMin = currentPos.clone();
        let mut yMin = currentPos.clone();
        let mut zMin = currentPos.clone();

        let mut xMax = currentPos.clone();
        let mut yMax = currentPos.clone();
        let mut zMax = currentPos.clone();

        let numElements = positions.len();
        let mut i = 0;
        while i < numElements {
            let x = positions[i] + center.x;
            let y = positions[i + 1] + center.y;
            let z = positions[i + 2] + center.z;

            currentPos.x = x;
            currentPos.y = y;
            currentPos.z = z;

            // Store points containing the the smallest and largest components
            if (x < xMin.x) {
                xMin = currentPos.clone();
            }

            if (x > xMax.x) {
                xMax = currentPos.clone();
            }

            if (y < yMin.y) {
                yMin = currentPos.clone();
            }

            if (y > yMax.y) {
                yMax = currentPos.clone();
            }

            if (z < zMin.z) {
                zMin = currentPos.clone();
            }

            if (z > zMax.z) {
                zMax = currentPos.clone();
            }
            i += stride;
        }

        // Compute x-, y-, and z-spans (Squared distances b/n each component's min. and max.).
        let xSpan = xMax.subtract(xMin).magnitude_squared();
        let ySpan = yMax.subtract(yMin).magnitude_squared();
        let zSpan = zMax.subtract(zMin).magnitude_squared();

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
        let mut ritterCenter = DVec3::ZERO;
        ritterCenter.x = (diameter1.x + diameter2.x) * 0.5;
        ritterCenter.y = (diameter1.y + diameter2.y) * 0.5;
        ritterCenter.z = (diameter1.z + diameter2.z) * 0.5;

        // Calculate the radius of the initial sphere found by Ritter's algorithm
        let mut radiusSquared = diameter2.subtract(ritterCenter).magnitude_squared();
        let mut ritterRadius = radiusSquared.sqrt();

        // Find the center of the sphere found using the Naive method.
        let mut minBoxPt = DVec3::ZERO;
        minBoxPt.x = xMin.x;
        minBoxPt.y = yMin.y;
        minBoxPt.z = zMin.z;

        let mut maxBoxPt = DVec3::ZERO;
        maxBoxPt.x = xMax.x;
        maxBoxPt.y = yMax.y;
        maxBoxPt.z = zMax.z;

        let naiveCenter = minBoxPt.midpoint(maxBoxPt);

        // Begin 2nd pass to find naive radius and modify the ritter sphere.
        let mut naiveRadius = 0.0;
        i = 0;
        while i < numElements {
            currentPos.x = positions[i] + center.x;
            currentPos.y = positions[i + 1] + center.y;
            currentPos.z = positions[i + 2] + center.z;

            // Find the furthest point from the naive center to calculate the naive radius.
            let r = currentPos.subtract(naiveCenter).magnitude();
            if (r > naiveRadius) {
                naiveRadius = r;
            }

            // Make adjustments to the Ritter Sphere to include all points.
            let oldCenterToPointSquared = currentPos.subtract(ritterCenter).magnitude_squared();
            if (oldCenterToPointSquared > radiusSquared) {
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
            i += stride;
        }

        if (ritterRadius < naiveRadius) {
            me.center = ritterCenter.clone();
            me.radius = ritterRadius;
        } else {
            me.center = naiveCenter.clone();
            me.radius = naiveRadius;
        }
        return me;
    }
}
impl BoundingVolume for BoundingSphere {
    fn intersect_plane(&self, plane: &Plane) -> Intersect {
        return self.intersectPlane(plane);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const positionsCenter: DVec3 = DVec3::new(10000001.0, 0.0, 0.0);
    const center: DVec3 = DVec3::new(10000000.0, 0.0, 0.0);
    const positionsRadius: f64 = 1.0;
    fn getPositions() -> Vec<DVec3> {
        return vec![
            (center + DVec3::new(1.0, 0.0, 0.0)),
            (center + DVec3::new(2.0, 0.0, 0.0)),
            (center + DVec3::new(0.0, 0.0, 0.0)),
            (center + DVec3::new(1.0, 1.0, 0.0)),
            (center + DVec3::new(1.0, -1.0, 0.0)),
            (center + DVec3::new(1.0, 0.0, 1.0)),
            (center + DVec3::new(1.0, 0.0, -1.0)),
        ];
    }
    fn get_vertices() -> Vec<f64> {
        let ps = getPositions();
        let mut res = vec![];
        for i in 0..ps.len() {
            res.push(ps[i].x);
            res.push(ps[i].y);
            res.push(ps[i].z);
        }
        return res;
    }

    #[test]
    fn test_bounding_sphere() {
        let positions = vec![
            DVec3::new(0.0, 0.0, 0.0),
            DVec3::new(1.0, 0.0, 0.0),
            DVec3::new(0.0, 1.0, 0.0),
            DVec3::new(0.0, 0.0, 1.0),
            DVec3::new(1.0, 1.0, 1.0),
        ];
        let bounding_sphere = BoundingSphere::from_points(&positions);
        assert_eq!(bounding_sphere.center, DVec3::new(0.5, 0.5, 0.5));
        assert_eq!(bounding_sphere.radius, 0.8660254037844386);
    }
    #[test]
    fn from_points_work_with_empty_points() {
        let positions = vec![];
        let bounding_sphere = BoundingSphere::from_points(&positions);
        assert_eq!(bounding_sphere.center, DVec3::ZERO);
        assert_eq!(bounding_sphere.radius, 0.0);
    }
    #[test]
    fn from_points_work_with_one_point() {
        let expectedCenter = DVec3::new(1.0, 2.0, 3.0);
        let positions = vec![expectedCenter];
        let sphere = BoundingSphere::from_points(&positions);
        assert_eq!(sphere.center, expectedCenter);
        assert_eq!(sphere.radius, 0.0);
    }
    #[test]
    fn from_points_compute_center_from_points() {
        let positions = getPositions();
        let sphere = BoundingSphere::from_points(&positions);
        assert_eq!(sphere.center, positionsCenter);
        assert_eq!(sphere.radius, 1.0);
    }
    #[test]
    fn test_union_left_enclosed_right() {
        let bs1 = BoundingSphere::new(DVec3::ZERO, 3.0);
        let bs2 = BoundingSphere::new(DVec3::UNIT_X, 1.0);
        let res = bs1.union(&bs2);
        assert!(res.equal(&bs1));
    }
    #[test]
    fn test_union_right_encloded_left() {
        let bs1 = BoundingSphere::new(DVec3::ZERO, 1.0);
        let bs2 = BoundingSphere::new(DVec3::UNIT_X, 3.0);
        let res = bs1.union(&bs2);
        assert!(res.equal(&bs2));
    }
    #[test]
    fn test_union_return_tight_fit() {
        let bs1 = BoundingSphere::new(DVec3::UNIT_X.negate().multiply_by_scalar(3.0), 3.0);
        let bs2 = BoundingSphere::new(DVec3::UNIT_X, 1.0);
        let expected = BoundingSphere::new(DVec3::UNIT_X.negate().multiply_by_scalar(2.0), 4.0);
        let actual = bs1.union(&bs2);
        assert!(actual.equal(&expected));
    }
    #[test]
    fn test_from_vertices() {
        let shpere = BoundingSphere::from_vertices(get_vertices());
        let radius = &shpere.radius;
        let new_center = &shpere.center;
        let r = DVec3::new(*radius, *radius, *radius);
        let max = r + *new_center;
        let min = *new_center - r;
        let positions = getPositions();
        for i in 0..positions.len() {
            let cur = positions[i];
            assert!(cur.x <= max.x && cur.x >= min.x);
            assert!(cur.y <= max.y && cur.y >= min.y);
            assert!(cur.z <= max.z && cur.z >= min.z);
        }
    }
}
