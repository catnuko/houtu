use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    prelude::*,
};
// use bevy_atmosphere::prelude::*;
use controllers::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin};

pub mod controller;
//复制进来的东西
pub mod controllers;

mod look_angles;
mod look_transform;

use bevy_atmosphere::prelude::*;
use houtu_scene::*;
pub use look_angles::*;
pub use look_transform::*;

pub struct CameraPlugin;

impl bevy::app::Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(AtmospherePlugin);
        app.insert_resource(Msaa::default())
            .add_plugin(LookTransformPlugin)
            .add_plugin(OrbitCameraPlugin::default())
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
    commands.spawn((Camera3dBundle::default(), AtmosphereCamera::default()));
    // .insert(OrbitCameraBundle::new(
    //     OrbitCameraController::default(),
    //     Vec3::new(x + 10000000., x + 10000000., x + 10000000.),
    //     Vec3::new(0., 0., 0.),
    //     Vec3::Y,
    // ));
}
