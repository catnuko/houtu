use bevy::math::{DMat4, DVec3};

use crate::{CullingVolume, PerspectiveOffCenterFrustum};

#[derive(Clone)]
pub struct PerspectiveFrustum {
    pub fov: f64,
    pub near: f64,
    pub far: f64,
    pub xOffset: f64,
    pub yOffset: f64,
    pub aspect_ratio: f64,
    _fov: f64,
    _fovy: f64,
    _near: f64,
    _far: f64,
    _xOffset: f64,
    _yOffset: f64,
    _aspect_ratio: f64,
    _sseDenominator: f64,
    _offCenterFrustum: PerspectiveOffCenterFrustum,
}
// 0.660105980317941
impl Default for PerspectiveFrustum {
    fn default() -> Self {
        let mut me = Self::new((60.0 as f64).to_radians(), 1.0, 1.0, 500000000.0, 0.0, 0.0);
        me.update_self();
        return me;
    }
}
impl PerspectiveFrustum {
    pub fn new(
        fov: f64,
        aspect_ratio: f64,
        near: f64,
        far: f64,
        xOffset: f64,
        yOffset: f64,
    ) -> Self {
        return Self {
            fov: fov,
            near: near,
            far: far,
            xOffset: xOffset,
            yOffset: yOffset,
            aspect_ratio: aspect_ratio,
            _fov: -1.0,
            _fovy: -1.0,
            _near: near,
            _far: far,
            _xOffset: xOffset,
            _yOffset: yOffset,
            _aspect_ratio: -1.0,
            _sseDenominator: -1.0,
            _offCenterFrustum: PerspectiveOffCenterFrustum::new(),
        };
    }
    pub fn get_fovy(&mut self) -> f64 {
        self.update_self();
        return self._fovy;
    }
    pub fn get_projection_matrix(&mut self) -> &DMat4 {
        self.update_self();
        return self._offCenterFrustum.get_projection_matrix();
    }
    pub fn get_infinite_projection_matrix(&mut self) -> &DMat4 {
        self.update_self();
        return self._offCenterFrustum.get_infinite_projection_matrix();
    }
    pub fn get_sse_denominator(&mut self) -> f64 {
        self.update_self();
        return self._sseDenominator;
    }

    pub fn get_off_center_frustum(&mut self) -> &PerspectiveOffCenterFrustum {
        self.update_self();
        return &self._offCenterFrustum;
    }
    pub fn computeCullingVolume(
        &mut self,
        position: &DVec3,
        direction: &DVec3,
        up: &DVec3,
    ) -> &CullingVolume {
        self.update_self();
        return self
            ._offCenterFrustum
            .computeCullingVolume(position, direction, up);
    }
    pub fn update_self(&mut self) {
        if self.fov != self._fov
            || self.aspect_ratio != self._aspect_ratio
            || self.near != self._near
            || self.far != self._far
            || self.xOffset != self._xOffset
            || self.yOffset != self._yOffset
        {
            let f = &mut self._offCenterFrustum;
            self._aspect_ratio = self.aspect_ratio;
            self._fov = self.fov;
            self._fovy = if self.aspect_ratio <= 1.0 {
                self.fov
            } else {
                ((self.fov * 0.5).tan() / self.aspect_ratio).atan() * 2.0
            };
            self._near = self.near;
            self._far = self.far;
            self._sseDenominator = 2.0 * (0.5 * self._fovy).tan();
            self._xOffset = self.xOffset;
            self._yOffset = self.yOffset;

            f.top = self.near * (0.5 * self._fovy).tan();
            f.bottom = -f.top;
            f.right = self.aspect_ratio * f.top;
            f.left = -f.right;
            f.near = self.near;
            f.far = self.far;

            f.right += self.xOffset;
            f.left += self.xOffset;
            f.top += self.yOffset;
            f.bottom += self.yOffset;
        }
    }
}
#[cfg(test)]
mod tests {
    use std::f64::consts::PI;

    use bevy::math::DVec4;

    use super::*;
    use crate::{equals_epsilon, Cartesian3, Cartesian4, EPSILON10, EPSILON14, EPSILON15};
    fn create_frustum() -> PerspectiveFrustum {
        let mut frustum = PerspectiveFrustum::default();
        frustum.near = 1.0;
        frustum.far = 2.0;
        frustum.aspect_ratio = 1.0;
        frustum.fov = PI / 3.0;
        return frustum;
    }
    fn get_planes(frustum: &mut PerspectiveFrustum) -> &CullingVolume {
        return frustum.computeCullingVolume(&DVec3::ZERO, &DVec3::UNIT_Z.negate(), &DVec3::UNIT_Y);
    }
    #[test]
    fn default_construct() {
        let mut f = PerspectiveFrustum::default();
        assert!(f.fov == (60.0 as f64).to_radians());
        assert!(f.aspect_ratio == 1.0);
        assert!(f.near == 1.0);
        assert!(f.far == 500000000.0);
        assert!(f.xOffset == 0.0);
        assert!(f.yOffset == 0.0);
    }
    #[test]
    fn get_left_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let plane = planes[0];
        let exp = DVec4::new(3.0f64.sqrt() / 2.0, 0.0, -0.5, 0.0);
        assert!(plane.equals_epsilon(exp, Some(EPSILON14), None));
    }
    #[test]
    fn get_right_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let plane = planes[1];
        let exp = DVec4::new(-3.0f64.sqrt() / 2.0, 0.0, -0.5, 0.0);
        assert!(plane.equals_epsilon(exp, Some(EPSILON14), None));
    }
    #[test]
    fn get_bottom_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let plane = planes[2];
        let exp = DVec4::new(0.0, 3.0f64.sqrt() / 2.0, -0.5, 0.0);
        assert!(plane.equals_epsilon(exp, Some(EPSILON14), None));
    }
    #[test]
    fn get_top_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let plane = planes[3];
        let exp = DVec4::new(0.0, -3.0f64.sqrt() / 2.0, -0.5, 0.0);
        assert!(plane.equals_epsilon(exp, Some(EPSILON14), None));
    }
    #[test]
    fn get_near_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let plane = planes[4];
        let exp = DVec4::new(0.0, 0.0, -1.0, -1.0);
        assert!(plane == exp);
    }
    #[test]
    fn get_far_plane() {
        let mut frustum = create_frustum();
        let culling_volume = get_planes(&mut frustum);
        let planes = culling_volume.get_planes();
        let plane = planes[5];
        let exp = DVec4::new(0.0, 0.0, 1.0, 2.0);
        assert!(plane == exp);
    }
}
