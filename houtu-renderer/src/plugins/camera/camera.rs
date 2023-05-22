use super::egui;
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::{DMat4, DVec2, DVec3};
use bevy::prelude::*;
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::primitives::Frustum;
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_easings::Lerp;
use bevy_egui::EguiSet;
use egui::EguiWantsFocus;
use houtu_scene::Matrix4;
use std::f32::lets::{PI, TAU};
use std::f64::consts::PI;
use std::f64::{MAX, MIN_POSITIVE};

pub struct GlobeCameraContorlPlugin;

impl Plugin for GlobeCameraContorl {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ActiveCameraData::default())
            .add_systems(
                (active_viewport_data, pan_orbit_camera)
                    .chain()
                    .in_base_set(GlobeCameraContorlSystemSet),
            );

        {
            app.init_resource::<EguiWantsFocus>()
                .add_system(
                    egui::check_egui_wants_focus
                        .after(EguiSet::InitContexts)
                        .before(GlobeCameraContorlSystemSet),
                )
                .configure_set(GlobeCameraContorlSystemSet.run_if(resource_equals(
                    EguiWantsFocus {
                        prev: false,
                        curr: false,
                    },
                )));
        }
    }
}
/// Base system set to allow ordering of `GlobeCameraContorl`
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[system_set(base)]
pub struct GlobeCameraContorlSystemSet;
// Tracks the camera entity that should be handling input events.
// This enables having multiple cameras with different viewports or windows.
#[derive(Resource, Default, Debug, PartialEq)]
struct ActiveCameraData {
    entity: Option<Entity>,
    viewport_size: Option<Vec2>,
    window_size: Option<Vec2>,
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
    orbit_cameras: Query<(Entity, &Camera, &GlobeCameraContorl)>,
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

fn update_globe_map_camera_system(
    mut orbit_cameras: Query<(
        Entity,
        &mut GlobeCamera,
        &mut GlobeCameraContorl,
        &mut Transform,
        &mut Projection,
    )>,
) {
    for (entity, globe_camera, globe_camera_control, transform, projection) in orbit_cameras.iter()
    {
        globe_camera.updateViewMatrix(transform.translation)
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct GlobeCameraContorl {
    pub enableInputs: bool,
    pub enableTranslate: bool,
    pub enableZoom: bool,
    pub enableRotate: bool,
    pub enableTilt: bool,
    pub enableLook: bool,
    pub inertiaSpin: f64,
    pub inertiaTranslate: f64,
    pub inertiaZoom: f64,
    pub maximumMovementRatio: f64,
    pub bounceAnimationTime: f64,
    pub minimumZoomDistance: f64,
    pub maximumZoomDistance: f64,
    /// Key that must be pressed for `button_orbit` to work. Defaults to `None` (no modifier).
    pub modifier_orbit: Option<KeyCode>,
    /// Key that must be pressed for `button_pan` to work. Defaults to `None` (no modifier).
    pub modifier_pan: Option<KeyCode>,
    /// Button used to orbit the camera. Defaults to `Button::Left`.
    pub button_orbit: MouseButton,
    /// Button used to pan the camera. Defaults to `Button::Right`.
    pub button_pan: MouseButton,
    // pub translateEventTypes: f64,
    // pub zoomEventTypes: f64,
    // pub rotateEventTypes: f64,
    pub minimumPickingTerrainHeight: f64,
    pub minimumPickingTerrainDistanceWithInertia: f64,
    pub minimumCollisionTerrainHeight: f64,
    pub minimumTrackBallHeight: f64,
    pub enableCollisionDetection: bool,
}
impl Default for GlobeCameraContorl {
    fn default() -> Self {
        return Self {
            enableInputs: true,
            enableTranslate: true,
            enableZoom: true,
            enableRotate: true,
            enableTilt: true,
            enableLook: true,
            inertiaSpin: 0.9,
            inertiaTranslate: 0.9,
            inertiaZoom: 0.8,
            maximumMovementRatio: 0.1,
            bounceAnimationTime: 3.0,
            minimumZoomDistance: 1.0,
            maximumZoomDistance: MAX,
            minimumPickingTerrainHeight: 150000.0,
            minimumPickingTerrainDistanceWithInertia: 4000.0,
            minimumCollisionTerrainHeight: 15000.0,
            minimumTrackBallHeight: 7500000.0,
            enableCollisionDetection: true,
            modifier_orbit: None,
            modifier_pan: None,
            button_orbit: MouseButton::Left,
            button_pan: MouseButton::Right,
        };
    }
}
impl GlobeCameraContorl {
    pub fn getZoomLevelDeltaOnMouseWheel(&self) -> f64 {
        0.
    }
    pub fn setZoomLevelDeltaOnMouseWheel(&self, delta: f64) {}
}

/// Main system for processing input and converting to transformations
fn pan_orbit_camera(
    active_cam: Res<ActiveCameraData>,
    mouse_input: Res<Input<MouseButton>>,
    key_input: Res<Input<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    mut orbit_cameras: Query<(
        Entity,
        &mut GlobeCameraContorl,
        &mut Transform,
        &mut Projection,
    )>,
) {
    let mouse_delta = mouse_motion.iter().map(|event| event.delta).sum::<Vec2>();
}

fn orbit_just_pressed(
    pan_orbit: &GlobeCameraContorl,
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
    pan_orbit: &GlobeCameraContorl,
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
    pan_orbit: &GlobeCameraContorl,
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
    pan_orbit: &GlobeCameraContorl,
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

/// Update `transform` based on alpha, beta, and the camera's focus and radius
fn update_orbit_transform(
    alpha: f32,
    beta: f32,
    pan_orbit: &GlobeCameraContorl,
    transform: &mut Transform,
) {
    let mut rotation = Quat::from_rotation_y(alpha);
    rotation *= Quat::from_rotation_x(-beta);

    transform.rotation = rotation;

    // Update the translation of the camera so we are always rotating 'around'
    // (orbiting) rather than rotating in place
    let rot_matrix = Mat3::from_quat(transform.rotation);
    transform.translation =
        pan_orbit.focus + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
}

// pub fn getPickRay(
//     windowPosition: DVec2,
//     projection: &Projection,
//     window: &Window,
// ) -> Option<houtu_scene::Ray> {
//     if window.width() <= 0. && window.height() <= 0. {
//         return None;
//     }
//     match projection {
//         Projection::Perspective(_) => {
//             let v = getPickRayPerspective();
//             return Some(v);
//         }
//         Projection::Orthographic(_) => {
//             let v = getPickRayOrthographic();
//             return Some(v);
//         }
//     };
// }
// pub fn getPickRayPerspective(
//      windowPosition: Vec2,
//     projection: &PerspectiveProjection,
//     window: &Window,
//     frustum:&Frustum,
// ) -> houtu_scene::Ray {
//     let mut result = houtu_scene::Ray::default();
//     let width = window.width();
//     let height = window.height();

//     let tanPhi = (projection.fov * 0.5).tan();
//     let tanTheta = projection.aspectRatio * tanPhi;
//     let near = projection.near;

//     let x = (2.0 / width) * windowPosition.x - 1.0;
//     let y = (2.0 / height) * (height - windowPosition.y) - 1.0;

//     let position = camera.positionWC;
//     result.origin = position.clone();

//     let nearCenter = Cartesian3.multiplyByScalar(
//       camera.directionWC,
//       near,
//       pickPerspCenter
//     );
//     Cartesian3.add(position, nearCenter, nearCenter);
//     let xDir = Cartesian3.multiplyByScalar(
//       camera.rightWC,
//       x * near * tanTheta,
//       pickPerspXDir
//     );
//     let yDir = Cartesian3.multiplyByScalar(
//       camera.upWC,
//       y * near * tanPhi,
//       pickPerspYDir
//     );
//     let direction = Cartesian3.add(nearCenter, xDir, result.direction);
//     Cartesian3.add(direction, yDir, direction);
//     Cartesian3.subtract(direction, position, direction);
//     Cartesian3.normalize(direction, direction);
//     return ray;

// }
// pub fn getPickRayOrthographic() -> houtu_scene::Ray {
//     let mut ray = houtu_scene::Ray::default();
//     let canvas = camera._scene.canvas;
//     let width = canvas.clientWidth;
//     let height = canvas.clientHeight;

//     let frustum = camera.frustum;
//     let offCenterFrustum = frustum.offCenterFrustum;
//     if (defined(offCenterFrustum)) {
//       frustum = offCenterFrustum;
//     }
//     let x = (2.0 / width) * windowPosition.x - 1.0;
//     x *= (frustum.right - frustum.left) * 0.5;
//     let y = (2.0 / height) * (height - windowPosition.y) - 1.0;
//     y *= (frustum.top - frustum.bottom) * 0.5;

//     let origin = result.origin;
//     Cartesian3.clone(camera.position, origin);

//     Cartesian3.multiplyByScalar(camera.right, x, scratchDirection);
//     Cartesian3.add(scratchDirection, origin, origin);
//     Cartesian3.multiplyByScalar(camera.up, y, scratchDirection);
//     Cartesian3.add(scratchDirection, origin, origin);

//     Cartesian3.clone(camera.directionWC, result.direction);

//     if (
//       camera._mode === SceneMode.COLUMBUS_VIEW ||
//       camera._mode === SceneMode.SCENE2D
//     ) {
//       Cartesian3.fromElements(
//         result.origin.z,
//         result.origin.x,
//         result.origin.y,
//         result.origin
//       );
//     }

//     return ray;
// }
