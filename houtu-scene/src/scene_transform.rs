use bevy::{
    math::{DMat4, DVec2, DVec3, DVec4},
    prelude::Vec2,
};

use crate::{
    lerp, BoundingRectangle, Cartesian3, Cartesian4, Ellipsoid, GeographicProjection, Matrix4,
    Projection,
};

pub struct SceneTransforms;

impl SceneTransforms {
    pub fn wgs84ToWindowCoordinates(
        position: &DVec3,
        window_size: &Vec2,
        view_matrix: &DMat4,
        projection_matrix: &DMat4,
    ) -> Option<Vec2> {
        return SceneTransforms::wgs84WithEyeOffsetToWindowCoordinates(
            position,
            &DVec3::ZERO,
            window_size,
            view_matrix,
            projection_matrix,
        );
    }
    pub fn wgs84WithEyeOffsetToWindowCoordinates(
        position: &DVec3,
        eyeOffset: &DVec3,
        window_size: &Vec2,
        view_matrix: &DMat4,
        projection_matrix: &DMat4,
    ) -> Option<Vec2> {
        // let frameState = scene.frameState;
        let actualPosition = SceneTransforms::computeActualWgs84Position(position);
        if actualPosition.is_none() {
            return None;
        }
        let actualPosition = actualPosition.unwrap();

        // Assuming viewport takes up the entire canvas...
        let mut viewport = BoundingRectangle::new();
        viewport.x = 0.;
        viewport.y = 0.;
        viewport.width = window_size.x as f64;
        viewport.height = window_size.y as f64;

        // let camera = scene.camera;
        let cameraCentered = false;
        let mut result = Vec2::ZERO;
        if (cameraCentered) {
            // View-projection matrix to transform from world coordinates to clip coordinates
            let positionCC =
                worldToClip(&actualPosition, eyeOffset, view_matrix, projection_matrix);
            if (positionCC.z < 0.) {
                return None;
            }

            result = SceneTransforms::clipToGLWindowCoordinates(&viewport, &positionCC);
        }

        result.y = window_size.y - result.y;
        return Some(result);
    }
    pub fn computeActualWgs84Position(position: &DVec3) -> Option<DVec3> {
        let result = position.clone();

        let cartographic = Ellipsoid::WGS84.cartesianToCartographic(position);
        if let Some(cartographic) = cartographic {
            let projectedPosition = GeographicProjection::WGS84.project(&cartographic);

            return Some(DVec3::from_elements(
                lerp(projectedPosition.z, position.x, 1.),
                lerp(projectedPosition.x, position.y, 1.),
                lerp(projectedPosition.y, position.z, 1.),
            ));
        } else {
            return None;
        }
    }
    pub fn clipToGLWindowCoordinates(viewport: &BoundingRectangle, position: &DVec4) -> Vec2 {
        // Perspective divide to transform from clip coordinates to normalized device coordinates
        let new_position = DVec3::from_elements(position.x, position.y, position.z);
        let positionNDC = new_position.divide_by_scalar(position.w);

        // Viewport transform to transform from clip coordinates to window coordinates
        let mut viewportTransform = viewport.computeViewportTransformation(0.0, 1.0);
        let positionWC = viewportTransform.multiply_by_point(&positionNDC);

        return Vec2::new(positionWC.x as f32, positionWC.y as f32);
    }
}

pub fn worldToClip(
    position: &DVec3,
    eyeOffset: &DVec3,
    view_matrix: &DMat4,
    projection_matrix: &DMat4,
) -> DVec4 {
    let mut positionEC = view_matrix.multiply_by_vector(&DVec4::from_elements(
        position.x, position.y, position.z, 1.,
    ));
    let mut new_position_ec = DVec3::from_cartesian4(positionEC);
    let zEyeOffset = eyeOffset.multiply_components(&new_position_ec.normalize());
    positionEC.x += eyeOffset.x + zEyeOffset.x;
    positionEC.y += eyeOffset.y + zEyeOffset.y;
    positionEC.z += zEyeOffset.z;

    return projection_matrix.multiply_by_vector(&positionEC);
}
