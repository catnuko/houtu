use bevy::math::{DMat4, DVec3};

use crate::{Cartesian3, CullingVolume, Matrix4};

#[derive(Clone)]
pub struct PerspectiveOffCenterFrustum {
    pub left: f64,
    _left: f64,
    pub right: f64,
    _right: f64,
    pub top: f64,
    _top: f64,
    pub bottom: f64,
    _bottom: f64,
    pub near: f64,
    _near: f64,
    pub far: f64,
    _far: f64,
    _culling_volume: CullingVolume,
    pub perspective_matrix: DMat4,
    pub infinite_perspective: DMat4,
}
impl PerspectiveOffCenterFrustum {
    pub fn new() -> Self {
        Self {
            left: -1.0,
            _left: -1.0,
            right: -1.0,
            _right: -1.0,
            top: -1.0,
            _top: -1.0,
            bottom: -1.0,
            _bottom: -1.0,
            near: -1.0,
            _near: -1.0,
            far: -1.0,
            _far: -1.0,
            _culling_volume: CullingVolume::new(None),
            infinite_perspective: DMat4::ZERO,
            perspective_matrix: DMat4::ZERO,
        }
    }
    fn update_self(&mut self) {
        let t = self.top;
        let b = self.bottom;
        let r = self.right;
        let l = self.left;
        let n = self.near;
        let f = self.far;

        if t != self._top
            || b != self._bottom
            || l != self._left
            || r != self._right
            || n != self._near
            || f != self._far
        {
            //>>includeStart('debug', pragmas.debug);
            if self.near <= 0.0 || self.near > self.far {
                panic!("near must be greater than zero and less than far.");
            }
            //>>includeEnd('debug');

            self._left = l;
            self._right = r;
            self._top = t;
            self._bottom = b;
            self._near = n;
            self._far = f;
            self.perspective_matrix = DMat4::compute_perspective_off_center(l, r, b, t, n, f);
            self.infinite_perspective =
                DMat4::compute_infinite_perspective_off_center(l, r, b, t, n);
        }
    }
    pub fn get_projection_matrix(&mut self) -> &DMat4 {
        self.update_self();
        return &self.perspective_matrix;
    }
    pub fn get_infinite_projection_matrix(&mut self) -> &DMat4 {
        self.update_self();
        return &self.infinite_perspective;
    }
    pub fn computeCullingVolume(
        &mut self,
        position: &DVec3,
        direction: &DVec3,
        up: &DVec3,
    ) -> &CullingVolume {
        let planes = &mut self._culling_volume.planes;
        let position = *position;
        let direction = *direction;
        let up = *up;
        let t = self.top;
        let b = self.bottom;
        let r = self.right;
        let l = self.left;
        let n = self.near;
        let f = self.far;

        let right = direction.cross(up);

        let mut near_center = direction * n;
        near_center = position + near_center;

        let mut far_center = direction * f;
        far_center = position + far_center;

        //LEFT plane computation
        let mut normal = right.multiply_by_scalar(l);
        normal = near_center + normal;
        normal = normal - position;
        normal = normal.normalize();
        normal = normal.cross(up);
        normal = normal.normalize();

        let mut plane = &mut planes[0];
        plane.x = normal.x;
        plane.y = normal.y;
        plane.z = normal.z;
        plane.w = -normal.dot(position);

        //RIGHT plane computation
        normal = up.cross(near_center + right * r - position).normalize();

        plane = &mut planes[1];
        plane.x = normal.x;
        plane.y = normal.y;
        plane.z = normal.z;
        plane.w = -normal.dot(position);

        //Bottom plane computation
        normal = right.cross(near_center + up * b - position).normalize();

        plane = &mut planes[2];
        plane.x = normal.x;
        plane.y = normal.y;
        plane.z = normal.z;
        plane.w = -normal.dot(position);

        //Top plane computation
        normal = (near_center + up * t - position).cross(right).normalize();

        plane = &mut planes[3];
        plane.x = normal.x;
        plane.y = normal.y;
        plane.z = normal.z;
        plane.w = -normal.dot(position);

        //Near plane computation
        plane = &mut planes[4];
        plane.x = direction.x;
        plane.y = direction.y;
        plane.z = direction.z;
        plane.w = -direction.dot(near_center);

        //Far plane computation
        normal = direction.negate();

        plane = &mut planes[5];
        plane.x = normal.x;
        plane.y = normal.y;
        plane.z = normal.z;
        plane.w = -normal.dot(far_center);
        // self._culling_volume = CullingVolume::new(Some([]))
        return &self._culling_volume;
    }
}

