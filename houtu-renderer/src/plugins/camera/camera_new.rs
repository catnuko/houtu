use super::egui;
use super::pan_orbit::pan_orbit_camera;
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

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ActiveCameraData::default())
            .add_systems(
                (active_viewport_data, pan_orbit_camera)
                    .chain()
                    .in_base_set(PanOrbitCameraSystemSet),
            );

        {
            app.init_resource::<EguiWantsFocus>()
                .add_system(
                    egui::check_egui_wants_focus
                        .after(EguiSet::InitContexts)
                        .before(PanOrbitCameraSystemSet),
                )
                .configure_set(
                    PanOrbitCameraSystemSet.run_if(resource_equals(EguiWantsFocus {
                        prev: false,
                        curr: false,
                    })),
                );
        }
    }
}

/// Base system set to allow ordering of `CameraControl`
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[system_set(base)]
pub struct PanOrbitCameraSystemSet;

#[derive(Component)]
pub struct CameraControl {
    pub right: DVec3,
    pub _right: DVec3,
    pub _rightWC: DVec3,

    pub up: DVec3,
    pub _up: DVec3,
    pub _upWC: DVec3,

    pub direction: DVec3,
    pub _direction: DVec3,
    pub _directionWC: DVec3,

    pub _transform: DMat4,
    pub _invTransform: DMat4,
    pub _actualTransform: DMat4,
    pub _actualInvTransform: DMat4,
    pub _transformChanged: bool,

    pub position: DVec3,
    pub _position: DVec3,
    pub _positionWC: DVec3,
    pub _positionCartographic: Cartographic,
    pub _oldPositionWC: Option<DVec3>,

    pub positionWCDeltaMagnitude: f64,
    pub positionWCDeltaMagnitudeLastFrame: f64,
    pub timeSinceMoved: f64,
    pub _lastMovedTimestamp: f64,
    pub defaultLookAmount: f64,
    pub defaultRotateAmount: f64,
    pub defaultZoomAmount: f64,
    pub defaultMoveAmount: f64,
    pub maximumZoomFactor: f64,
    pub percentageChanged: f64,
    pub _viewMatrix: DMat4,
    pub _invViewMatrix: DMat4,

    /// Button used to orbit the camera. Defaults to `Button::Left`.
    pub button_orbit: MouseButton,
    /// Button used to pan the camera. Defaults to `Button::Right`.
    pub button_pan: MouseButton,
    /// Key that must be pressed for `button_orbit` to work. Defaults to `None` (no modifier).
    pub modifier_orbit: Option<KeyCode>,
    /// Key that must be pressed for `button_pan` to work. Defaults to `None` (no modifier).
    pub modifier_pan: Option<KeyCode>,
}

impl Default for CameraControl {
    fn default() -> Self {
        Self {
            positionWCDeltaMagnitude: 0.0,
            positionWCDeltaMagnitudeLastFrame: 0.0,
            timeSinceMoved: 0.0,
            _lastMovedTimestamp: 0.0,
            defaultMoveAmount: 100000.0,
            defaultLookAmount: PI / 60.0,
            defaultRotateAmount: PI / 3600.0,
            defaultZoomAmount: 100000.0,
            maximumZoomFactor: 1.5,
            percentageChanged: 0.5,
            _viewMatrix: DMat4::ZERO,
            _invViewMatrix: DMat4::ZERO,
            right: DVec3::ZERO,
            _right: DVec3::ZERO,
            _rightWC: DVec3::ZERO,

            up: DVec3::ZERO,
            _up: DVec3::ZERO,
            _upWC: DVec3::ZERO,

            direction: DVec3::ZERO,
            _direction: DVec3::ZERO,
            _directionWC: DVec3::ZERO,

            _transform: DMat4::IDENTITY,
            _invTransform: DMat4::IDENTITY,
            _actualTransform: DMat4::IDENTITY,
            _actualInvTransform: DMat4::IDENTITY,
            _transformChanged: false,

            position: DVec3::ZERO,
            _position: DVec3::ZERO,
            _positionWC: DVec3::ZERO,
            _positionCartographic: Cartographic::ZERO,
            _oldPositionWC: None,
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Right,
            modifier_orbit: None,
            modifier_pan: None,
        }
    }
}

impl CameraControl {}

// Tracks the camera entity that should be handling input events.
// This enables having multiple cameras with different viewports or windows.
#[derive(Resource, Default, Debug, PartialEq)]
pub struct ActiveCameraData {
    pub entity: Option<Entity>,
    pub viewport_size: Option<Vec2>,
    pub window_size: Option<Vec2>,
}

