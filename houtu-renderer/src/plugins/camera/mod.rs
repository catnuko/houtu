use std::f32::consts::TAU;

use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    math::{DVec2, DVec3},
    prelude::*,
    render::primitives::Frustum,
};
mod camera;
mod camera_event_aggregator;
mod camera_new;
mod camera_old;
mod egui;
mod pan_orbit;
use camera_old::{PanOrbitCamera, PanOrbitCameraPlugin};

use bevy_atmosphere::prelude::*;
use houtu_scene::*;

use self::camera_new::{CameraControl, CameraControlPlugin};

pub struct CameraPlugin;

impl bevy::app::Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugin(AtmospherePlugin);
        app.insert_resource(Msaa::default())
            // .add_plugin(PanOrbitCameraPlugin)
            .add_plugin(CameraControlPlugin)
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
            // PanOrbitCamera {
            //     beta: TAU * 0.1,
            //     radius: x + 10000000.0,
            //     ..Default::default()
            // },
            CameraControl::default(),
            GlobeMapCamera::default(),
        ));
}
#[derive(Component)]
pub struct GlobeMapCamera {
    pub position_cartesian: DVec3,
    pub position_cartographic: Option<Cartographic>,
    pub culling_volume: Option<CullingVolume>,
    pub direction: DVec3,
    pub up: DVec3,
    pub right: DVec3,
    pub _sseDenominator: f64,
    pub drawingBufferWidth: u32,
    pub drawingBufferHeight: u32,
    pub pixelRatio: f64,
    pub _cameraUnderground: bool,
    pub _minimumPickingTerrainHeight: f64,
}
impl Default for GlobeMapCamera {
    fn default() -> Self {
        Self {
            position_cartesian: DVec3::ZERO,
            direction: DVec3::ZERO,
            up: DVec3::ZERO,
            right: DVec3::ZERO,
            position_cartographic: None,
            culling_volume: None,
            pixelRatio: 1.0,
            _sseDenominator: 0.0,
            drawingBufferWidth: 0,
            drawingBufferHeight: 0,
            _cameraUnderground: false,
            _minimumPickingTerrainHeight: 150000.0,
        }
    }
}

pub fn getPickRay(
    windowPosition: Vec2,
    window_size: &Vec2,
    projection: &PerspectiveProjection,
    camera: &GlobeMapCamera,
) -> houtu_scene::Ray {
    if window_size.x <= 0. && window_size.y <= 0. {
        return None;
    }
    return getPickRayPerspective(windowPosition, window_size, projection, camera);
}
pub fn getPickRayPerspective(
    windowPosition: Vec2,
    window_size: &Vec2,
    projection: &PerspectiveProjection,
    camera: &GlobeMapCamera,
) -> houtu_scene::Ray {
    let mut result = houtu_scene::Ray::default();
    let width = window_size.x as f64;
    let height = window_size.y as f64;
    let aspectRatio = width / height;
    let tanPhi = (projection.fov as f64 * 0.5).tan();
    let tanTheta = aspectRatio * tanPhi;
    let near = projection.near as f64;

    let x = (2.0 / width) * windowPosition.x as f64 - 1.0;
    let y = (2.0 / height) * (height - windowPosition.y as f64) - 1.0;

    let position = camera.position_cartesian;
    result.origin = position.clone();

    let mut nearCenter = camera.direction.multiply_by_scalar(near);
    nearCenter = position + nearCenter;
    let xDir = camera.right.multiply_by_scalar(x * near * tanTheta);
    let yDir = camera.up.multiply_by_scalar(y * near * tanPhi);
    let mut direction = nearCenter + xDir;
    direction = direction + yDir;
    direction = direction + position;
    direction = direction.normalize();
    result.direction = direction;
    return result;
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
        globe_map_camera.culling_volume = Some(CullingVolume::new(Some(
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
        )));
        let rotMat = Mat3::from_quat(transform.rotation);
        let x_axis = rotMat.x_axis;
        let y_axis = rotMat.y_axis;
        globe_map_camera.direction = DVec3::new(x_axis.x as f64, x_axis.y as f64, x_axis.z as f64);
        globe_map_camera.up = DVec3::new(y_axis.x as f64, y_axis.y as f64, y_axis.z as f64);
        globe_map_camera.right = globe_map_camera.direction.cross(globe_map_camera.up);
        if (globe_map_camera._sseDenominator == 0.0) {
            if let bevy::prelude::Projection::Perspective(p) = projection {
                globe_map_camera._sseDenominator = (2.0 * (0.5 * p.fov) as f64).tan();
            }
            if let Some(info) = camera.physical_target_size() {
                globe_map_camera.drawingBufferWidth = info.x;
                globe_map_camera.drawingBufferHeight = info.y;
            }
        }
    }
}
