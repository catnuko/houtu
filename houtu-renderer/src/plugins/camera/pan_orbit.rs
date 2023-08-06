use crate::plugins::camera::globe_camra::SetViewOrientation;

use super::camera_event_aggregator::{
    Aggregator, ControlEvent, EventStartPositionWrap, MovementState,
};
use super::globe_camra::GlobeCamera;
use super::GlobeCameraControl;

use bevy::math::{DMat4, DVec2, DVec3};
use bevy::prelude::*;
use bevy::render::camera::CameraProjection;
use bevy::window::PrimaryWindow;

use houtu_scene::{
    acos_clamped, to_mat4_64, Cartesian3, Ellipsoid, HeadingPitchRoll, IntersectionTests, Plane,
    Ray, SceneTransforms, Transforms, EPSILON14, EPSILON2, EPSILON3, EPSILON4,
};
use std::f64::consts::{PI, TAU};
use std::ops::Neg;
pub fn pan_orbit_camera(
    primary_query: Query<&Window, With<PrimaryWindow>>,
    event_start_position_wrap: ResMut<EventStartPositionWrap>,
    aggregator: ResMut<Aggregator>,
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
            _entity,
            mut transform,
            projection,
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
            globe_camera_control.update(&mut globe_camera);
            match event {
                ControlEvent::Zoom(data) => {
                    //zoom3D函数的内容
                    let start_position =
                        aggregator.get_start_mouse_position("WHEEL", &event_start_position_wrap);
                    let movement = &data.movement;
                    let mut window_position;
                    if globe_camera_control._camera_underground {
                        window_position = start_position.clone();
                    } else {
                        window_position = DVec2::ZERO;
                        window_position.x = window_size.x / 2.0;
                        window_position.y = window_size.y / 2.0;
                    }

                    // let ray = globe_camera.getPickRay(&window_position, &window_size);

                    let height = Ellipsoid::WGS84
                        .cartesianToCartographic(&globe_camera.position)
                        .unwrap()
                        .height;

                    let distance = height;
                    let unit_position = globe_camera.position.normalize();
                    let unit_position_dot_direction = unit_position.dot(globe_camera.direction);

                    //以下是handleZoom函数
                    let mut percentage = 1.0;
                    percentage = unit_position_dot_direction.abs().clamp(0.25, 1.0);
                    let diff = (movement.end_position.y - movement.start_position.y) as f64;
                    // distance_measure should be the height above the ellipsoid.
                    // When approaching the surface, the zoom_rate slows and stops minimum_zoom_distance above it.
                    let distance_measure = distance;
                    let zoom_factor = globe_camera_control._zoom_factor;
                    let approaching_surface = diff > 0.;
                    let min_height = {
                        if approaching_surface {
                            globe_camera_control.minimum_zoom_distance * percentage
                        } else {
                            0.
                        }
                    };
                    let max_height = globe_camera_control.maximum_zoom_distance;

                    let min_distance = distance_measure - min_height;
                    let mut zoom_rate = zoom_factor * min_distance;
                    zoom_rate = zoom_rate.clamp(
                        globe_camera_control._minimum_zoom_rate,
                        globe_camera_control._maximum_zoom_rate,
                    );
                    let mut range_window_ratio = diff / window_size.y as f64;
                    range_window_ratio =
                        range_window_ratio.min(globe_camera_control.maximum_movement_ratio);
                    let mut distance = zoom_rate * range_window_ratio;

                    if globe_camera_control.enable_collision_detection
                        || globe_camera_control.minimum_zoom_distance == 0.0
                    // || !defined(globe_camera_control._globe)
                    // look-at mode
                    {
                        if distance > 0.0 && (distance_measure - min_height).abs() < 1.0 {
                            continue;
                        }

                        if distance < 0.0 && (distance_measure - max_height).abs() < 1.0 {
                            continue;
                        }

                        if distance_measure - distance < min_height {
                            distance = distance_measure - min_height - 1.0;
                        } else if distance_measure - distance > max_height {
                            distance = distance_measure - max_height;
                        }
                    }

                    // let scene = globe_camera_control._scene;
                    // let camera = scene.camera;
                    // let mode = scene.mode;

                    let mut hpr = HeadingPitchRoll::default();
                    hpr.heading = globe_camera.get_heading();
                    hpr.pitch = globe_camera.get_pitch();
                    hpr.roll = globe_camera.get_roll();

                    let same_start_position =
                        start_position.eq(&globe_camera_control._zoom_mouse_start);
                    let mut zooming_on_vector = globe_camera_control._zooming_on_vector;
                    let mut rotating_zoom = globe_camera_control._rotating_zoom;
                    let picked_position;

                    if !same_start_position {
                        picked_position =
                            globe_camera.pick_ellipsoid(&start_position, &window_size);

                        globe_camera_control._zoom_mouse_start = start_position.clone();
                        if picked_position.is_some() {
                            globe_camera_control._use_zoom_world_position = true;
                            globe_camera_control._zoom_world_position =
                                picked_position.unwrap().clone();
                        } else {
                            globe_camera_control._use_zoom_world_position = false;
                        }

                        zooming_on_vector = false;
                        globe_camera_control._zooming_on_vector = false;
                        rotating_zoom = false;
                        globe_camera_control._rotating_zoom = false;
                        globe_camera_control._zooming_underground =
                            globe_camera_control._camera_underground;
                    }
                    //用startPosition在球面上拾取不到坐标时,放大一些距离
                    if !globe_camera_control._use_zoom_world_position {
                        globe_camera.zoom_in(Some(distance));
                        globe_camera.update_camera_matrix(&mut transform);
                        return;
                    }
                    //以下是拾取到坐标是时执行以下代码

                    let mut zoom_on_vector = false;

                    //相机高度小于两百万米时，开启rotatingZoom
                    if globe_camera.get_position_cartographic().height < 2000000. {
                        rotating_zoom = true;
                    }

                    if !same_start_position || rotating_zoom {
                        let camera_position_normal = globe_camera.position.normalize();
                        if globe_camera_control._camera_underground
                            || globe_camera_control._zooming_underground
                            || (globe_camera.get_position_cartographic().height < 3000.0
                                && (globe_camera.direction.dot(camera_position_normal)).abs() < 0.6)
                        {
                            zoom_on_vector = true;
                        } else {
                            let mut center_pixel = DVec2::ZERO;
                            center_pixel.x = window_size.x / 2.;
                            center_pixel.y = window_size.y / 2.;
                            //TODO: pickEllipsoid取代globe.pick，此刻还没加载地形和模型，所以暂时这么做
                            let center_position =
                                globe_camera.pick_ellipsoid(&center_pixel, &window_size);
                            // If center_position is not defined, it means the globe does not cover the center position of screen
                            // 如果centerPosition没定义，意味着屏幕的中心点处没有地球，开启zoomOnVector
                            if center_position.is_none() {
                                zoom_on_vector = true;
                            } else if globe_camera.get_position_cartographic().height < 1000000. {
                                // The math in the else block assumes the camera
                                // points toward the earth surface, so we check it here.
                                // Theoretically, we should check for 90 degree, but it doesn't behave well when parallel
                                // to the earth surface
                                // 相机方向向量和相机位置向量的夹角在0-120°之间，开启zoomOnVector
                                if globe_camera.direction.dot(camera_position_normal) >= -0.5 {
                                    zoom_on_vector = true;
                                } else {
                                    let mut camera_position = globe_camera.position.clone();
                                    let target = globe_camera_control._zoom_world_position;

                                    let mut target_normal = DVec3::ZERO;

                                    target_normal = target.normalize();

                                    if target_normal.dot(camera_position_normal) < 0.0 {
                                        globe_camera.update_camera_matrix(&mut transform);
                                        return;
                                    }

                                    let mut center = DVec3::ZERO;
                                    let mut forward = DVec3::ZERO;
                                    forward = globe_camera.direction.clone();
                                    center = camera_position + forward.multiply_by_scalar(1000.);

                                    let mut position_to_target = DVec3::ZERO;
                                    let mut position_to_target_normal = DVec3::ZERO;
                                    position_to_target = target - camera_position;

                                    position_to_target_normal = position_to_target.normalize();

                                    let alpha_dot =
                                        camera_position_normal.dot(position_to_target_normal);
                                    if alpha_dot >= 0.0 {
                                        // We zoomed past the target, and this zoom is not valid anymore.
                                        // This line causes the next zoom movement to pick a new starting point.
                                        globe_camera_control._zoom_mouse_start.x = -1.0;
                                        globe_camera.update_camera_matrix(&mut transform);
                                        return;
                                    }
                                    let alpha = (-alpha_dot).acos();
                                    let camera_distance = camera_position.magnitude();
                                    let target_distance = target.magnitude();
                                    let remaining_distance = camera_distance - distance;
                                    let position_to_target_distance =
                                        position_to_target.magnitude();

                                    let gamma = ((position_to_target_distance / target_distance)
                                        * alpha.sin())
                                    .clamp(-1.0, 1.0)
                                    .asin();
                                    // 已推断出alpha和gamma角，找不到delta角在哪，如果有明白的人，请指点，下面给出研究成果，帮助理解。
                                    // https://www.geogebra.org/m/qxn5dvhk
                                    // 如果能找到delta就能找到beta，从而推断出pMid和cMid的含义
                                    let delta = ((remaining_distance / target_distance)
                                        * alpha.sin())
                                    .clamp(-1.0, 1.0)
                                    .asin();

                                    let beta = gamma - delta + alpha;

                                    let mut up = DVec3::ZERO;
                                    up = camera_position.normalize();
                                    let mut right = DVec3::ZERO;
                                    right = position_to_target_normal.cross(up);
                                    right = right.normalize();

                                    forward = up.cross(right).normalize();

                                    // Calculate new position to move to
                                    center = center
                                        .normalize()
                                        .multiply_by_scalar(center.magnitude() - distance);
                                    camera_position = camera_position.normalize();
                                    camera_position =
                                        camera_position.multiply_by_scalar(remaining_distance);

                                    // Pan
                                    let mut pMid = DVec3::ZERO;
                                    pMid = (up.multiply_by_scalar(beta.cos() - 1.)
                                        + forward.multiply_by_scalar(beta.sin()))
                                    .multiply_by_scalar(remaining_distance);
                                    camera_position = camera_position + pMid;

                                    up = center.normalize();
                                    forward = up.cross(right).normalize();

                                    let mut cMid = DVec3::ZERO;
                                    cMid = (up.multiply_by_scalar(beta.cos() - 1.)
                                        + forward.multiply_by_scalar(beta.sin()))
                                    .multiply_by_scalar(center.magnitude());
                                    center = center + cMid;

                                    // Update camera
                                    // Set new position
                                    globe_camera.position = camera_position;

                                    // Set new direction
                                    globe_camera.direction =
                                        center.subtract(camera_position).normalize();
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
                                    globe_camera.update_camera_matrix(&mut transform);
                                    return;
                                }
                            } else {
                                let position_normal = center_position.unwrap().normalize();
                                let picked_normal =
                                    globe_camera_control._zoom_world_position.normalize();
                                let dot_product = picked_normal.dot(position_normal);

                                //夹角在0-90度之间
                                if dot_product > 0.0 && dot_product < 1.0 {
                                    let angle = acos_clamped(dot_product);
                                    let axis = picked_normal.cross(position_normal);

                                    let denom = {
                                        //大于20度时，denom为相机高度的0.75倍，小于20度时denom为相机高度减去distance
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

                        globe_camera_control._rotating_zoom = !zoom_on_vector;
                    }

                    if (!same_start_position && zoom_on_vector) || zooming_on_vector {
                        let ray;
                        let zoomMouseStart = SceneTransforms::wgs84ToWindowCoordinates(
                            &globe_camera_control._zoom_world_position,
                            &window_size,
                            &to_mat4_64(&global_transform.compute_matrix()),
                            &to_mat4_64(&projection.get_projection_matrix()),
                        );
                        if start_position.eq(&globe_camera_control._zoom_mouse_start)
                            && zoomMouseStart.is_some()
                        {
                            let v = zoomMouseStart.unwrap();
                            ray = globe_camera.getPickRay(&v, &window_size)
                        } else {
                            ray = globe_camera.getPickRay(&start_position, &window_size);
                        }

                        let rayDirection = ray.direction;

                        globe_camera.move_direction(&rayDirection, distance);

                        globe_camera_control._zooming_on_vector = true;
                    } else {
                        globe_camera.zoom_in(Some(distance));
                    }

                    if !globe_camera_control._camera_underground {
                        globe_camera.set_view(
                            None,
                            Some(SetViewOrientation::HeadingPitchRoll(hpr)),
                            None,
                            None,
                        );
                    }
                    globe_camera.update_camera_matrix(&mut transform);
                    // println!("controlevent zoom {:?}", data);
                }

                ControlEvent::Spin(data) => {
                    // println!("controlevent spin {:?}", data);
                    let start_position = aggregator
                        .get_start_mouse_position("LEFT_DRAG", &event_start_position_wrap);
                    let mut movement = data.movement.clone();
                    spin3D(
                        &mut globe_camera_control,
                        &mut globe_camera,
                        &start_position,
                        &mut movement,
                        &window_size,
                    );
                    globe_camera.update_camera_matrix(&mut transform);
                }

                ControlEvent::Tilt(data) => {
                    // println!("controlevent tilt {:?}", data);
                    let start_position = aggregator
                        .get_start_mouse_position("MIDDLE_DRAG", &event_start_position_wrap);
                    let mut movement = data.movement.clone();
                    tilt3D(
                        &mut globe_camera_control,
                        &mut globe_camera,
                        &start_position,
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
    start_position: &DVec2,
    movement: &MovementState,
    window_size: &DVec2,
) {
    let ellipsoid = Ellipsoid::WGS84;
    let min_height = controller.minimum_zoom_distance * 0.25;
    let height = ellipsoid
        .cartesianToCartographic(&camera.get_position_wc())
        .unwrap()
        .height;
    if height - min_height - 1.0 < EPSILON3
        && movement.end_position.y - movement.start_position.y < 0.
    {
        return;
    }

    let mut window_position = DVec2::ZERO;
    window_position.x = window_size[0] / 2.;
    window_position.y = window_size[1] / 2.;
    let ray = camera.getPickRay(&window_position, window_size);

    let center;
    let intersection = IntersectionTests::rayEllipsoid(&ray, Some(&ellipsoid));
    if intersection.is_some() {
        let intersection = intersection.unwrap();
        center = Ray::getPoint(&ray, intersection.start);
    } else if height > controller._minimum_track_ball_height {
        let grazing_altitude_location =
            IntersectionTests::grazing_altitude_location(&ray, Some(&ellipsoid));
        if grazing_altitude_location.is_none() {
            return;
        }
        let grazing_altitude_location = grazing_altitude_location.unwrap();
        let mut grazing_altitude_cart = ellipsoid
            .cartesianToCartographic(&grazing_altitude_location)
            .unwrap();
        grazing_altitude_cart.height = 0.0;
        center = ellipsoid.cartographicToCartesian(&grazing_altitude_cart);
    } else {
        controller._looking = true;
        let up = ellipsoid.geodeticSurfaceNormal(&camera.position);
        look3D(
            controller,
            camera,
            start_position,
            movement,
            up,
            window_size,
        );
        controller._tilt_center_mouse_position = start_position.clone();
        return;
    }

    let transform = Transforms::eastNorthUpToFixedFrame(&center, None);

    let oldEllipsoid = controller._ellipsoid;
    controller._ellipsoid = Ellipsoid::UNIT_SPHERE;
    controller._rotate_factor = 1.0;
    controller._rotate_rate_range_adjustment = 1.0;

    let old_transform = camera.get_transform().clone();
    camera._setTransform(&transform);

    rotate3D(
        controller,
        camera,
        start_position,
        movement,
        window_size,
        Some(DVec3::UNIT_Z),
        None,
        None,
    );

    camera._setTransform(&old_transform);
    controller._ellipsoid = oldEllipsoid;

    let radius = oldEllipsoid.maximum_radius;
    controller._rotate_factor = 1.0 / radius;
    controller._rotate_rate_range_adjustment = radius;
}
fn tilt3D(
    controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    start_position: &DVec2,
    movement: &MovementState,
    window_size: &DVec2,
) {
    if !(camera.get_transform() == DMat4::IDENTITY) {
        return;
    }

    // if movement.angleAndHeight.is_some() {
    //     movement = movement.angleAndHeight;
    // }

    if !start_position.eq(&controller._tilt_center_mouse_position) {
        controller._tilt_on_ellipsoid = false;
        controller._looking = false;
    }

    if controller._looking {
        let up = Ellipsoid::WGS84.geodeticSurfaceNormal(&camera.position);
        look3D(
            controller,
            camera,
            start_position,
            movement,
            up,
            window_size,
        );
        return;
    }
    let cartographic = Ellipsoid::WGS84
        .cartesianToCartographic(&camera.position)
        .unwrap();

    if controller._tilt_on_ellipsoid
        || cartographic.height > controller._minimum_collision_terrain_height
    {
        controller._tilt_on_ellipsoid = true;
        tilt3DOnEllipsoid(controller, camera, start_position, movement, window_size);
    } else {
        // tilt3DOnTerrain(controller, start_position, movement);
        panic!("暂时没有地形")
    }
}

fn spin3D(
    controller: &mut GlobeCameraControl,

    camera: &mut GlobeCamera,
    start_position: &DVec2,
    movement: &mut MovementState,
    window_size: &DVec2,
) {
    let _camera_underground = controller._camera_underground;
    let mut ellipsoid = Ellipsoid::WGS84;

    if !camera.get_transform().eq(&DMat4::IDENTITY) {
        rotate3D(
            controller,
            camera,
            start_position,
            movement,
            window_size,
            None,
            None,
            None,
        );
        return;
    }

    let magnitude;
    let mut radii;

    let up = ellipsoid.geodeticSurfaceNormal(&camera.position);

    if start_position.eq(&controller._rotate_mouse_position) {
        if controller._looking {
            look3D(
                controller,
                camera,
                start_position,
                movement,
                up,
                window_size,
            );
        } else if controller._rotating {
            rotate3D(
                controller,
                camera,
                start_position,
                movement,
                window_size,
                None,
                None,
                None,
            );
        } else if controller._strafing {
            continueStrafing(controller, camera, movement, window_size);
        } else {
            if camera.position.magnitude() < controller._rotate_start_position.length() {
                // Pan action is no longer valid if camera moves below the pan ellipsoid
                return;
            }
            magnitude = controller._rotate_start_position.length();
            radii = DVec3::ZERO;
            radii.x = magnitude;
            radii.y = magnitude;
            radii.z = magnitude;
            ellipsoid = Ellipsoid::from_vec3(radii);
            pan3D(controller, camera, start_position, movement, window_size);
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
    let _globe = false;
    let spin_3d_pick = camera.pick_ellipsoid(&movement.start_position, window_size);
    if spin_3d_pick.is_some() {
        pan3D(controller, camera, start_position, movement, window_size);
        controller._rotate_start_position = spin_3d_pick.unwrap();
    } else if height > controller._minimum_track_ball_height {
        controller._rotating = true;
        rotate3D(
            controller,
            camera,
            start_position,
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
            start_position,
            movement,
            None,
            window_size,
        );
    }
    controller._rotate_mouse_position = start_position.clone();
}
fn rotate3D(
    controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    _start_position: &DVec2,
    movement: &MovementState,
    window_size: &DVec2,
    constrained_axis: Option<DVec3>,
    rotate_only_vertical: Option<bool>,
    rotate_only_horizontal: Option<bool>,
) {
    let rotate_only_vertical = rotate_only_vertical.unwrap_or(false);
    let rotate_only_horizontal = rotate_only_horizontal.unwrap_or(false);

    let oldAxis = camera.constrained_axis;
    if constrained_axis.is_some() {
        camera.constrained_axis = constrained_axis;
    }

    let rho = camera.position.magnitude();
    let mut rotateRate =
        controller._rotate_factor * (rho - controller._rotate_rate_range_adjustment);

    if rotateRate > controller._maximum_rotate_rate {
        rotateRate = controller._maximum_rotate_rate;
    }

    if rotateRate < controller._minimum_rotate_rate {
        rotateRate = controller._minimum_rotate_rate;
    }

    let mut phi_window_ratio =
        ((movement.start_position.x - movement.end_position.x) / window_size.x) as f64;
    let mut theta_window_ratio =
        ((movement.start_position.y - movement.end_position.y) / window_size.y) as f64;
    phi_window_ratio = phi_window_ratio.min(controller.maximum_movement_ratio);
    theta_window_ratio = theta_window_ratio.min(controller.maximum_movement_ratio);

    let delta_phi = rotateRate * phi_window_ratio * PI * 2.0;
    let delta_theta = rotateRate * theta_window_ratio * PI;

    if !rotate_only_vertical {
        camera.rotate_right(Some(delta_phi));
    }

    if !rotate_only_horizontal {
        camera.rotate_up(Some(delta_theta));
    }

    camera.constrained_axis = oldAxis;
}

fn pan3D(
    controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    start_position: &DVec2,
    movement: &MovementState,
    window_size: &DVec2,
) {
    let start_mouse_position = movement.start_position.clone();
    let end_mouse_position = movement.end_position.clone();

    let p0 = camera.pick_ellipsoid(&start_mouse_position, window_size);
    let p1 = camera.pick_ellipsoid(&end_mouse_position, window_size);

    if p0.is_none() || p1.is_none() {
        controller._rotating = true;
        rotate3D(
            controller,
            camera,
            start_position,
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

    p0 = camera.world_to_camera_coordinates(&p0);
    p1 = camera.world_to_camera_coordinates(&p1);

    if camera.constrained_axis.is_none() {
        p0 = p0.normalize();
        p1 = p1.normalize();
        let dot = p0.dot(p1);
        let axis = p0.cross(p1);

        if dot < 1.0 && !axis.equals_epsilon(DVec3::ZERO, Some(EPSILON14), None) {
            // dot is in [0, 1]
            let angle = dot.acos();
            camera.rotate(axis, Some(angle));
        }
    } else {
        let basis0 = camera.constrained_axis.unwrap();
        let mut basis1 = basis0.most_orthogonal_axis();
        basis1 = basis1.cross(basis0);
        basis1 = basis1.normalize();
        let basis2 = basis0.cross(basis1);

        let start_rho = p0.magnitude();
        let start_dot = basis0.dot(p0);
        let start_theta = (start_dot / start_rho).acos();
        let mut start_rej = basis0.multiply_by_scalar(start_dot);
        start_rej = p0.subtract(start_rej).normalize();

        let end_rho = p1.magnitude();
        let end_dot = basis0.dot(p1);
        let end_theta = (end_dot / end_rho).acos();
        let mut end_rej = basis0.multiply_by_scalar(end_dot);
        end_rej = p1.subtract(end_rej).normalize();

        let mut start_phi = (start_rej.dot(basis1)).acos();
        if start_rej.dot(basis2) < 0. {
            start_phi = TAU - start_phi;
        }

        let mut end_phi = (end_rej.dot(basis1)).acos();
        if end_rej.dot(basis2) < 0. {
            end_phi = TAU - end_phi;
        }

        let delta_phi = start_phi - end_phi;

        let east;
        if basis0.equals_epsilon(camera.position, Some(EPSILON2), None) {
            east = camera.right;
        } else {
            east = basis0.cross(camera.position);
        }

        let plane_normal = basis0.cross(east);
        let side0 = plane_normal.dot(p0.subtract(basis0));
        let side1 = plane_normal.dot(p1.subtract(basis0));

        let delta_theta;
        if side0 > 0. && side1 > 0. {
            delta_theta = end_theta - start_theta;
        } else if side0 > 0. && side1 <= 0. {
            if camera.position.dot(basis0) > 0. {
                delta_theta = -start_theta - end_theta;
            } else {
                delta_theta = start_theta + end_theta;
            }
        } else {
            delta_theta = start_theta - end_theta;
        }

        camera.rotate_right(Some(delta_phi));
        camera.rotate_up(Some(delta_theta));
    }
}

fn look3D(
    controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    _start_position: &DVec2,
    movement: &MovementState,
    rotation_axis: Option<DVec3>,
    window_size: &DVec2,
) {
    let mut start_pos = DVec2::ZERO;
    start_pos.x = movement.start_position.x as f64;
    start_pos.y = 0.0;
    let mut end_pos = DVec2::ZERO;
    end_pos.x = movement.end_position.x as f64;
    end_pos.y = 0.0;

    let mut start_ray = camera.getPickRay(&start_pos, window_size);
    let mut end_ray = camera.getPickRay(&end_pos, window_size);
    let mut angle = 0.0;
    let mut start;
    let mut end;
    start = start_ray.direction;
    end = end_ray.direction;

    let mut dot = start.dot(end);
    if dot < 1.0 {
        // dot is in [0, 1]
        angle = dot.acos();
    }
    angle = if movement.start_position.x > movement.end_position.x {
        -angle
    } else {
        angle
    };

    let horizontalRotationAxis = controller._horizontal_rotation_axis;
    if rotation_axis.is_some() {
        camera.look(&rotation_axis.unwrap(), Some(-angle));
    } else if horizontalRotationAxis.is_some() {
        camera.look(&horizontalRotationAxis.unwrap(), Some(-angle));
    } else {
        camera.look_left(Some(angle));
    }

    start_pos.x = 0.0;
    start_pos.y = movement.start_position.y;
    end_pos.x = 0.0;
    end_pos.y = movement.end_position.y;

    start_ray = camera.getPickRay(&start_pos, window_size);
    end_ray = camera.getPickRay(&end_pos, window_size);
    angle = 0.0;

    start = start_ray.direction;
    end = end_ray.direction;

    dot = start.dot(end);
    if dot < 1.0 {
        // dot is in [0, 1]
        angle = dot.acos();
    }
    angle = if movement.start_position.y > movement.end_position.y {
        -angle
    } else {
        angle
    };

    let rotation_axis = rotation_axis.unwrap_or(horizontalRotationAxis.unwrap_or(DVec3::ZERO));
    if rotation_axis != DVec3::ZERO {
        let direction = camera.direction;
        let negativeRotationAxis = rotation_axis.neg();
        let north_parallel = direction.equals_epsilon(rotation_axis, Some(EPSILON2), None);
        let south_parallel = direction.equals_epsilon(negativeRotationAxis, Some(EPSILON2), None);
        if !north_parallel && !south_parallel {
            dot = direction.dot(rotation_axis);
            let mut angle_to_axis = acos_clamped(dot);
            if angle > 0. && angle > angle_to_axis {
                angle = angle_to_axis - EPSILON4;
            }

            dot = direction.dot(negativeRotationAxis);
            angle_to_axis = acos_clamped(dot);
            if angle < 0. && -angle > angle_to_axis {
                angle = -angle_to_axis + EPSILON4;
            }

            let tangent = rotation_axis.cross(direction);
            camera.look(&tangent, Some(angle));
        } else if (north_parallel && angle < 0.) || (south_parallel && angle > 0.) {
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
    let original_end_position = movement.end_position;
    let inertial_delta = movement.end_position - movement.start_position;
    let mut end_position = controller._strafe_end_mouse_position;
    end_position = end_position + inertial_delta;
    movement.end_position = end_position;
    strafe(
        controller,
        camera,
        movement,
        &controller._strafe_start_position.clone(),
        window_size,
    );
    movement.end_position = original_end_position;
}

fn strafe(
    _controller: &mut GlobeCameraControl,
    camera: &mut GlobeCamera,
    movement: &MovementState,
    strafe_start_position: &DVec3,
    window_size: &DVec2,
) {
    let ray = camera.getPickRay(&movement.end_position, window_size);

    let mut direction = camera.direction.clone();
    let plane = Plane::fromPointNormal(&strafe_start_position, &direction);
    let intersection = IntersectionTests::rayPlane(&ray, &plane);
    if intersection.is_none() {
        return;
    }
    let intersection = intersection.unwrap();
    direction = *strafe_start_position - intersection;

    camera.position = camera.position + direction;
}
