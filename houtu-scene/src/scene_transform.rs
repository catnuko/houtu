use bevy::math::{DMat4, DVec2, DVec3, DVec4};

use crate::{
    lerp, BoundingRectangle, Cartesian3, Cartesian4, Ellipsoid, GeographicProjection, Matrix4,
    Projection,
};

pub struct SceneTransforms;

impl SceneTransforms {
    pub fn wgs84_to_window_coordinates(
        position: &DVec3,
        window_size: &DVec2,
        view_matrix: &DMat4,
        projection_matrix: &DMat4,
    ) -> Option<DVec2> {
        return SceneTransforms::wgs84_with_eye_offset_to_window_coordinates(
            position,
            &DVec3::ZERO,
            window_size,
            view_matrix,
            projection_matrix,
        );
    }
    pub fn wgs84_with_eye_offset_to_window_coordinates(
        position: &DVec3,
        eye_offset: &DVec3,
        window_size: &DVec2,
        view_matrix: &DMat4,
        projection_matrix: &DMat4,
    ) -> Option<DVec2> {
        // let frameState = scene.frameState;
        let actual_position = SceneTransforms::compute_actual_wgs84_position(position);
        if actual_position.is_none() {
            return None;
        }
        let actual_position = actual_position.unwrap();

        // Assuming viewport takes up the entire canvas...
        let mut viewport = BoundingRectangle::new();
        viewport.x = 0.;
        viewport.y = 0.;
        viewport.width = window_size.x as f64;
        viewport.height = window_size.y as f64;

        // let camera = scene.camera;
        let camera_centered = false;
        let mut result = DVec2::ZERO;
        if camera_centered {
            // View-projection matrix to transform from world coordinates to clip coordinates
            let position_cc =
                world_to_clip(&actual_position, eye_offset, view_matrix, projection_matrix);
            if position_cc.z < 0. {
                return None;
            }

            result = SceneTransforms::clip_to_gl_window_coordinates(&viewport, &position_cc);
        }

        result.y = window_size.y - result.y;
        return Some(result);
    }
    pub fn compute_actual_wgs84_position(position: &DVec3) -> Option<DVec3> {
        let result = position.clone();

        let cartographic = Ellipsoid::WGS84.cartesian_to_cartographic(position);
        if let Some(cartographic) = cartographic {
            let projected_position = GeographicProjection::WGS84.project(&cartographic);

            return Some(DVec3::from_elements(
                lerp(projected_position.z, position.x, 1.),
                lerp(projected_position.x, position.y, 1.),
                lerp(projected_position.y, position.z, 1.),
            ));
        } else {
            return None;
        }
    }
    pub fn clip_to_gl_window_coordinates(viewport: &BoundingRectangle, position: &DVec4) -> DVec2 {
        // Perspective divide to transform from clip coordinates to normalized device coordinates
        let new_position = DVec3::from_elements(position.x, position.y, position.z);
        let position_ndc = new_position.divide_by_scalar(position.w);

        // Viewport transform to transform from clip coordinates to window coordinates
        let mut viewport_transform = viewport.computeViewportTransformation(0.0, 1.0);
        let position_wc = viewport_transform.multiply_by_point(&position_ndc);

        return DVec2::new(position_wc.x, position_wc.y);
    }
}

pub fn world_to_clip(
    position: &DVec3,
    eye_offset: &DVec3,
    view_matrix: &DMat4,
    projection_matrix: &DMat4,
) -> DVec4 {
    let mut position_ec = view_matrix.multiply_by_vector(&DVec4::from_elements(
        position.x, position.y, position.z, 1.,
    ));
    let mut new_position_ec = DVec3::from_cartesian4(position_ec);
    let z_eye_offset = eye_offset.multiply_components(&new_position_ec.normalize());
    position_ec.x += eye_offset.x + z_eye_offset.x;
    position_ec.y += eye_offset.y + z_eye_offset.y;
    position_ec.z += z_eye_offset.z;

    return projection_matrix.multiply_by_vector(&position_ec);
}
