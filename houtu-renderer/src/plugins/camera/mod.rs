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
    window::PrimaryWindow,
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
        app.insert_resource(Msaa::default())
            .add_plugin(CameraControlPlugin)
            .add_startup_system(setup)
            .add_system(pan_orbit_camera);
    }
}
impl Default for CameraPlugin {
    fn default() -> Self {
        Self {}
    }
}

fn setup(mut commands: Commands, primary_query: Query<&Window, With<PrimaryWindow>>) {
    let Ok(primary) = primary_query.get_single() else {
        return;
    };
    let ellipsoid = Ellipsoid::WGS84;
    let x = ellipsoid.semimajor_axis() as f32;
    commands.spawn((
        Camera3dBundle {
            projection: bevy::prelude::Projection::Perspective(PerspectiveProjection {
                fov: (60.0 as f32).to_radians(),
                aspect_ratio: primary.width() / primary.height(),
                near: 1.,
                far: 500000000.0,
            }),
            ..Default::default()
        },
        GlobeCamera::default(),
        GlobeCameraControl::default(),
    ));
}
#[derive(Component)]
pub struct GlobeCameraControl {
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
            pixelRatio: 1.0,
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
