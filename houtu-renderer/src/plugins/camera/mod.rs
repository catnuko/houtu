use std::{
    f32::consts::TAU,
    f64::{
        consts::{FRAC_PI_2, PI},
        MAX,
    },
};

use bevy::{
    input::mouse::{MouseButtonInput, MouseMotion, MouseWheel},
    math::{DMat4, DVec2, DVec3},
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
use houtu_scene::{Projection, *};

use self::{
    camera_new::{CameraControlPlugin, GlobeCamera},
    pan_orbit::pan_orbit_camera,
};

pub struct CameraPlugin;

impl bevy::app::Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        // app.add_plugin(AtmospherePlugin);
        app.insert_resource(Msaa::default())
            // .add_plugin(PanOrbitCameraPlugin)
            .add_plugin(CameraControlPlugin)
            .add_startup_system(setup)
            .add_system(pan_orbit_camera);
        // .add_system(globe_map_camera_system);

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
            GlobeCamera::default(),
            GlobeCameraControl::default(),
        ));
}
#[derive(Component)]
pub struct GlobeCameraControl {
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
    pub minimumZoomDistance: f64,
    pub maximumZoomDistance: f64,
    pub _minimumZoomRate: f64,
    pub _maximumZoomRate: f64,
    pub enableCollisionDetection: bool,
    pub _zoomMouseStart: Vec2,
    pub _rotatingZoom: bool,
    pub _zoomingOnVector: bool,
    pub _useZoomWorldPosition: bool,
    pub _zoomWorldPosition: DVec3,
    pub _zoomingUnderground: bool,
    pub _maxCoord: DVec3,
    pub _zoomFactor: f64,
    pub maximumMovementRatio: f64,
    pub bounceAnimationTime: f64,
}
impl Default for GlobeCameraControl {
    fn default() -> Self {
        let max_coord = GeographicProjection::WGS84.project(&Cartographic::new(PI, FRAC_PI_2, 0.));
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
            _zoomFactor: 5.0,
            maximumMovementRatio: 1.0,
            bounceAnimationTime: 3.0,
            minimumZoomDistance: 1.0,
            maximumZoomDistance: MAX,
            _minimumZoomRate: 20.0,
            _maximumZoomRate: 5906376272000.0,
            enableCollisionDetection: true,
            _zoomMouseStart: Vec2::ZERO,
            _rotatingZoom: false,
            _zoomingOnVector: false,
            _useZoomWorldPosition: false,
            _zoomWorldPosition: DVec3::ZERO,
            _zoomingUnderground: false,
            _maxCoord: max_coord,
        }
    }
}
impl GlobeCameraControl {
    fn getMagnitude(&self) -> f64 {
        return self.position_cartesian.magnitude();
    }
}

fn globe_map_camera_system(
    mut query: Query<
        (
            &mut GlobeCameraControl,
            &mut Transform,
            &Frustum,
            &bevy::prelude::Projection,
            &bevy::prelude::Camera,
        ),
        (With<Camera3d>, Changed<Transform>),
    >,
) {
    for (mut globe_camera_control, transform, frustum, projection, camera) in &mut query {
        globe_camera_control.position_cartesian = DVec3::new(
            transform.translation.x as f64,
            transform.translation.x as f64,
            transform.translation.x as f64,
        );
        let position_cartographic =
            Ellipsoid::WGS84.cartesianToCartographic(&globe_camera_control.position_cartesian);
        globe_camera_control.position_cartographic = position_cartographic;
        globe_camera_control.culling_volume = Some(CullingVolume::new(Some(
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
        globe_camera_control.direction =
            DVec3::new(x_axis.x as f64, x_axis.y as f64, x_axis.z as f64);
        globe_camera_control.up = DVec3::new(y_axis.x as f64, y_axis.y as f64, y_axis.z as f64);
        globe_camera_control.right = globe_camera_control
            .direction
            .cross(globe_camera_control.up);
        if (globe_camera_control._sseDenominator == 0.0) {
            if let bevy::prelude::Projection::Perspective(p) = projection {
                globe_camera_control._sseDenominator = (2.0 * (0.5 * p.fov) as f64).tan();
            }
            if let Some(info) = camera.physical_target_size() {
                globe_camera_control.drawingBufferWidth = info.x;
                globe_camera_control.drawingBufferHeight = info.y;
            }
        }
    }
}
