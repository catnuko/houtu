use crate::plugins::camera::camera_new::SetViewOrientation;

use super::camera_event_aggregator::{Aggregator, ControlEvent, EventStartPositionWrap};
use super::camera_new::GlobeCamera;
use super::{egui, GlobeCameraControl};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::{DMat4, DVec3};
use bevy::prelude::*;
use bevy::render::camera::{CameraProjection, RenderTarget};
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_easings::Lerp;
use bevy_egui::{EguiSet, WindowSize};
use egui::EguiWantsFocus;
use houtu_scene::{
    acos_clamped, Cartesian3, Cartographic, Ellipsoid, HeadingPitchRoll, SceneTransforms,
};
use std::f64::consts::{PI, TAU};
pub fn pan_orbit_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll_events: EventReader<MouseWheel>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    mut event_start_position_wrap: ResMut<EventStartPositionWrap>,
    mut aggregator: ResMut<Aggregator>,
    mut orbit_cameras: Query<(
        Entity,
        &mut Transform,
        &mut Projection,
        &mut GlobalTransform,
        &mut GlobeCamera,
        &mut GlobeCameraControl,
    )>,
    mut control_event_rader: EventReader<ControlEvent>,
) {
    let Ok(primary) = primary_query.get_single() else {
        return;
    };
    let window_size = Vec2 {
        x: primary.width(),
        y: primary.height(),
    };

    let mouse_delta = mouse_motion.iter().map(|event| event.delta).sum::<Vec2>();
    for event in control_event_rader.iter() {
        for (
            entity,
            mut transform,
            mut projection,
            global_transform,
            mut globe_camera,
            mut globe_camera_control,
        ) in &mut orbit_cameras
        {
            let projection = if let Projection::Perspective(v) = &*projection {
                v
            } else {
                return;
            };
            match event {
                ControlEvent::Zoom(data) => {
                    let startPosition =
                        aggregator.getStartMousePosition("WHEEL", &event_start_position_wrap);
                    let movement = &data.movement;
                    let mut windowPosition;
                    if globe_camera_control._cameraUnderground {
                        windowPosition = startPosition.clone();
                    } else {
                        windowPosition = Vec2::ZERO;
                        windowPosition.x = window_size.x / 2.0;
                        windowPosition.y = window_size.y / 2.0;
                    }
                    let ray = globe_camera.getPickRay(&windowPosition, &window_size);
                    let position = ray.origin;
                    let direction = ray.direction;
                    let height = Ellipsoid::WGS84
                        .cartesianToCartographic(&globe_camera.position)
                        .unwrap()
                        .height;
                    let normal = DVec3::UNIT_X;
                    let distance = height;
                    let unitPosition = globe_camera.position.normalize();
                    let unitPositionDotDirection = unitPosition.dot(globe_camera.direction);
                    let mut percentage = 1.0;
                    percentage = unitPositionDotDirection.abs().clamp(0.25, 1.0);
                    let diff = (movement.endPosition.y - movement.startPosition.y) as f64;
                    // distanceMeasure should be the height above the ellipsoid.
                    // When approaching the surface, the zoomRate slows and stops minimumZoomDistance above it.
                    let distanceMeasure = distance;
                    let zoomFactor = globe_camera_control._zoomFactor;
                    let approachingSurface = diff > 0.;
                    let minHeight = {
                        if approachingSurface {
                            globe_camera_control.minimumZoomDistance * percentage
                        } else {
                            0.
                        }
                    };
                    let maxHeight = globe_camera_control.maximumZoomDistance;

                    let minDistance = distanceMeasure - minHeight;
                    let mut zoomRate = zoomFactor * minDistance;
                    zoomRate = zoomRate.clamp(
                        globe_camera_control._minimumZoomRate,
                        globe_camera_control._maximumZoomRate,
                    );
                    let mut rangeWindowRatio = diff / window_size.y as f64;
                    rangeWindowRatio =
                        rangeWindowRatio.min(globe_camera_control.maximumMovementRatio);
                    let mut distance = zoomRate * rangeWindowRatio;

                    if globe_camera_control.enableCollisionDetection
                        || globe_camera_control.minimumZoomDistance == 0.0
                    // || !defined(globe_camera_control._globe)
                    // look-at mode
                    {
                        if (distance > 0.0 && (distanceMeasure - minHeight).abs() < 1.0) {
                            continue;
                        }

                        if (distance < 0.0 && (distanceMeasure - maxHeight).abs() < 1.0) {
                            continue;
                        }

                        if (distanceMeasure - distance < minHeight) {
                            distance = distanceMeasure - minHeight - 1.0;
                        } else if (distanceMeasure - distance > maxHeight) {
                            distance = distanceMeasure - maxHeight;
                        }
                    }

                    // let scene = globe_camera_control._scene;
                    // let camera = scene.camera;
                    // let mode = scene.mode;

                    let mut hpr = HeadingPitchRoll::default();
                    hpr.heading = globe_camera.hpr.heading;
                    hpr.pitch = globe_camera.hpr.pitch;
                    hpr.roll = globe_camera.hpr.roll;

                    let sameStartPosition = startPosition.eq(&globe_camera_control._zoomMouseStart);
                    let mut zoomingOnVector = globe_camera_control._zoomingOnVector;
                    let mut rotatingZoom = globe_camera_control._rotatingZoom;
                    let pickedPosition;

                    if (!sameStartPosition) {
                        pickedPosition = globe_camera.pickEllipsoid(&startPosition, &window_size);

                        globe_camera_control._zoomMouseStart = startPosition.clone();
                        if (pickedPosition.is_some()) {
                            globe_camera_control._useZoomWorldPosition = true;
                            globe_camera_control._zoomWorldPosition =
                                pickedPosition.unwrap().clone();
                        } else {
                            globe_camera_control._useZoomWorldPosition = false;
                        }

                        zoomingOnVector = false;
                        globe_camera_control._zoomingOnVector = false;
                        rotatingZoom = false;
                        globe_camera_control._rotatingZoom = false;
                        globe_camera_control._zoomingUnderground =
                            globe_camera_control._cameraUnderground;
                    }

                    if (!globe_camera_control._useZoomWorldPosition) {
                        globe_camera.zoom_in(Some(distance));
                        globe_camera.update_camera_matrix(&mut transform);
                        return;
                    }

                    let mut zoomOnVector = false;

                    if (globe_camera._positionCartographic.height < 2000000.) {
                        rotatingZoom = true;
                    }

                    if (!sameStartPosition || rotatingZoom) {
                        let cameraPositionNormal = globe_camera.position.normalize();
                        if (globe_camera_control._cameraUnderground
                            || globe_camera_control._zoomingUnderground
                            || (globe_camera._positionCartographic.height < 3000.0
                                && (globe_camera.direction.dot(cameraPositionNormal)).abs() < 0.6))
                        {
                            zoomOnVector = true;
                        } else {
                            let mut centerPixel = Vec2::ZERO;
                            centerPixel.x = window_size.x / 2.;
                            centerPixel.y = window_size.y / 2.;
                            //TODO: pickEllipsoid取代globe.pick，此刻还没加载地形和模型，所以暂时这么做
                            let centerPosition =
                                globe_camera.pickEllipsoid(&centerPixel, &window_size);
                            // If centerPosition is not defined, it means the globe does not cover the center position of screen

                            if (centerPosition.is_none()) {
                                zoomOnVector = true;
                            } else if (globe_camera._positionCartographic.height < 1000000.) {
                                // The math in the else block assumes the camera
                                // points toward the earth surface, so we check it here.
                                // Theoretically, we should check for 90 degree, but it doesn't behave well when parallel
                                // to the earth surface
                                if (globe_camera.direction.dot(cameraPositionNormal) >= -0.5) {
                                    zoomOnVector = true;
                                } else {
                                    let mut cameraPosition = globe_camera.position.clone();
                                    let target = globe_camera_control._zoomWorldPosition;

                                    let mut targetNormal = DVec3::ZERO;

                                    targetNormal = target.normalize();

                                    if (targetNormal.dot(cameraPositionNormal) < 0.0) {
                                        globe_camera.update_camera_matrix(&mut transform);
                                        return;
                                    }

                                    let mut center = DVec3::ZERO;
                                    let mut forward = DVec3::ZERO;
                                    forward = globe_camera.direction.clone();
                                    center = cameraPosition + forward.multiply_by_scalar(1000.);

                                    let mut positionToTarget = DVec3::ZERO;
                                    let mut positionToTargetNormal = DVec3::ZERO;
                                    positionToTarget = target.subtract(cameraPosition);

                                    positionToTargetNormal = positionToTarget.normalize();

                                    let alphaDot = cameraPositionNormal.dot(positionToTargetNormal);
                                    if (alphaDot >= 0.0) {
                                        // We zoomed past the target, and this zoom is not valid anymore.
                                        // This line causes the next zoom movement to pick a new starting point.
                                        globe_camera_control._zoomMouseStart.x = -1.0;
                                        globe_camera.update_camera_matrix(&mut transform);
                                        return;
                                    }
                                    let alpha = (-alphaDot).acos();
                                    let cameraDistance = cameraPosition.magnitude();
                                    let targetDistance = target.magnitude();
                                    let remainingDistance = cameraDistance - distance;
                                    let positionToTargetDistance = positionToTarget.magnitude();

                                    let gamma = ((positionToTargetDistance / targetDistance)
                                        * alpha.sin())
                                    .clamp(-1.0, 1.0)
                                    .asin();

                                    let delta = ((remainingDistance / targetDistance)
                                        * alpha.sin())
                                    .clamp(-1.0, 1.0)
                                    .asin();

                                    let beta = gamma - delta + alpha;

                                    let mut up = DVec3::ZERO;
                                    up = cameraPosition.normalize();
                                    let mut right = DVec3::ZERO;
                                    right = positionToTargetNormal.cross(up);
                                    right = right.normalize();

                                    forward = up.cross(right).normalize();

                                    // Calculate new position to move to
                                    center = center
                                        .normalize()
                                        .multiply_by_scalar(center.magnitude() - distance);
                                    cameraPosition = cameraPosition.normalize();
                                    cameraPosition.multiply_by_scalar(remainingDistance);

                                    // Pan
                                    let mut pMid = DVec3::ZERO;
                                    pMid = (up.multiply_by_scalar(beta.cos() - 1.)
                                        + forward.multiply_by_scalar(beta.sin()))
                                    .multiply_by_scalar(remainingDistance);
                                    cameraPosition = cameraPosition + pMid;

                                    up = center.normalize();
                                    forward = up.cross(right).normalize();

                                    let mut cMid = DVec3::ZERO;
                                    cMid = (up.multiply_by_scalar(beta.cos() - 1.)
                                        + forward.multiply_by_scalar(beta.sin()))
                                    .multiply_by_scalar(center.magnitude());
                                    center = center + cMid;

                                    // Update camera

                                    // Set new position
                                    globe_camera.position = cameraPosition;

                                    // Set new direction
                                    globe_camera.direction =
                                        center.subtract(cameraPosition).normalize();
                                    globe_camera.direction = globe_camera.direction.clone();
                                    // Set new right & up vectors
                                    globe_camera.right =
                                        globe_camera.direction.cross(globe_camera.up);
                                    globe_camera.up =
                                        globe_camera.right.cross(globe_camera.direction);

                                    globe_camera.set_view(
                                        None,
                                        Some(SetViewOrientation::HeadingPitchRoll(hpr)),
                                        None,
                                        None,
                                    );
                                    return;
                                }
                            } else {
                                let positionNormal = centerPosition.unwrap().normalize();
                                let pickedNormal =
                                    globe_camera_control._zoomWorldPosition.normalize();
                                let dotProduct = pickedNormal.dot(positionNormal);

                                if (dotProduct > 0.0 && dotProduct < 1.0) {
                                    let angle = acos_clamped(dotProduct);
                                    let axis = pickedNormal.cross(positionNormal);

                                    let denom = {
                                        if angle.abs() > (20.0 as f64).to_radians() {
                                            globe_camera._positionCartographic.height * 0.75
                                        } else {
                                            globe_camera._positionCartographic.height - distance
                                        }
                                    };

                                    let scalar = distance / denom;
                                    globe_camera.rotate(axis, Some(angle * scalar));
                                }
                            }
                        }

                        globe_camera_control._rotatingZoom = !zoomOnVector;
                    }

                    if ((!sameStartPosition && zoomOnVector) || zoomingOnVector) {
                        let ray;
                        let zoomMouseStart = SceneTransforms::wgs84ToWindowCoordinates(
                            &globe_camera_control._zoomWorldPosition,
                            &window_size,
                            &to_mat4_64(&global_transform.compute_matrix()),
                            &to_mat4_64(&projection.get_projection_matrix()),
                        );
                        if (startPosition.eq(&globe_camera_control._zoomMouseStart)
                            && zoomMouseStart.is_some())
                        {
                            let v = zoomMouseStart.unwrap();
                            ray = globe_camera.getPickRay(&v, &window_size)
                        } else {
                            ray = globe_camera.getPickRay(&startPosition, &window_size);
                        }

                        let rayDirection = ray.direction;

                        globe_camera.move_direction(&rayDirection, distance);

                        globe_camera_control._zoomingOnVector = true;
                    } else {
                        globe_camera.zoom_in(Some(distance));
                    }

                    if (!globe_camera_control._cameraUnderground) {
                        globe_camera.set_view(
                            None,
                            Some(SetViewOrientation::HeadingPitchRoll(hpr)),
                            None,
                            None,
                        );
                    }
                    globe_camera.update_camera_matrix(&mut transform);
                    println!("controlevent zoom {:?}", data);
                }

                ControlEvent::Spin(data) => {
                    println!("controlevent spin {:?}", data);
                }

                ControlEvent::Tilt(data) => {
                    println!("controlevent tilt {:?}", data);
                }
            }
        }
    }
}
pub fn to_mat4_64(mat4: &Mat4) -> DMat4 {
    let mut matrix: [f32; 16] = [
        0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    ];
    mat4.write_cols_to_slice(&mut matrix);
    let mut new_matrix: [f64; 16] = [0.; 16];
    matrix
        .iter()
        .enumerate()
        .for_each(|(i, x)| new_matrix[i] = x.clone() as f64);
    return DMat4::from_cols_array(&new_matrix);
}
pub fn to_mat4_32(mat4: &DMat4) -> Mat4 {
    let mut matrix: [f64; 16] = [
        0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0., 0.,
    ];
    mat4.write_cols_to_slice(&mut matrix);
    let mut new_matrix: [f32; 16] = [0.; 16];
    matrix
        .iter()
        .enumerate()
        .for_each(|(i, x)| new_matrix[i] = x.clone() as f32);
    return Mat4::from_cols_array(&new_matrix);
}
// pub fn getZoomDistanceUnderground(
//     ellipsoid: &Ellipsoid,
//     ray: houtu_scene::Ray,
//     camera: &GlobeCameraControl,
// ) -> f64 {
//     let origin = ray.origin;
//     let direction = ray.direction;
//     let distanceFromSurface = getDistanceFromSurface(camera);

