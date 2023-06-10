use crate::plugins::camera::camera_new::SetViewOrientation;

use super::camera_event_aggregator::{
    Aggregator, ControlEvent, EventStartPositionWrap, MovementState,
};
use super::camera_new::GlobeCamera;
use super::{egui, GlobeCameraControl};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::{DMat4, DVec2, DVec3};
use bevy::prelude::*;
use bevy::render::camera::{CameraProjection, RenderTarget};
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_easings::Lerp;
use bevy_egui::{EguiSet, WindowSize};
use egui::EguiWantsFocus;
use houtu_scene::{
    acos_clamped, to_mat4_64, Cartesian3, Cartographic, Ellipsoid, HeadingPitchRoll,
    IntersectionTests, Plane, Ray, SceneTransforms, EPSILON14, EPSILON2, EPSILON3, EPSILON4,
};
use std::f64::consts::{PI, TAU};
use std::ops::Neg;
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
    let window_size = DVec2 {
        x: primary.width() as f64,
        y: primary.height() as f64,
    };

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
                        windowPosition = DVec2::ZERO;
                        windowPosition.x = window_size.x / 2.0;
                        windowPosition.y = window_size.y / 2.0;
                    }

                    // let ray = globe_camera.getPickRay(&windowPosition, &window_size);

                    let height = Ellipsoid::WGS84
                        .cartesianToCartographic(&globe_camera.position)
                        .unwrap()
                        .height;

                    let distance = height;
                    let unitPosition = globe_camera.position.normalize();
                    let unitPositionDotDirection = unitPosition.dot(globe_camera.direction);

                    //以下是handleZoom函数
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
                    hpr.heading = globe_camera.get_heading();
                    hpr.pitch = globe_camera.get_pitch();
                    hpr.roll = globe_camera.get_roll();

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

                    if (globe_camera.get_position_cartographic().height < 2000000.) {
                        rotatingZoom = true;
                    }

                    if (!sameStartPosition || rotatingZoom) {
                        let cameraPositionNormal = globe_camera.position.normalize();
                        if (globe_camera_control._cameraUnderground
                            || globe_camera_control._zoomingUnderground
                            || (globe_camera.get_position_cartographic().height < 3000.0
                                && (globe_camera.direction.dot(cameraPositionNormal)).abs() < 0.6))
                        {
                            zoomOnVector = true;
                        } else {
                            let mut centerPixel = DVec2::ZERO;
                            centerPixel.x = window_size.x / 2.;
                            centerPixel.y = window_size.y / 2.;
                            //TODO: pickEllipsoid取代globe.pick，此刻还没加载地形和模型，所以暂时这么做
                            let centerPosition =
                                globe_camera.pickEllipsoid(&centerPixel, &window_size);
                            // If centerPosition is not defined, it means the globe does not cover the center position of screen

                            if (centerPosition.is_none()) {
                                zoomOnVector = true;
                            } else if (globe_camera.get_position_cartographic().height < 1000000.) {
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
                                    positionToTarget = target - cameraPosition;

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
                                            globe_camera.get_position_cartographic().height * 0.75
                                        } else {
                                            globe_camera.get_position_cartographic().height
                                                - distance
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
                    let startPosition =
                        aggregator.getStartMousePosition("LEFT_DRAG", &event_start_position_wrap);
                    let mut movement = data.movement.clone();
                    spin3D(
                        &mut globe_camera_control,
                        &mut globe_camera,
                        &startPosition,
                        &mut movement,
                        &window_size,
                    );
                    globe_camera.update_camera_matrix(&mut transform);
                }

                ControlEvent::Tilt(data) => {
                    println!("controlevent tilt {:?}", data);
                    let startPosition =
                        aggregator.getStartMousePosition("MIDDLE_DRAG", &event_start_position_wrap);
                    let mut movement = data.movement.clone();
                    tilt3D(
                        &mut globe_camera_control,
                        &mut globe_camera,
                        &startPosition,
                        &mut movement,
                        &window_size,
                    );
                    globe_camera.update_camera_matrix(&mut transform);
                }
            }
        }
    }
}

