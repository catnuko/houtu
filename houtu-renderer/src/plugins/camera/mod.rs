use std::f32::consts::TAU;

use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    math::DVec3,
    prelude::*,
    render::primitives::Frustum,
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
            .add_startup_system(setup)
            .add_system(globe_map_camera_system);

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
            GlobeMapCamera::default(),
        ));
}
#[derive(Component)]
pub struct GlobeMapCamera {
    pub position_cartesian: DVec3,
    pub position_cartographic: Option<Cartographic>,
    pub culling_volume: CullingVolume,
    pub direction: DVec3,
    pub up: DVec3,
    pub right: DVec3,
    pub _sseDenominator: f64,
    pub drawingBufferWidth: u32,
    pub drawingBufferHeight: u32,
    pub pixelRatio: f64,
}
impl Default for GlobeMapCamera {
    fn default() -> Self {
        Self {
            pixelRatio: 1.0,
            ..Default::default()
        }
    }
}
fn globe_map_camera_system(
    mut query: Query<
        (
            &mut GlobeMapCamera,
            &mut Transform,
            &Frustum,
            &bevy::prelude::Projection,
            &bevy::prelude::Camera,
        ),
        (With<Camera3d>, Changed<Transform>),
    >,
    ellipsoid: Res<Ellipsoid>,
) {
    for (mut globe_map_camera, transform, frustum, projection, camera) in &mut query {
        globe_map_camera.position_cartesian = DVec3::new(
            transform.translation.x as f64,
            transform.translation.x as f64,
            transform.translation.x as f64,
        );
        let position_cartographic =
            ellipsoid.cartesianToCartographic(globe_map_camera.position_cartesian);
        globe_map_camera.position_cartographic = position_cartographic;
        globe_map_camera.culling_volume = CullingVolume::new(Some(
            frustum
                .planes
                .iter()
                .map(|x| {
                    let v = x.normal_d();
                    return houtu_scene::Plane {
                        normal: DVec3 {
                            x: v.x as f64,
                            y: v.y as f64,
                            z: v.z as f64,
                        },
                        distance: v.w as f64,
                    };
                })
                .collect(),
        ));
        let rotMat = Mat3::from_quat(transform.rotation);
        let x_axis = rotMat.x_axis;
        let y_axis = rotMat.y_axis;
        globe_map_camera.direction = DVec3::new(x_axis.x as f64, x_axis.y as f64, x_axis.z as f64);
        globe_map_camera.up = DVec3::new(y_axis.x as f64, y_axis.y as f64, y_axis.z as f64);
        globe_map_camera.right = globe_map_camera.direction.cross(globe_map_camera.up);
        if (globe_map_camera._sseDenominator == 0) {
            if let bevy::prelude::Projection::Perspective(p) = projection {
                globe_map_camera._sseDenominator = 2.0 * (0.5 * p.fov).tan();
            }
            if let Some(info) = camera.physical_target_size() {
                globe_map_camera.drawingBufferWidth = info.x;
                globe_map_camera.drawingBufferHeight = info.y;
            }
        }
    }
}
