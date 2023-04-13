use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
};
use geodesy::preamble::*;
pub mod controller;
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        app.add_system(controller::pan_orbit_camera);
    }
}
impl Default for Plugin {
    fn default() -> Self {
        Self {}
    }
}

fn setup(mut commands: Commands) {
    let ellipsoid = Ellipsoid::named("WGS84").unwrap();
    let x = ellipsoid.semimajor_axis() as f32;
    let translation = Vec3::new(-2.0, 2.5, 5.0);
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(x + 10000000., x, x)
                .looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
            ..default()
        },
        controller::PanOrbitCamera {
            radius:x,
            ..Default::default()
        },
    ));
}

// fn scroll_events(mut scroll_evr: EventReader<MouseWheel>) {
//     use bevy::input::mouse::MouseScrollUnit;
//     for ev in scroll_evr.iter() {
//         match ev.unit {
//             MouseScrollUnit::Line => {
//                 println!(
//                     "Scroll (line units): vertical: {}, horizontal: {}",
//                     ev.y, ev.x
//                 );
//             }
//             MouseScrollUnit::Pixel => {
//                 println!(
//                     "Scroll (pixel units): vertical: {}, horizontal: {}",
//                     ev.y, ev.x
//                 );
//             }
//         }
//     }
// }
// fn mouse_button_input(buttons: Res<Input<MouseButton>>) {
//     if buttons.just_pressed(MouseButton::Left) {
//         // Left button was pressed
//         println!("Left button was pressed");
//     }
//     if buttons.just_released(MouseButton::Left) {
//         // Left Button was released
//         println!("Left Button was released");
//     }
//     if buttons.pressed(MouseButton::Right) {
//         // Right Button is being held down
//         println!("Right Button is being held down");
//     }
// }
// fn mouse_motion(mut motion_evr: EventReader<MouseMotion>) {
//     for ev in motion_evr.iter() {
//         println!("Mouse moved: X: {} px, Y: {} px", ev.delta.x, ev.delta.y);
//     }
// }
