use super::camera_new::{ActiveCameraData, CameraControl};
use super::{egui, GlobeMapCamera};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::{DMat4, DVec3};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_easings::Lerp;
use bevy_egui::EguiSet;
use egui::EguiWantsFocus;
use houtu_scene::Cartographic;
use std::f64::consts::{PI, TAU};
pub fn pan_orbit_camera(
    active_cam: Res<ActiveCameraData>,
    mouse_input: Res<Input<MouseButton>>,
    key_input: Res<Input<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut orbit_cameras: Query<(
        Entity,
        &mut CameraControl,
        &mut Transform,
        &mut Projection,
        &GlobeMapCamera,
    )>,
) {
    let mouse_delta = mouse_motion.iter().map(|event| event.delta).sum::<Vec2>();

    for (entity, mut pan_orbit, mut transform, mut projection, globe_mae_camera) in
        orbit_cameras.iter_mut()
    {}
}