#[cfg(test)]
mod tests {
    use crate::{equals_epsilon, Cartesian4, EPSILON10, EPSILON15, EPSILON6};

    use super::*;
    fn create_frustum() -> PerspectiveOffCenterFrustum {
        let mut frustum = PerspectiveOffCenterFrustum::new();
        frustum.right = 1.0;
        frustum.left = -frustum.right;
        frustum.top = 1.0;
        frustum.bottom = -frustum.top;
        frustum.near = 1.0;
        frustum.far = 2.0;
        return frustum;
    }
    fn get_planes(frustum: &mut PerspectiveOffCenterFrustum) -> &CullingVolume {
        return frustum.computeCullingVolume(&DVec3::ZERO, &DVec3::UNIT_Z.negate(), &DVec3::UNIT_Y);
    }

    #[test]
    fn get_left_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let left_plane = planes[0];
        let x = 1.0 / 2.0f64.sqrt();
        let expcted_result = DVec4::new(x, 0.0, -x, 0.0);
        assert!(left_plane.equals_epsilon(expcted_result, Some(EPSILON15), None));
    }
    #[test]
    fn get_right_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let left_plane = planes[1];
        let x = 1.0 / 2.0f64.sqrt();
        let expcted_result = DVec4::new(-x, 0.0, -x, 0.0);
        assert!(left_plane.equals_epsilon(expcted_result, Some(EPSILON15), None));
    }
    #[test]
    fn get_bottom_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let left_plane = planes[2];
        let x = 1.0 / 2.0f64.sqrt();
        let expcted_result = DVec4::new(0.0, x, -x, 0.0);
        assert!(left_plane.equals_epsilon(expcted_result, Some(EPSILON15), None));
    }
    #[test]
    fn get_top_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let left_plane = planes[3];
        let x = 1.0 / 2.0f64.sqrt();
        let expcted_result = DVec4::new(0.0, -x, -x, 0.0);
        assert!(left_plane.equals_epsilon(expcted_result, Some(EPSILON15), None));
    }
    #[test]
    fn get_near_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let left_plane = planes[4];
        let expcted_result = DVec4::new(0.0, 0.0, -1.0, -1.0);
        assert!(left_plane.equals_epsilon(expcted_result, Some(EPSILON15), None));
    }
    #[test]
    fn get_far_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let left_plane = planes[5];
        let expcted_result = DVec4::new(0.0, 0.0, 1.0, 2.0);
        assert!(left_plane.equals_epsilon(expcted_result, Some(EPSILON15), None));
    }
    #[test]
    fn get_perspective_project_matrix() {
        let mut frustum = create_frustum();
        let exp = DMat4::compute_perspective_off_center(
            frustum.left,
            frustum.right,
            frustum.bottom,
            frustum.top,
            frustum.near,
            frustum.far,
        );
        assert!(exp.abs_diff_eq(frustum.get_projection_matrix().clone(), EPSILON6))
    }
    #[test]
    fn get_infinite_perspective_matrix() {
        let mut frustum = create_frustum();
        let exp = DMat4::compute_infinite_perspective_off_center(
            frustum.left,
            frustum.right,
            frustum.bottom,
            frustum.top,
            frustum.near,
        );
        assert!(exp.abs_diff_eq(frustum.get_infinite_projection_matrix().clone(), EPSILON6))
    }
}
