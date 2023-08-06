

use bevy::{
    math::{DVec2},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_prototype_debug_lines::*;
use houtu_scene::{Ellipsoid};

use crate::plugins::camera::{GlobeCamera, MouseEvent};

use super::ui_state::UiState;

pub fn debug_system(
    primary_query: Query<&Window, With<PrimaryWindow>>,
    state: Res<UiState>,
    mut mouse_event_reader: EventReader<MouseEvent>,
    mut orbit_cameras: Query<(
        Entity,
        &mut Transform,
        &mut Projection,
        &mut GlobalTransform,
        &mut GlobeCamera,
    )>,
    mut lines: ResMut<DebugLines>,
) {
    let Ok(primary) = primary_query.get_single() else {
        return;
    };
    let window_size = DVec2 {
        x: primary.width() as f64,
        y: primary.height() as f64,
    };

    for (_entity, _transform, _projection, _global_transform, mut globe_camera) in
        &mut orbit_cameras
    {
        for event in mouse_event_reader.iter() {
            match event {
                MouseEvent::LeftClick(position) => {
                    if state.debug_camera_position {
                        let camera_llh = globe_camera.get_position_cartographic();
                        println!(
                            "camera lonlatheight: {},{},{}",
                            camera_llh.longitude.to_degrees(),
                            camera_llh.latitude.to_degrees(),
                            camera_llh.height
                        );
                        let camera_cartesian3 = globe_camera.get_position_wc();
                        println!(
                            "camera cartesian3: {},{},{}",
                            camera_cartesian3.x, camera_cartesian3.y, camera_cartesian3.z
                        );
                        println!("window positon: x={},y={}", position.x, position.y);
                        let cartesian = globe_camera.pick_ellipsoid(&position, &window_size);
                        if cartesian.is_some() {
                            let cartesian = cartesian.unwrap();
                            println!(
                                "cartesian: x={},y={},z={}",
                                cartesian.x, cartesian.y, cartesian.z
                            );
                            let cartographic = Ellipsoid::WGS84.cartesianToCartographic(&cartesian);
                            if cartographic.is_some() {
                                let cartogaphic = cartographic.unwrap();
                                println!(
                                    "cartogaphic: lon={},lat={},height={}",
                                    cartogaphic.longitude.to_degrees(),
                                    cartogaphic.latitude.to_degrees(),
                                    cartogaphic.height
                                );
                            }
                        }
                    }

                    if state.debug_camera_dur {
                        let _ray = globe_camera.getPickRay(&position, &window_size);
                        let direction = globe_camera.position
                            + globe_camera.get_direction_wc().normalize() * 100000000.0;
                        let up = globe_camera.position
                            + globe_camera.get_up_wc().normalize() * 100000000.0;
                        let right = globe_camera.position
                            + globe_camera.get_right_wc().normalize() * 100000000.0;
                        lines.line_colored(
                            Vec3::new(
                                globe_camera.position.x as f32,
                                globe_camera.position.y as f32,
                                globe_camera.position.z as f32,
                            ),
                            Vec3::new(direction.x as f32, direction.y as f32, direction.z as f32),
                            3.0,
                            Color::RED,
                        );
                        lines.line_colored(
                            Vec3::new(
                                globe_camera.position.x as f32,
                                globe_camera.position.y as f32,
                                globe_camera.position.z as f32,
                            ),
                            Vec3::new(right.x as f32, right.y as f32, right.z as f32),
                            3.0,
                            Color::GREEN,
                        );
                        lines.line_colored(
                            Vec3::new(
                                globe_camera.position.x as f32,
                                globe_camera.position.y as f32,
                                globe_camera.position.z as f32,
                            ),
                            Vec3::new(up.x as f32, up.y as f32, up.z as f32),
                            3.0,
                            Color::BLUE,
                        );
                    }
                }
                _ => {}
            }
        }
    }
}
