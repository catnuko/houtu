use bevy::{
    math::{DMat3, DQuat},
    prelude::*,
};
use houtu_scene::{Cartesian3, FrustumGeometry, Matrix3};

use crate::camera::GlobeCamera;

use super::ui_state::UiState;
pub fn debug_system(
    mut state: ResMut<UiState>,
    mut commands: Commands,
    mut camera_query: Query<(Entity, &mut GlobeCamera)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (_entity, mut globe_camera) in &mut camera_query {
        if state.show_frustum_planes {
            if state.frustum_planes_entity.is_none() {
                let p = globe_camera.get_position_wc().clone();
                let d = globe_camera.get_direction_wc().clone();
                let u = globe_camera.get_up_wc().clone();
                let mut r = globe_camera.get_right_wc().clone();
                r = r.negate();
                let mut rotation = DMat3::ZERO;
                rotation.set_column(0, &r);
                rotation.set_column(1, &u);
                rotation.set_column(2, &d);
                let orientation = DQuat::from_mat3(&rotation);

                let geometry = FrustumGeometry::new(globe_camera.frustum.clone(), p, orientation);
                let entity = commands.spawn_empty().id();
                state.frustum_planes_entity = Some(entity);
                commands.entity(entity).insert(PbrBundle {
                    mesh: meshes.add(geometry.into()),
                    material: materials.add(Color::RED.with_a(0.5).into()),
                    ..Default::default()
                });
            }
        } else {
            if let Some(entity) = state.frustum_planes_entity {
                commands.entity(entity).despawn();
                state.frustum_planes_entity = None;
            }
        }
    }
}