fn tilt3DOnEllipsoid(
    controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    startPosition: &DVec2,
    movement: &MovementState,
    window_size: &DVec2,
) {
    let ellipsoid = Ellipsoid::WGS84;
    let minHeight = controller.minimumZoomDistance * 0.25;
    let height = ellipsoid
        .cartesianToCartographic(&camera.get_position_wc())
        .unwrap()
        .height;
    if (height - minHeight - 1.0 < EPSILON3
        && movement.endPosition.y - movement.startPosition.y < 0.)
    {
        return;
    }

    let mut windowPosition = DVec2::ZERO;
    windowPosition.x = window_size[0] / 2.;
    windowPosition.y = window_size[1] / 2.;
    let ray = camera.getPickRay(&windowPosition, window_size);

    let center;
    let intersection = IntersectionTests::rayEllipsoid(&ray, Some(&ellipsoid));
    if (intersection.is_some()) {
        let intersection = intersection.unwrap();
        center = Ray::getPoint(&ray, intersection.start);
    } else if (height > controller._minimumTrackBallHeight) {
        let grazingAltitudeLocation =
            IntersectionTests::grazingAltitudeLocation(&ray, Some(&ellipsoid));
        if grazingAltitudeLocation.is_none() {
            return;
        }
        let grazingAltitudeLocation = grazingAltitudeLocation.unwrap();
        let mut grazingAltitudeCart = ellipsoid
            .cartesianToCartographic(&grazingAltitudeLocation)
            .unwrap();
        grazingAltitudeCart.height = 0.0;
        center = ellipsoid.cartographicToCartesian(&grazingAltitudeCart);
    } else {
        controller._looking = true;
        let up = ellipsoid.geodeticSurfaceNormal(&camera.position);
        look3D(controller, camera, startPosition, movement, up, window_size);
        controller._tiltCenterMousePosition = startPosition.clone();
        return;
    }
}
fn tilt3D(
    controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    startPosition: &DVec2,
    movement: &MovementState,
    window_size: &DVec2,
) {
    if (!(camera.get_transform() == DMat4::IDENTITY)) {
        return;
    }

    // if movement.angleAndHeight.is_some() {
    //     movement = movement.angleAndHeight;
    // }

    if (!startPosition.eq(&controller._tiltCenterMousePosition)) {
        controller._tiltOnEllipsoid = false;
        controller._looking = false;
    }

    if (controller._looking) {
        let up = Ellipsoid::WGS84.geodeticSurfaceNormal(&camera.position);
        look3D(controller, camera, startPosition, movement, up, window_size);
        return;
    }
    let mut cartographic = Ellipsoid::WGS84
        .cartesianToCartographic(&camera.position)
        .unwrap();

    if (controller._tiltOnEllipsoid
        || cartographic.height > controller._minimumCollisionTerrainHeight)
    {
        controller._tiltOnEllipsoid = true;
        tilt3DOnEllipsoid(controller, camera, startPosition, movement, window_size);
    } else {
        // tilt3DOnTerrain(controller, startPosition, movement);
        panic!("暂时没有地形")
    }
}

fn spin3D(
    controller: &mut GlobeCameraControl,

    camera: &mut GlobeCamera,
    startPosition: &DVec2,
    movement: &mut MovementState,
    window_size: &DVec2,
) {
    let cameraUnderground = controller._cameraUnderground;
    let mut ellipsoid = Ellipsoid::WGS84;

    if (!camera.get_transform().eq(&DMat4::IDENTITY)) {
        rotate3D(
            controller,
            camera,
            startPosition,
            movement,
            window_size,
            None,
            None,
            None,
        );
        return;
    }

    let mut magnitude;
    let mut radii;

    let up = ellipsoid.geodeticSurfaceNormal(&camera.position);

    if (startPosition.eq(&controller._rotateMousePosition)) {
        if (controller._looking) {
            look3D(controller, camera, startPosition, movement, up, window_size);
        } else if (controller._rotating) {
            rotate3D(
                controller,
                camera,
                startPosition,
                movement,
                window_size,
                None,
                None,
                None,
            );
        } else if (controller._strafing) {
            continueStrafing(controller, camera, movement, window_size);
        } else {
            if (camera.position.magnitude() < controller._rotateStartPosition.length()) {
                // Pan action is no longer valid if camera moves below the pan ellipsoid
                return;
            }
            magnitude = controller._rotateStartPosition.length();
            radii = DVec3::ZERO;
            radii.x = magnitude;
            radii.y = magnitude;
            radii.z = magnitude;
            ellipsoid = Ellipsoid::from_vec3(radii);
            pan3D(controller, camera, startPosition, movement, window_size);
        }
        return;
    }
    controller._looking = false;
    controller._rotating = false;
    controller._strafing = false;
    let height = ellipsoid
        .cartesianToCartographic(&camera.get_position_wc())
        .unwrap()
        .height;
    let globe = false;
    let spin3DPick = camera.pickEllipsoid(&movement.startPosition, window_size);
    if (spin3DPick.is_some()) {
        pan3D(controller, camera, startPosition, movement, window_size);
        controller._rotateStartPosition = spin3DPick.unwrap();
    } else if height > controller._minimumTrackBallHeight {
        controller._rotating = true;
        rotate3D(
            controller,
            camera,
            startPosition,
            movement,
            window_size,
            None,
            None,
            None,
        );
    } else {
        controller._looking = true;
        look3D(
            controller,
            camera,
            startPosition,
            movement,
            None,
            window_size,
        );
    }
    controller._rotateMousePosition = startPosition.clone();
}
fn rotate3D(
    controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    startPosition: &DVec2,
    movement: &MovementState,
    window_size: &DVec2,
    constrainedAxis: Option<DVec3>,
    rotateOnlyVertical: Option<bool>,
    rotateOnlyHorizontal: Option<bool>,
) {
    let rotateOnlyVertical = rotateOnlyVertical.unwrap_or(false);
    let rotateOnlyHorizontal = rotateOnlyHorizontal.unwrap_or(false);

    let oldAxis = camera.constrainedAxis;
    if (constrainedAxis.is_some()) {
        camera.constrainedAxis = constrainedAxis;
    }

    let rho = camera.position.magnitude();
    let mut rotateRate = controller._rotateFactor * (rho - controller._rotateRateRangeAdjustment);

    if (rotateRate > controller._maximumRotateRate) {
        rotateRate = controller._maximumRotateRate;
    }

    if (rotateRate < controller._minimumRotateRate) {
        rotateRate = controller._minimumRotateRate;
    }

    let mut phiWindowRatio =
        ((movement.startPosition.x - movement.endPosition.x) / window_size.x) as f64;
    let mut thetaWindowRatio =
        ((movement.startPosition.y - movement.endPosition.y) / window_size.y) as f64;
    phiWindowRatio = phiWindowRatio.min(controller.maximumMovementRatio);
    thetaWindowRatio = thetaWindowRatio.min(controller.maximumMovementRatio);

    let deltaPhi = rotateRate * phiWindowRatio * PI * 2.0;
    let deltaTheta = rotateRate * thetaWindowRatio * PI;

    if (!rotateOnlyVertical) {
        camera.rotate_right(Some(deltaPhi));
    }

    if (!rotateOnlyHorizontal) {
        camera.rotate_up(Some(deltaTheta));
    }

    camera.constrainedAxis = oldAxis;
}