// Gathers data about the active viewport, i.e. the viewport the user is interacting with. This
// enables multiple viewports/windows.
fn active_viewport_data(
    mut active_cam: ResMut<ActiveCameraData>,
    mouse_input: Res<Input<MouseButton>>,
    key_input: Res<Input<KeyCode>>,
    scroll_events: EventReader<MouseWheel>,
    primary_windows: Query<&Window, With<PrimaryWindow>>,
    other_windows: Query<&Window, Without<PrimaryWindow>>,
    orbit_cameras: Query<(Entity, &Camera, &CameraControl)>,
) {
    let mut new_resource: Option<ActiveCameraData> = None;
    let mut max_cam_order = 0;

    for (entity, camera, pan_orbit) in orbit_cameras.iter() {
        let input_just_activated = orbit_just_pressed(pan_orbit, &mouse_input, &key_input)
            || pan_just_pressed(pan_orbit, &mouse_input, &key_input)
            || !scroll_events.is_empty();

        if input_just_activated {
            // First check if cursor is in the same window as this camera
            if let RenderTarget::Window(win_ref) = camera.target {
                let window = match win_ref {
                    WindowRef::Primary => primary_windows
                        .get_single()
                        .expect("Must exist, since the camera is referencing it"),
                    WindowRef::Entity(entity) => other_windows
                        .get(entity)
                        .expect("Must exist, since the camera is referencing it"),
                };
                if let Some(mut cursor_pos) = window.cursor_position() {
                    // Now check if cursor is within this camera's viewport
                    if let Some(vp_rect) = camera.logical_viewport_rect() {
                        // Window coordinates have Y starting at the bottom, so we need to reverse
                        // the y component before comparing with the viewport rect
                        cursor_pos.y = window.height() - cursor_pos.y;
                        let cursor_in_vp = cursor_pos.x > vp_rect.0.x
                            && cursor_pos.x < vp_rect.1.x
                            && cursor_pos.y > vp_rect.0.y
                            && cursor_pos.y < vp_rect.1.y;

                        // Only set if camera order is higher. This may overwrite a previous value
                        // in the case the viewport overlapping another viewport.
                        if cursor_in_vp && camera.order >= max_cam_order {
                            new_resource = Some(ActiveCameraData {
                                entity: Some(entity),
                                viewport_size: camera.logical_viewport_size(),
                                window_size: Some(Vec2::new(window.width(), window.height())),
                            });
                            max_cam_order = camera.order;
                        }
                    }
                }
            }
        }
    }

    if let Some(new_resource) = new_resource {
        active_cam.set_if_neq(new_resource);
    }
}

fn orbit_pressed(
    pan_orbit: &CameraControl,
    mouse_input: &Res<Input<MouseButton>>,
    key_input: &Res<Input<KeyCode>>,
) -> bool {
    let is_pressed = pan_orbit
        .modifier_orbit
        .map_or(true, |modifier| key_input.pressed(modifier))
        && mouse_input.pressed(pan_orbit.button_orbit);

    is_pressed
        && pan_orbit
            .modifier_pan
            .map_or(true, |modifier| !key_input.pressed(modifier))
}

fn orbit_just_pressed(
    pan_orbit: &CameraControl,
    mouse_input: &Res<Input<MouseButton>>,
    key_input: &Res<Input<KeyCode>>,
) -> bool {
    let just_pressed = pan_orbit
        .modifier_orbit
        .map_or(true, |modifier| key_input.pressed(modifier))
        && (mouse_input.just_pressed(pan_orbit.button_orbit));

    just_pressed
        && pan_orbit
            .modifier_pan
            .map_or(true, |modifier| !key_input.pressed(modifier))
}

fn orbit_just_released(
    pan_orbit: &CameraControl,
    mouse_input: &Res<Input<MouseButton>>,
    key_input: &Res<Input<KeyCode>>,
) -> bool {
    let just_released = pan_orbit
        .modifier_orbit
        .map_or(true, |modifier| key_input.pressed(modifier))
        && (mouse_input.just_released(pan_orbit.button_orbit));

    just_released
        && pan_orbit
            .modifier_pan
            .map_or(true, |modifier| !key_input.pressed(modifier))
}

fn pan_pressed(
    pan_orbit: &CameraControl,
    mouse_input: &Res<Input<MouseButton>>,
    key_input: &Res<Input<KeyCode>>,
) -> bool {
    let is_pressed = pan_orbit
        .modifier_pan
        .map_or(true, |modifier| key_input.pressed(modifier))
        && mouse_input.pressed(pan_orbit.button_pan);

    is_pressed
        && pan_orbit
            .modifier_orbit
            .map_or(true, |modifier| !key_input.pressed(modifier))
}

fn pan_just_pressed(
    pan_orbit: &CameraControl,
    mouse_input: &Res<Input<MouseButton>>,
    key_input: &Res<Input<KeyCode>>,
) -> bool {
    let just_pressed = pan_orbit
        .modifier_pan
        .map_or(true, |modifier| key_input.pressed(modifier))
        && (mouse_input.just_pressed(pan_orbit.button_pan));

    just_pressed
        && pan_orbit
            .modifier_orbit
            .map_or(true, |modifier| !key_input.pressed(modifier))
}

// /// Update `transform` based on alpha, beta, and the camera's focus and radius
// fn update_orbit_transform(
//     alpha: f32,
//     beta: f32,
//     pan_orbit: &CameraControl,
//     transform: &mut Transform,
// ) {
//     let mut rotation = Quat::from_rotation_y(alpha);
//     rotation *= Quat::from_rotation_x(-beta);

//     transform.rotation = rotation;

//     // Update the translation of the camera so we are always rotating 'around'
//     // (orbiting) rather than rotating in place
//     let rot_matrix = Mat3::from_quat(transform.rotation);
//     transform.translation =
//         pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
// }
