use std::f32::consts::TAU;

use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
};
mod camera;
mod camera_old;
mod egui;
use camera_old::{PanOrbitCamera, PanOrbitCameraPlugin};

use bevy_atmosphere::prelude::*;
use houtu_scene::*;

pub struct CameraPlugin;

impl bevy::app::Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugin(AtmospherePlugin);
        app.insert_resource(Msaa::default())
            .add_plugin(PanOrbitCameraPlugin)
            .add_startup_system(setup);

        // app.add_system(controller::pan_orbit_camera);
    }
}
impl Default for CameraPlugin {
    fn default() -> Self {
        Self {}
    }
}

fn setup(mut commands: Commands) {
    let ellipsoid = Ellipsoid::WGS84;
    let x = ellipsoid.semimajor_axis() as f32;
    commands
        // .spawn((Camera3dBundle::default(), AtmosphereCamera::default()))
        .spawn((
            Camera3dBundle::default(),
            PanOrbitCamera {
                beta: TAU * 0.1,
                radius: x + 10000000.0,
                ..Default::default()
            },
        ));
}