fn pan3D(
    controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    startPosition: &DVec2,
    movement: &MovementState,
    window_size: &DVec2,
) {
    let startMousePosition = movement.startPosition.clone();
    let endMousePosition = movement.endPosition.clone();

    let p0 = camera.pickEllipsoid(&startMousePosition, window_size);
    let p1 = camera.pickEllipsoid(&endMousePosition, window_size);

    if (p0.is_none() || p1.is_none()) {
        controller._rotating = true;
        rotate3D(
            controller,
            camera,
            startPosition,
            movement,
            window_size,
            None,
            None,
            None,
        );
        return;
    }
    let mut p0 = p0.unwrap();
    let mut p1 = p1.unwrap();

    p0 = camera.worldToCameraCoordinates(&p0);
    p1 = camera.worldToCameraCoordinates(&p1);

    if camera.constrainedAxis.is_none() {
        p0 = p0.normalize();
        p1 = p1.normalize();
        let dot = p0.dot(p1);
        let axis = p0.cross(p1);

        if (dot < 1.0 && !axis.equals_epsilon(DVec3::ZERO, Some(EPSILON14), None)) {
            // dot is in [0, 1]
            let angle = dot.acos();
            camera.rotate(axis, Some(angle));
        }
    } else {
        let basis0 = camera.constrainedAxis.unwrap();
        let mut basis1 = basis0.most_orthogonal_axis();
        basis1 = basis1.cross(basis0);
        basis1 = basis1.normalize();
        let basis2 = basis0.cross(basis1);

        let startRho = p0.magnitude();
        let startDot = basis0.dot(p0);
        let startTheta = (startDot / startRho).acos();
        let mut startRej = basis0.multiply_by_scalar(startDot);
        startRej = p0.subtract(startRej).normalize();

        let endRho = p1.magnitude();
        let endDot = basis0.dot(p1);
        let endTheta = (endDot / endRho).acos();
        let mut endRej = basis0.multiply_by_scalar(endDot);
        endRej = p1.subtract(endRej).normalize();

        let mut startPhi = (startRej.dot(basis1)).acos();
        if (startRej.dot(basis2) < 0.) {
            startPhi = TAU - startPhi;
        }

        let mut endPhi = (endRej.dot(basis1)).acos();
        if (endRej.dot(basis2) < 0.) {
            endPhi = TAU - endPhi;
        }

        let deltaPhi = startPhi - endPhi;

        let east;
        if (basis0.equals_epsilon(camera.position, Some(EPSILON2), None)) {
            east = camera.right;
        } else {
            east = basis0.cross(camera.position);
        }

        let planeNormal = basis0.cross(east);
        let side0 = planeNormal.dot(p0.subtract(basis0));
        let side1 = planeNormal.dot(p1.subtract(basis0));

        let deltaTheta;
        if (side0 > 0. && side1 > 0.) {
            deltaTheta = endTheta - startTheta;
        } else if (side0 > 0. && side1 <= 0.) {
            if (camera.position.dot(basis0) > 0.) {
                deltaTheta = -startTheta - endTheta;
            } else {
                deltaTheta = startTheta + endTheta;
            }
        } else {
            deltaTheta = startTheta - endTheta;
        }

        camera.rotate_right(Some(deltaPhi));
        camera.rotate_up(Some(deltaTheta));
    }
}

