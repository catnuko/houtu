use bevy::math::{DVec3, DVec4};

use crate::{Cartesian3, CullingVolume};

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
    _cullingVolume: CullingVolume,
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
            _cullingVolume: CullingVolume::new(None),
        }
    }
    pub fn computeCullingVolume(
        &mut self,
        position: &DVec3,
        direction: &DVec3,
        up: &DVec3,
    ) -> &CullingVolume {
        let planes = self._cullingVolume.planes;
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

        let nearCenter = direction * n;

        let farCenter = direction * f;

        //Left plane computation
        let mut normal = (right * l + nearCenter - position)
            .normalize()
            .cross(up)
            .normalize();

        let mut plane = planes[0];
        plane.x = normal.x;
        plane.y = normal.y;
        plane.z = normal.z;
        plane.w = -normal.dot(position);

        //Right plane computation
        normal = up.cross((right * r + nearCenter - position)).normalize();

        plane = planes[1];
        plane.x = normal.x;
        plane.y = normal.y;
        plane.z = normal.z;
        plane.w = -normal.dot(position);

        //Bottom plane computation
        normal = right.cross(up * b + nearCenter - position).normalize();

        plane = planes[2];
        plane.x = normal.x;
        plane.y = normal.y;
        plane.z = normal.z;
        plane.w = -normal.dot(position);

        //Top plane computation
        normal = (up * t + nearCenter - position).cross(right).normalize();

        plane = planes[3];
        plane.x = normal.x;
        plane.y = normal.y;
        plane.z = normal.z;
        plane.w = -normal.dot(position);

        //Near plane computation
        plane = planes[4];
        plane.x = direction.x;
        plane.y = direction.y;
        plane.z = direction.z;
        plane.w = -direction.dot(nearCenter);

        //Far plane computation
        normal = direction.negate();

        plane = planes[5];
        plane.x = normal.x;
        plane.y = normal.y;
        plane.z = normal.z;
        plane.w = -normal.dot(farCenter);
        return &self._cullingVolume;
    }
}