//     // Weight zoom distance based on how strongly the pick ray is pointing inward.
//     // Geocentric normal is accurate enough for these purposes
//     let surfaceNormal = origin.normalize();
//     let mut strength = (surfaceNormal.dot(direction)).abs();
//     strength = strength.max(0.5) * 2.0;
//     return distanceFromSurface * strength;
// }
// pub fn getDistanceFromSurface(camera: &GlobeCameraControl) -> f64 {
//     let mut height = 0.0;
//     let cartographic =
//         Ellipsoid::WGS84.cartesianToCartographic(&globe_camera_control.position_cartesian);
//     if let Some(v) = cartographic {
//         height = v.height;
//     }
//     let globeHeight = 0.;
//     let distanceFromSurface = (globeHeight - height).abs();
//     return distanceFromSurface;
// }
// pub fn pickGlobe(camera:&GlobeCameraControl, mousePosition:Vec2) ->DVec3{
//     let scene = controller._scene;
//     let globe = controller._globe;
//     let camera = scene.camera;

//     if (!defined(globe)) {
//       return undefined;
//     }

//     let cullBackFaces = !globe_camera_control._cameraUnderground;

//     let depthIntersection;
//     if (scene.pickPositionSupported) {
//       depthIntersection = scene.pickPositionWorldCoordinates(
//         mousePosition,
//         scratchDepthIntersection
//       );
//     }

//     let ray = globe_camera_control.getPickRay(mousePosition, pickGlobeScratchRay);
//     let rayIntersection = globe.pickWorldCoordinates(
//       ray,
//       scene,
//       cullBackFaces,
//       scratchRayIntersection
//     );

//     let pickDistance = defined(depthIntersection)
//       ? Cartesian3.distance(depthIntersection, globe_camera_control.positionWC)
//       : Number.POSITIVE_INFINITY;
//     let rayDistance = defined(rayIntersection)
//       ? Cartesian3.distance(rayIntersection, globe_camera_control.positionWC)
//       : Number.POSITIVE_INFINITY;

//     if (pickDistance < rayDistance) {
//       return Cartesian3.clone(depthIntersection, result);
//     }

//     return Cartesian3.clone(rayIntersection, result);
//   }
