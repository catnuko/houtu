use std::f64::consts::PI;

use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_prototype_debug_lines::*;
use houtu_scene::{Cartesian2, Cartesian3, Ellipsoid, HeadingPitchRoll};

use crate::plugins::camera::camera_new::LookAtTransformOffset;

use super::{camera_event_aggregator::MouseEvent, camera_new::GlobeCamera};
pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_system(debug_system);
    }
}
pub fn debug_system(
    mut commands: Commands,
    primary_query: Query<&Window, With<PrimaryWindow>>,

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

    for (entity, mut transform, mut projection, global_transform, mut globe_camera) in
        &mut orbit_cameras
    {
        for event in mouse_event_reader.iter() {
            match event {
                MouseEvent::LeftClick(position) => {
                    println!("window positon: x={},y={}", position.x, position.y);
                    let cartesian = globe_camera.pickEllipsoid(&position, &window_size);
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
                                cartogaphic.longitude, cartogaphic.latitude, cartogaphic.height
                            );
                        }
                    }
                    // globe_camera.look_at(
                    //     &DVec3::from_degrees(120.0, 28.0, Some(100000.), None),
                    //     LookAtTransformOffset::HeadingPitchRoll(HeadingPitchRoll::new(
                    //         0.0,
                    //         -PI / 2.,
                    //         0.0,
                    //     )),
                    // );
                    // globe_camera.update_camera_matrix(&mut transform);

                    //debug camera position direction right up
                    let ray = globe_camera.getPickRay(&position, &window_size);
                    let direction = globe_camera.position
                        + globe_camera.get_direction_wc().normalize() * 100000000.0;
                    let up =
                        globe_camera.position + globe_camera.get_up_wc().normalize() * 100000000.0;
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
                _ => {}
            }
        }
    }
}