fn look3D(
    controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    startPosition: &DVec2,
    movement: &MovementState,
    rotationAxis: Option<DVec3>,
    window_size: &DVec2,
) {
    let mut startPos = DVec2::ZERO;
    startPos.x = movement.startPosition.x as f64;
    startPos.y = 0.0;
    let mut endPos = DVec2::ZERO;
    endPos.x = movement.endPosition.x as f64;
    endPos.y = 0.0;

    let mut startRay = camera.getPickRay(&startPos, window_size);
    let mut endRay = camera.getPickRay(&endPos, window_size);
    let mut angle = 0.0;
    let mut start;
    let mut end;
    start = startRay.direction;
    end = endRay.direction;

    let mut dot = start.dot(end);
    if (dot < 1.0) {
        // dot is in [0, 1]
        angle = dot.acos();
    }
    angle = if movement.startPosition.x > movement.endPosition.x {
        -angle
    } else {
        angle
    };

    let horizontalRotationAxis = controller._horizontalRotationAxis;
    if (rotationAxis.is_some()) {
        camera.look(&rotationAxis.unwrap(), Some(-angle));
    } else if (horizontalRotationAxis.is_some()) {
        camera.look(&horizontalRotationAxis.unwrap(), Some(-angle));
    } else {
        camera.look_left(Some(angle));
    }

    startPos.x = 0.0;
    startPos.y = movement.startPosition.y;
    endPos.x = 0.0;
    endPos.y = movement.endPosition.y;

    startRay = camera.getPickRay(&startPos, window_size);
    endRay = camera.getPickRay(&endPos, window_size);
    angle = 0.0;

    start = startRay.direction;
    end = endRay.direction;

    dot = start.dot(end);
    if (dot < 1.0) {
        // dot is in [0, 1]
        angle = dot.acos();
    }
    angle = if movement.startPosition.y > movement.endPosition.y {
        -angle
    } else {
        angle
    };

    let rotationAxis = rotationAxis.unwrap_or(horizontalRotationAxis.unwrap_or(DVec3::ZERO));
    if (rotationAxis != DVec3::ZERO) {
        let direction = camera.direction;
        let negativeRotationAxis = rotationAxis.neg();
        let northParallel = direction.equals_epsilon(rotationAxis, Some(EPSILON2), None);
        let southParallel = direction.equals_epsilon(negativeRotationAxis, Some(EPSILON2), None);
        if (!northParallel && !southParallel) {
            dot = direction.dot(rotationAxis);
            let mut angleToAxis = acos_clamped(dot);
            if (angle > 0. && angle > angleToAxis) {
                angle = angleToAxis - EPSILON4;
            }

            dot = direction.dot(negativeRotationAxis);
            angleToAxis = acos_clamped(dot);
            if (angle < 0. && -angle > angleToAxis) {
                angle = -angleToAxis + EPSILON4;
            }

            let tangent = rotationAxis.cross(direction);
            camera.look(&tangent, Some(angle));
        } else if ((northParallel && angle < 0.) || (southParallel && angle > 0.)) {
            camera.look(&camera.right.clone(), Some(-angle));
        }
    } else {
        camera.look_up(Some(angle));
    }
}

fn continueStrafing(
    controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    movement: &mut MovementState,
    window_size: &DVec2,
) {
    // Update the end position continually based on the inertial delta
    let originalEndPosition = movement.endPosition;
    let inertialDelta = movement.endPosition - movement.startPosition;
    let mut endPosition = controller._strafeEndMousePosition;
    endPosition = endPosition + inertialDelta;
    movement.endPosition = endPosition;
    strafe(
        controller,
        camera,
        movement,
        &controller._strafeStartPosition.clone(),
        window_size,
    );
    movement.endPosition = originalEndPosition;
}

fn strafe(
    controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    movement: &MovementState,
    strafeStartPosition: &DVec3,
    window_size: &DVec2,
) {
    let ray = camera.getPickRay(&movement.endPosition, window_size);

    let mut direction = camera.direction.clone();
    let plane = Plane::fromPointNormal(&strafeStartPosition, &direction);
    let intersection = IntersectionTests::rayPlane(&ray, &plane);
    if (intersection.is_none()) {
        return;
    }
    let intersection = intersection.unwrap();
    direction = *strafeStartPosition - intersection;

    camera.position = camera.position + direction;
}
