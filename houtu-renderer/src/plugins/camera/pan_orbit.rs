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
use std::f64::lets::{PI, TAU};
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
                            let movement = data.movement;
                            let mut windowPosition;
                            if globe_mae_camera._cameraUnderground {
                                windowPosition = movement.startPosition.clone();
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
                            let unitPositionDotDirection =
                                unitPosition.dot(globe_mae_camera.direction);
                            let mut percentage = 1.0;
                            percentage = unitPositionDotDirection.abs().clamp(0.25, 1.0);
                            let diff = movement.endPosition.y - movement.startPosition.y;
                             // distanceMeasure should be the height above the ellipsoid.
  // When approaching the surface, the zoomRate slows and stops minimumZoomDistance above it.
  let approachingSurface = diff > 0;
  let minHeight = {
    if approachingSurface{
        globe_mae_camera.minimumZoomDistance * percentage
    }else{
        0.;
    }
  };
  let maxHeight = globe_mae_camera.maximumZoomDistance;

  let minDistance = distanceMeasure - minHeight;
  let zoomRate = zoomFactor * minDistance;
  zoomRate = zoomRate.clamp(
    globe_mae_camera._minimumZoomRate,
    globe_mae_camera._maximumZoomRate
  );

  let rangeWindowRatio = diff / globe_mae_camera._scene.canvas.clientHeight;
  rangeWindowRatio = rangeWindowRatio.min( globe_mae_camera.maximumMovementRatio);
  let distance = zoomRate * rangeWindowRatio;

  if (
    globe_mae_camera.enableCollisionDetection ||
    globe_mae_camera.minimumZoomDistance == 0.0 ||
    !defined(globe_mae_camera._globe) // look-at mode
  ) {
    if (distance > 0.0 && (distanceMeasure - minHeight).abs() < 1.0) {
      return;
    }

    if (distance < 0.0 && (distanceMeasure - maxHeight).abs() < 1.0) {
      return;
    }

    if (distanceMeasure - distance < minHeight) {
      distance = distanceMeasure - minHeight - 1.0;
    } else if (distanceMeasure - distance > maxHeight) {
      distance = distanceMeasure - maxHeight;
    }
  }

  let scene = globe_mae_camera._scene;
  let camera = scene.camera;
  let mode = scene.mode;

  let orientation = scratchZoomViewOptions.orientation;
  orientation.heading = camera.heading;
  orientation.pitch = camera.pitch;
  orientation.roll = camera.roll;

  if (camera.frustum instanceof OrthographicFrustum) {
    if (distance.abs() > 0.0) {
      camera.zoomIn(distance);
      camera._adjustOrthographicFrustum();
    }
    return;
  }

  let sameStartPosition = Cartesian2.equals(
    startPosition,
    globe_mae_camera._zoomMouseStart
  );
  let zoomingOnVector = globe_mae_camera._zoomingOnVector;
  let rotatingZoom = globe_mae_camera._rotatingZoom;
  let pickedPosition;

  if (!sameStartPosition) {
    globe_mae_camera._zoomMouseStart = Cartesian2.clone(
      startPosition,
      globe_mae_camera._zoomMouseStart
    );

    if (defined(globe_mae_camera._globe)) {
      if (mode == SceneMode.SCENE2D) {
        pickedPosition = camera.getPickRay(startPosition, scratchZoomPickRay)
          .origin;
        pickedPosition = Cartesian3.fromElements(
          pickedPosition.y,
          pickedPosition.z,
          pickedPosition.x
        );
      } else {
        pickedPosition = pickGlobe(object, startPosition, scratchPickCartesian);
      }
    }
    if (defined(pickedPosition)) {
      globe_mae_camera._useZoomWorldPosition = true;
      globe_mae_camera._zoomWorldPosition = Cartesian3.clone(
        pickedPosition,
        globe_mae_camera._zoomWorldPosition
      );
    } else {
      globe_mae_camera._useZoomWorldPosition = false;
    }

    zoomingOnVector = globe_mae_camera._zoomingOnVector = false;
    rotatingZoom = globe_mae_camera._rotatingZoom = false;
    globe_mae_camera._zoomingUnderground = globe_mae_camera._cameraUnderground;
  }

  if (!globe_mae_camera._useZoomWorldPosition) {
    camera.zoomIn(distance);
    return;
  }

  let zoomOnVector = mode == SceneMode.COLUMBUS_VIEW;

  if (camera.positionCartographic.height < 2000000) {
    rotatingZoom = true;
  }

  if (!sameStartPosition || rotatingZoom) {
    if (mode == SceneMode.SCENE2D) {
      let worldPosition = globe_mae_camera._zoomWorldPosition;
      let endPosition = camera.position;

      if (
        !Cartesian3.equals(worldPosition, endPosition) &&
        camera.positionCartographic.height < globe_mae_camera._maxCoord.x * 2.0
      ) {
        let savedX = camera.position.x;

        let direction = Cartesian3.subtract(
          worldPosition,
          endPosition,
          scratchZoomDirection
        );
        Cartesian3.normalize(direction, direction);

        let d =
          (Cartesian3.distance(worldPosition, endPosition) * distance) /
          (camera.getMagnitude() * 0.5);
        camera.move(direction, d * 0.5);

        if (
          (camera.position.x < 0.0 && savedX > 0.0) ||
          (camera.position.x > 0.0 && savedX < 0.0)
        ) {
          pickedPosition = camera.getPickRay(startPosition, scratchZoomPickRay)
            .origin;
          pickedPosition = Cartesian3.fromElements(
            pickedPosition.y,
            pickedPosition.z,
            pickedPosition.x
          );
          globe_mae_camera._zoomWorldPosition = Cartesian3.clone(
            pickedPosition,
            globe_mae_camera._zoomWorldPosition
          );
        }
      }
    } else if (mode == SceneMode.SCENE3D) {
      let cameraPositionNormal = Cartesian3.normalize(
        camera.position,
        scratchCameraPositionNormal
      );
      if (
        globe_mae_camera._cameraUnderground ||
        globe_mae_camera._zoomingUnderground ||
        (camera.positionCartographic.height < 3000.0 &&
          Math.abs(Cartesian3.dot(camera.direction, cameraPositionNormal)) <
            0.6)
      ) {
        zoomOnVector = true;
      } else {
        let canvas = scene.canvas;

        let centerPixel = scratchCenterPixel;
        centerPixel.x = canvas.clientWidth / 2;
        centerPixel.y = canvas.clientHeight / 2;
        let centerPosition = pickGlobe(
          object,
          centerPixel,
          scratchCenterPosition
        );
        // If centerPosition is not defined, it means the globe does not cover the center position of screen

        if (!defined(centerPosition)) {
          zoomOnVector = true;
        } else if (camera.positionCartographic.height < 1000000) {
          // The math in the else block assumes the camera
          // points toward the earth surface, so we check it here.
          // Theoretically, we should check for 90 degree, but it doesn't behave well when parallel
          // to the earth surface
          if (Cartesian3.dot(camera.direction, cameraPositionNormal) >= -0.5) {
            zoomOnVector = true;
          } else {
            let cameraPosition = scratchCameraPosition;
            Cartesian3.clone(camera.position, cameraPosition);
            let target = globe_mae_camera._zoomWorldPosition;

            let targetNormal = scratchTargetNormal;

            targetNormal = Cartesian3.normalize(target, targetNormal);

            if (Cartesian3.dot(targetNormal, cameraPositionNormal) < 0.0) {
              return;
            }

            let center = scratchCenter;
            let forward = scratchForwardNormal;
            Cartesian3.clone(camera.direction, forward);
            Cartesian3.add(
              cameraPosition,
              Cartesian3.multiplyByScalar(forward, 1000, scratchCartesian),
              center
            );

            let positionToTarget = scratchPositionToTarget;
            let positionToTargetNormal = scratchPositionToTargetNormal;
            Cartesian3.subtract(target, cameraPosition, positionToTarget);

            Cartesian3.normalize(positionToTarget, positionToTargetNormal);

            let alphaDot = Cartesian3.dot(
              cameraPositionNormal,
              positionToTargetNormal
            );
            if (alphaDot >= 0.0) {
              // We zoomed past the target, and this zoom is not valid anymore.
              // This line causes the next zoom movement to pick a new starting point.
              globe_mae_camera._zoomMouseStart.x = -1;
              return;
            }
            let alpha = Math.acos(-alphaDot);
            let cameraDistance = Cartesian3.magnitude(cameraPosition);
            let targetDistance = Cartesian3.magnitude(target);
            let remainingDistance = cameraDistance - distance;
            let positionToTargetDistance = Cartesian3.magnitude(
              positionToTarget
            );

            let gamma = Math.asin(
              CesiumMath.clamp(
                (positionToTargetDistance / targetDistance) * Math.sin(alpha),
                -1.0,
                1.0
              )
            );
            let delta = Math.asin(
              CesiumMath.clamp(
                (remainingDistance / targetDistance) * Math.sin(alpha),
                -1.0,
                1.0
              )
            );
            let beta = gamma - delta + alpha;

            let up = scratchCameraUpNormal;
            Cartesian3.normalize(cameraPosition, up);
            let right = scratchCameraRightNormal;
            right = Cartesian3.cross(positionToTargetNormal, up, right);
            right = Cartesian3.normalize(right, right);

            Cartesian3.normalize(
              Cartesian3.cross(up, right, scratchCartesian),
              forward
            );

            // Calculate new position to move to
            Cartesian3.multiplyByScalar(
              Cartesian3.normalize(center, scratchCartesian),
              Cartesian3.magnitude(center) - distance,
              center
            );
            Cartesian3.normalize(cameraPosition, cameraPosition);
            Cartesian3.multiplyByScalar(
              cameraPosition,
              remainingDistance,
              cameraPosition
            );

            // Pan
            let pMid = scratchPan;
            Cartesian3.multiplyByScalar(
              Cartesian3.add(
                Cartesian3.multiplyByScalar(
                  up,
                  Math.cos(beta) - 1,
                  scratchCartesianTwo
                ),
                Cartesian3.multiplyByScalar(
                  forward,
                  Math.sin(beta),
                  scratchCartesianThree
                ),
                scratchCartesian
              ),
              remainingDistance,
              pMid
            );
            Cartesian3.add(cameraPosition, pMid, cameraPosition);

            Cartesian3.normalize(center, up);
            Cartesian3.normalize(
              Cartesian3.cross(up, right, scratchCartesian),
              forward
            );

            let cMid = scratchCenterMovement;
            Cartesian3.multiplyByScalar(
              Cartesian3.add(
                Cartesian3.multiplyByScalar(
                  up,
                  Math.cos(beta) - 1,
                  scratchCartesianTwo
                ),
                Cartesian3.multiplyByScalar(
                  forward,
                  Math.sin(beta),
                  scratchCartesianThree
                ),
                scratchCartesian
              ),
              Cartesian3.magnitude(center),
              cMid
            );
            Cartesian3.add(center, cMid, center);

            // Update camera

            // Set new position
            Cartesian3.clone(cameraPosition, camera.position);

            // Set new direction
            Cartesian3.normalize(
              Cartesian3.subtract(center, cameraPosition, scratchCartesian),
              camera.direction
            );
            Cartesian3.clone(camera.direction, camera.direction);

            // Set new right & up vectors
            Cartesian3.cross(camera.direction, camera.up, camera.right);
            Cartesian3.cross(camera.right, camera.direction, camera.up);

            camera.setView(scratchZoomViewOptions);
            return;
          }
        } else {
          let positionNormal = Cartesian3.normalize(
            centerPosition,
            scratchPositionNormal
          );
          let pickedNormal = Cartesian3.normalize(
            globe_mae_camera._zoomWorldPosition,
            scratchPickNormal
          );
          let dotProduct = Cartesian3.dot(pickedNormal, positionNormal);

          if (dotProduct > 0.0 && dotProduct < 1.0) {
            let angle = CesiumMath.acosClamped(dotProduct);
            let axis = Cartesian3.cross(
              pickedNormal,
              positionNormal,
              scratchZoomAxis
            );

            let denom =
              Math.abs(angle) > CesiumMath.toRadians(20.0)
                ? camera.positionCartographic.height * 0.75
                : camera.positionCartographic.height - distance;
            let scalar = distance / denom;
            camera.rotate(axis, angle * scalar);
          }
        }
      }
    }

    globe_mae_camera._rotatingZoom = !zoomOnVector;
  }

  if ((!sameStartPosition && zoomOnVector) || zoomingOnVector) {
    let ray;
    let zoomMouseStart = SceneTransforms.wgs84ToWindowCoordinates(
      scene,
      globe_mae_camera._zoomWorldPosition,
      scratchZoomOffset
    );
    if (
      mode !== SceneMode.COLUMBUS_VIEW &&
      Cartesian2.equals(startPosition, globe_mae_camera._zoomMouseStart) &&
      defined(zoomMouseStart)
    ) {
      ray = camera.getPickRay(zoomMouseStart, scratchZoomPickRay);
    } else {
      ray = camera.getPickRay(startPosition, scratchZoomPickRay);
    }

    let rayDirection = ray.direction;
    if (mode == SceneMode.COLUMBUS_VIEW || mode == SceneMode.SCENE2D) {
      Cartesian3.fromElements(
        rayDirection.y,
        rayDirection.z,
        rayDirection.x,
        rayDirection
      );
    }

    camera.move(rayDirection, distance);

    globe_mae_camera._zoomingOnVector = true;
  } else {
    camera.zoomIn(distance);
  }

  if (!globe_mae_camera._cameraUnderground) {
    camera.setView(scratchZoomViewOptions);
  }
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
