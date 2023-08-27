use crate::{
    ellipsoid::Ellipsoid, geometry::Rectangle, math::*, BoundingVolume, Intersect,
    OrientedBoundingBox, Plane,
};
use bevy::math::{DVec3};

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
    // pub fn fromRectangle3D(rectangle,ellipsoid:Ellipsoid,surface_height,)->Self{
    //     ellipsoid = defaultValue(ellipsoid, Ellipsoid.WGS84);
    //     surface_height = defaultValue(surface_height, 0.0);

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
    //       surface_height,
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
            // LEFT sphere wins.
            return self.clone();
        }

        if rightRadius >= centerSeparation + leftRadius {
            // RIGHT sphere wins.
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
    pub fn from_corner_points(corner: DVec3, opposite_corner: DVec3) -> Self {
        let center = (corner + opposite_corner) * 0.5;
        let radius = center.distance(opposite_corner);
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
        let current_pos = positions[0].clone();

        let mut x_min = current_pos.clone();
        let mut y_min = current_pos.clone();
        let mut z_min = current_pos.clone();

        let mut x_max = current_pos.clone();
        let mut y_max = current_pos.clone();
        let mut z_max = current_pos.clone();

        let num_positions = positions.len();
        for i in 1..num_positions {
            let current_pos = positions[i].clone();
            let x = current_pos.x;
            let y = current_pos.y;
            let z = current_pos.z;

            // Store points containing the the smallest and largest components
            if x < x_min.x {
                x_min = current_pos.clone();
            }

            if x > x_max.x {
                x_max = current_pos.clone();
            }

            if y < y_min.y {
                y_min = current_pos.clone();
            }

            if y > y_max.y {
                y_max = current_pos.clone();
            }

            if z < z_min.z {
                z_min = current_pos.clone();
            }

            if z > z_max.z {
                z_max = current_pos.clone();
            }
        }

        // Compute x-, y-, and z-spans (Squared distances b/n each component's min. and max.).
        let x_span = (x_max - x_min).magnitude_squared();
        let y_span = (y_max - y_min).magnitude_squared();
        let z_span = (z_max - z_min).magnitude_squared();

        // Set the diameter endpoints to the largest span.
        let mut diameter1 = x_min;
        let mut diameter2 = x_max;
        let mut max_span = x_span;
        if y_span > max_span {
            max_span = y_span;
            diameter1 = y_min;
            diameter2 = y_max;
        }
        if z_span > max_span {
            max_span = z_span;
            diameter1 = z_min;
            diameter2 = z_max;
        }

        // Calculate the center of the initial sphere found by Ritter's algorithm
        let mut ritter_center = DVec3::ZERO;
        ritter_center.x = (diameter1.x + diameter2.x) * 0.5;
        ritter_center.y = (diameter1.y + diameter2.y) * 0.5;
        ritter_center.z = (diameter1.z + diameter2.z) * 0.5;

        // Calculate the radius of the initial sphere found by Ritter's algorithm
        let mut radius_squared = (diameter2 - ritter_center).magnitude_squared();
        let mut ritter_radius = radius_squared.sqrt();

        // Find the center of the sphere found using the Naive method.
        let mut min_box_pt = DVec3::ZERO;
        min_box_pt.x = x_min.x;
        min_box_pt.y = y_min.y;
        min_box_pt.z = z_min.z;

        let mut max_box_pt = DVec3::ZERO;
        max_box_pt.x = x_max.x;
        max_box_pt.y = y_max.y;
        max_box_pt.z = z_max.z;

        let naive_center = (min_box_pt + max_box_pt) * 0.5;

        // Begin 2nd pass to find naive radius and modify the ritter sphere.
        let mut naive_radius: f64 = 0.;
        for i in 0..num_positions {
            let current_pos = positions[i].clone();

            // Find the furthest point from the naive center to calculate the naive radius.
            let r = (current_pos - naive_center).magnitude();
            if r > naive_radius {
                naive_radius = r;
            }

            // Make adjustments to the Ritter Sphere to include all points.
            let old_center_to_point_squared = (current_pos - ritter_center).magnitude_squared();
            if old_center_to_point_squared > radius_squared {
                let old_center_to_point = old_center_to_point_squared.sqrt();
                // Calculate new radius to include the point that lies outside
                ritter_radius = (ritter_radius + old_center_to_point) * 0.5;
                radius_squared = ritter_radius * ritter_radius;
                // Calculate center of new Ritter sphere
                let old_to_new = old_center_to_point - ritter_radius;
                ritter_center.x = (ritter_radius * ritter_center.x + old_to_new * current_pos.x)
                    / old_center_to_point;
                ritter_center.y = (ritter_radius * ritter_center.y + old_to_new * current_pos.y)
                    / old_center_to_point;
                ritter_center.z = (ritter_radius * ritter_center.z + old_to_new * current_pos.z)
                    / old_center_to_point;
            }
        }
        let mut result = Self::default();
        if ritter_radius < naive_radius {
            result.center = ritter_center;
            result.radius = ritter_radius;
        } else {
            result.center = naive_center;
            result.radius = naive_radius;
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
        surface_height: Option<f64>,
    ) -> Self {
        let positions = rectangle.subsample(ellipsoid, surface_height);
        return Self::from_points(&positions);
    }

    pub fn from_oriented_bouding_box(oriented_bounding_box: &OrientedBoundingBox) -> Self {
        let half_axes = oriented_bounding_box.half_axes;
        let mut u = half_axes.get_column(0);
        let v = half_axes.get_column(1);
        let w = half_axes.get_column(2);
        u = u + v;
        u = u + w;

        return Self {
            center: oriented_bounding_box.center.clone(),
            radius: u.magnitude(),
        };
    }
    pub fn intersect_plane(&self, plane: &Plane) -> Intersect {
        let center = self.center;
        let radius = self.radius;
        let normal = plane.normal;
        let distance_to_plane = normal.dot(center) + plane.distance;

        if distance_to_plane < -radius {
            // The center point is negative side of the plane normal
            return Intersect::OUTSIDE;
        } else if distance_to_plane < radius {
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

        let mut current_pos = DVec3::ZERO;
        current_pos.x = positions[0] + center.x;
        current_pos.y = positions[1] + center.y;
        current_pos.z = positions[2] + center.z;

        let mut x_min = current_pos.clone();
        let mut y_min = current_pos.clone();
        let mut z_min = current_pos.clone();

        let mut x_max = current_pos.clone();
        let mut y_max = current_pos.clone();
        let mut z_max = current_pos.clone();

        let num_elements = positions.len();
        let mut i = 0;
        while i < num_elements {
            let x = positions[i] + center.x;
            let y = positions[i + 1] + center.y;
            let z = positions[i + 2] + center.z;

            current_pos.x = x;
            current_pos.y = y;
            current_pos.z = z;

            // Store points containing the the smallest and largest components
            if x < x_min.x {
                x_min = current_pos.clone();
            }

            if x > x_max.x {
                x_max = current_pos.clone();
            }

            if y < y_min.y {
                y_min = current_pos.clone();
            }

            if y > y_max.y {
                y_max = current_pos.clone();
            }

            if z < z_min.z {
                z_min = current_pos.clone();
            }

            if z > z_max.z {
                z_max = current_pos.clone();
            }
            i += stride;
        }

        // Compute x-, y-, and z-spans (Squared distances b/n each component's min. and max.).
        let x_span = x_max.subtract(x_min).magnitude_squared();
        let y_span = y_max.subtract(y_min).magnitude_squared();
        let z_span = z_max.subtract(z_min).magnitude_squared();

        // Set the diameter endpoints to the largest span.
        let mut diameter1 = x_min;
        let mut diameter2 = x_max;
        let mut max_span = x_span;
        if y_span > max_span {
            max_span = y_span;
            diameter1 = y_min;
            diameter2 = y_max;
        }
        if z_span > max_span {
            max_span = z_span;
            diameter1 = z_min;
            diameter2 = z_max;
        }

        // Calculate the center of the initial sphere found by Ritter's algorithm
        let mut ritter_center = DVec3::ZERO;
        ritter_center.x = (diameter1.x + diameter2.x) * 0.5;
        ritter_center.y = (diameter1.y + diameter2.y) * 0.5;
        ritter_center.z = (diameter1.z + diameter2.z) * 0.5;

        // Calculate the radius of the initial sphere found by Ritter's algorithm
        let mut radius_squared = diameter2.subtract(ritter_center).magnitude_squared();
        let mut ritter_radius = radius_squared.sqrt();

        // Find the center of the sphere found using the Naive method.
        let mut min_box_pt = DVec3::ZERO;
        min_box_pt.x = x_min.x;
        min_box_pt.y = y_min.y;
        min_box_pt.z = z_min.z;

        let mut max_box_pt = DVec3::ZERO;
        max_box_pt.x = x_max.x;
        max_box_pt.y = y_max.y;
        max_box_pt.z = z_max.z;

        let naive_center = min_box_pt.midpoint(max_box_pt);

        // Begin 2nd pass to find naive radius and modify the ritter sphere.
        let mut naive_radius = 0.0;
        i = 0;
        while i < num_elements {
            current_pos.x = positions[i] + center.x;
            current_pos.y = positions[i + 1] + center.y;
            current_pos.z = positions[i + 2] + center.z;

            // Find the furthest point from the naive center to calculate the naive radius.
            let r = current_pos.subtract(naive_center).magnitude();
            if r > naive_radius {
                naive_radius = r;
            }

            // Make adjustments to the Ritter Sphere to include all points.
            let old_center_to_point_squared =
                current_pos.subtract(ritter_center).magnitude_squared();
            if old_center_to_point_squared > radius_squared {
                let old_center_to_point = old_center_to_point_squared.sqrt();
                // Calculate new radius to include the point that lies outside
                ritter_radius = (ritter_radius + old_center_to_point) * 0.5;
                radius_squared = ritter_radius * ritter_radius;
                // Calculate center of new Ritter sphere
                let old_to_new = old_center_to_point - ritter_radius;
                ritter_center.x = (ritter_radius * ritter_center.x + old_to_new * current_pos.x)
                    / old_center_to_point;
                ritter_center.y = (ritter_radius * ritter_center.y + old_to_new * current_pos.y)
                    / old_center_to_point;
                ritter_center.z = (ritter_radius * ritter_center.z + old_to_new * current_pos.z)
                    / old_center_to_point;
            }
            i += stride;
        }

        if ritter_radius < naive_radius {
            me.center = ritter_center.clone();
            me.radius = ritter_radius;
        } else {
            me.center = naive_center.clone();
            me.radius = naive_radius;
        }
        return me;
    }
}
impl BoundingVolume for BoundingSphere {
    fn intersect_plane(&self, plane: &Plane) -> Intersect {
        return self.intersect_plane(plane);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    const POSITIONS_CENTER: DVec3 = DVec3::new(10000001.0, 0.0, 0.0);
    const CENTER: DVec3 = DVec3::new(10000000.0, 0.0, 0.0);
    const POSITIONS_RADIUS: f64 = 1.0;
    fn get_positions() -> Vec<DVec3> {
        return vec![
            (CENTER + DVec3::new(1.0, 0.0, 0.0)),
            (CENTER + DVec3::new(2.0, 0.0, 0.0)),
            (CENTER + DVec3::new(0.0, 0.0, 0.0)),
            (CENTER + DVec3::new(1.0, 1.0, 0.0)),
            (CENTER + DVec3::new(1.0, -1.0, 0.0)),
            (CENTER + DVec3::new(1.0, 0.0, 1.0)),
            (CENTER + DVec3::new(1.0, 0.0, -1.0)),
        ];
    }
    fn get_vertices() -> Vec<f64> {
        let ps = get_positions();
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
        let expected_center = DVec3::new(1.0, 2.0, 3.0);
        let positions = vec![expected_center];
        let sphere = BoundingSphere::from_points(&positions);
        assert_eq!(sphere.center, expected_center);
        assert_eq!(sphere.radius, 0.0);
    }
    #[test]
    fn from_points_compute_center_from_points() {
        let positions = get_positions();
        let sphere = BoundingSphere::from_points(&positions);
        assert_eq!(sphere.center, POSITIONS_CENTER);
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
        let positions = get_positions();
        for i in 0..positions.len() {
            let cur = positions[i];
            assert!(cur.x <= max.x && cur.x >= min.x);
            assert!(cur.y <= max.y && cur.y >= min.y);
            assert!(cur.z <= max.z && cur.z >= min.z);
        }
    }
}
