use super::camera_event_aggregator::ControlEvent;
use super::camera_new::{ActiveCameraData, CameraControl};
use super::{egui, getPickRay, GlobeMapCamera};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::{DMat4, DVec3};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_easings::Lerp;
use bevy_egui::EguiSet;
use egui::EguiWantsFocus;
use houtu_scene::{Cartesian3, Cartographic, Ellipsoid};
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
    mut control_event_rader: EventReader<ControlEvent>,
) {
    if let Some(window_size) = active_cam.viewport_size {
        let mouse_delta = mouse_motion.iter().map(|event| event.delta).sum::<Vec2>();
        for event in control_event_rader.iter() {
            for (entity, mut pan_orbit, mut transform, mut projection, globe_mae_camera) in
                orbit_cameras.iter_mut()
            {
                if let (Projection::Perspective(p), Some(camera_position_cartographic)) =
                    (*projection, globe_mae_camera.position_cartographic)
                {
                    match event {
                        ControlEvent::Zoom(data) => {
                            let mut windowPosition;
                            if globe_mae_camera._cameraUnderground {
                                windowPosition = data.movement.startPosition.clone();
                            } else {
                                windowPosition = Vec2::ZERO;
                                windowPosition.x = window_size.x / 2.0;
                                windowPosition.y = window_size.y / 2.0;
                            }
                            let ray =
                                getPickRay(windowPosition, &window_size, &p, globe_mae_camera);
                            let position = ray.origin;
                            let direction = ray.direction;
                            let height = camera_position_cartographic.height;
                            let normal = DVec3::UNIT_X;
                            let distance = height;
                            let unitPosition = globe_mae_camera.position_cartesian.normalize();
                        }

                        ControlEvent::Spin(data) => {}

                        ControlEvent::Tilt(data) => {}
                    }
                }
            }
        }
    }
}
pub fn getZoomDistanceUnderground(
    ellipsoid: &Ellipsoid,
    ray: houtu_scene::Ray,
    camera: &GlobeMapCamera,
) -> f64 {
    let origin = ray.origin;
    let direction = ray.direction;
    let distanceFromSurface = getDistanceFromSurface(ellipsoid, camera);

    // Weight zoom distance based on how strongly the pick ray is pointing inward.
    // Geocentric normal is accurate enough for these purposes
    let surfaceNormal = origin.normalize();
    let mut strength = (surfaceNormal.dot(direction)).abs();
    strength = strength.max(0.5) * 2.0;
    return distanceFromSurface * strength;
}
pub fn getDistanceFromSurface(ellipsoid: &Ellipsoid, camera: &GlobeMapCamera) -> f64 {
    // let ellipsoid = controller._ellipsoid;
    // let scene = controller._scene;
    // let camera = scene.camera;
    // let mode = scene.mode;

    let mut height = 0.0;
    let cartographic = ellipsoid.cartesianToCartographic(camera.position_cartesian);
    if let Some(v) = cartographic {
        height = v.height;
    }
    let globeHeight = 0.;
    let distanceFromSurface = (globeHeight - height).abs();
    return distanceFromSurface;
}
// pub fn pickGlobe(camera:&GlobeMapCamera, mousePosition:Vec2) ->DVec3{
//     let scene = controller._scene;
//     let globe = controller._globe;
//     let camera = scene.camera;

//     if (!defined(globe)) {
//       return undefined;
//     }

//     let cullBackFaces = !camera._cameraUnderground;

//     let depthIntersection;
//     if (scene.pickPositionSupported) {
//       depthIntersection = scene.pickPositionWorldCoordinates(
//         mousePosition,
//         scratchDepthIntersection
//       );
//     }

//     let ray = camera.getPickRay(mousePosition, pickGlobeScratchRay);
//     let rayIntersection = globe.pickWorldCoordinates(
//       ray,
//       scene,
//       cullBackFaces,
//       scratchRayIntersection
//     );

//     let pickDistance = defined(depthIntersection)
//       ? Cartesian3.distance(depthIntersection, camera.positionWC)
//       : Number.POSITIVE_INFINITY;
//     let rayDistance = defined(rayIntersection)
//       ? Cartesian3.distance(rayIntersection, camera.positionWC)
//       : Number.POSITIVE_INFINITY;

//     if (pickDistance < rayDistance) {
//       return Cartesian3.clone(depthIntersection, result);
//     }

//     return Cartesian3.clone(rayIntersection, result);
//   }
