use bevy::math::DVec3;
use geodesy::Ellipsoid;

pub struct BoundingSphere {
    center: DVec3,
    radius: f64,
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
    //     return BoundingSphere.fromPoints(positions, result);
    // }
    pub fn fromPoints(positions: Vec<DVec3>) -> Self {
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
        let mut xSpan = (xMax - xMin).length_squared();
        let mut ySpan = (yMax - yMin).length_squared();
        let mut zSpan = (zMax - zMin).length_squared();

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
        let mut radiusSquared = (diameter2 - ritterCenter).length_squared();
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
            let r = (currentPos - naiveCenter).length();
            if r > naiveRadius {
                naiveRadius = r;
            }

            // Make adjustments to the Ritter Sphere to include all points.
            let oldCenterToPointSquared = (currentPos - ritterCenter).length_squared();
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
}

#[cfg(test)]
mod tests {
    use super::*;
    const positionsCenter: DVec3 = DVec3::new(10000001.0, 0.0, 0.0);
    const center: DVec3 = DVec3::new(10000000.0, 0.0, 0.0);
    const positionsRadius: f64 = 1.0;
    fn getPositions() -> Vec<DVec3> {
        return vec![
            center + DVec3::new(1.0, 0.0, 0.0),
            center + DVec3::new(2.0, 0.0, 0.0),
            center + DVec3::new(0.0, 0.0, 0.0),
            center + DVec3::new(1.0, 1.0, 0.0),
            center + DVec3::new(1.0, -1.0, 0.0),
            center + DVec3::new(1.0, 0.0, 1.0),
            center + DVec3::new(1.0, 0.0, -1.0),
        ];
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
        let bounding_sphere = BoundingSphere::fromPoints(positions);
        assert_eq!(bounding_sphere.center, DVec3::new(0.5, 0.5, 0.5));
        assert_eq!(bounding_sphere.radius, 0.8660254037844386);
    }
    #[test]
    fn from_points_work_with_empty_points() {
        let positions = vec![];
        let bounding_sphere = BoundingSphere::fromPoints(positions);
        assert_eq!(bounding_sphere.center, DVec3::ZERO);
        assert_eq!(bounding_sphere.radius, 0.0);
    }
    #[test]
    fn from_points_work_with_one_point() {
        let expectedCenter = DVec3::new(1.0, 2.0, 3.0);
        let sphere = BoundingSphere::fromPoints(vec![expectedCenter]);
        assert_eq!(sphere.center, expectedCenter);
        assert_eq!(sphere.radius, 0.0);
    }
    #[test]
    fn from_points_compute_center_from_points() {
        let sphere = BoundingSphere::fromPoints(getPositions());
        assert_eq!(sphere.center, positionsCenter);
        assert_eq!(sphere.radius, 1.0);
    }
}
