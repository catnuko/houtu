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
mod camera_event_aggregator;
mod camera_new;
mod camera_old;
mod debug_system;
mod egui;
mod pan_orbit;
use self::{
    camera_new::CameraControlPlugin, debug_system::debug_system, pan_orbit::pan_orbit_camera,
};
pub use camera_new::GlobeCamera;
use camera_old::{PanOrbitCamera, PanOrbitCameraPlugin};
use houtu_scene::{Projection, *};

pub struct CameraPlugin;

impl bevy::app::Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Msaa::default())
            .add_plugin(CameraControlPlugin)
            .add_plugin(debug_system::Plugin)
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
    pub minimumZoomDistance: f64,
    pub maximumZoomDistance: f64,
    pub _minimumZoomRate: f64,
    pub _maximumZoomRate: f64,
    pub _maximumRotateRate: f64,
    pub _minimumRotateRate: f64,
    pub _minimumUndergroundPickDistance: f64,
    pub _maximumUndergroundPickDistance: f64,
    pub enableCollisionDetection: bool,
    pub _maxCoord: DVec3,
    pub _zoomFactor: f64,
    pub maximumMovementRatio: f64,
    pub bounceAnimationTime: f64,
    pub _tiltCenterMousePosition: DVec2,
    pub _tiltCenter: DVec2,
    pub _rotateMousePosition: DVec2,
    pub _rotateStartPosition: DVec3,
    pub _strafeStartPosition: DVec3,
    pub _strafeMousePosition: DVec2,
    pub _strafeEndMousePosition: DVec2,
    pub _zoomMouseStart: DVec2,
    pub _zoomWorldPosition: DVec3,
    pub minimumTrackBallHeight: f64,
    pub _minimumTrackBallHeight: f64,
    pub minimumCollisionTerrainHeight: f64,
    pub _minimumCollisionTerrainHeight: f64,
    pub minimumPickingTerrainHeight: f64,
    pub _minimumPickingTerrainHeight: f64,
    pub minimumPickingTerrainDistanceWithInertia: f64,
    pub _useZoomWorldPosition: bool,
    pub _tiltCVOffMap: bool,
    pub _looking: bool,
    pub _rotating: bool,
    pub _strafing: bool,
    pub _zoomingOnVector: bool,
    pub _zoomingUnderground: bool,
    pub _rotatingZoom: bool,
    pub _adjustedHeightForTerrain: bool,
    pub _cameraUnderground: bool,
    pub _tiltOnEllipsoid: bool,
    pub _rotateFactor: f64,
    pub _rotateRateRangeAdjustment: f64,
    pub _horizontalRotationAxis: Option<DVec3>,
    pub _ellipsoid: Ellipsoid,
}
impl Default for GlobeCameraControl {
    fn default() -> Self {
        let max_coord = GeographicProjection::WGS84.project(&Cartographic::new(PI, FRAC_PI_2, 0.));
        let ellipsoid = Ellipsoid::WGS84;
        Self {
            pixelRatio: 1.0,
            drawingBufferWidth: 0,
            drawingBufferHeight: 0,
            _cameraUnderground: false,
            _zoomFactor: 5.0,
            maximumMovementRatio: 1.0,
            bounceAnimationTime: 3.0,
            minimumZoomDistance: 1.0,
            maximumZoomDistance: MAX,
            _minimumZoomRate: 20.0,
            _maximumZoomRate: 5906376272000.0,
            enableCollisionDetection: true,
            _rotatingZoom: false,
            _zoomingOnVector: false,
            _useZoomWorldPosition: false,
            _zoomingUnderground: false,
            _maxCoord: max_coord,
            _tiltCenterMousePosition: DVec2::new(-1.0, -1.0),
            _tiltCenter: DVec2::new(0.0, 0.0),
            _rotateMousePosition: DVec2::new(-1.0, -1.0),
            _rotateStartPosition: DVec3::new(0.0, 0.0, 0.0),
            _strafeStartPosition: DVec3::new(0.0, 0.0, 0.0),
            _strafeMousePosition: DVec2::new(0.0, 0.0),
            _strafeEndMousePosition: DVec2::new(0.0, 0.0),
            _zoomMouseStart: DVec2::new(-1.0, -1.0),
            _zoomWorldPosition: DVec3::new(0.0, 0.0, 0.0),
            minimumTrackBallHeight: 7500000.0,
            _minimumTrackBallHeight: 7500000.0,
            minimumCollisionTerrainHeight: 150000.0,
            _minimumCollisionTerrainHeight: 150000.0,
            minimumPickingTerrainHeight: 150000.0,
            _minimumPickingTerrainHeight: 150000.0,
            minimumPickingTerrainDistanceWithInertia: 4000.0,
            _tiltCVOffMap: false,
            _looking: false,
            _rotating: false,
            _strafing: false,
            _adjustedHeightForTerrain: false,
            _tiltOnEllipsoid: false,
            _maximumRotateRate: 1.77,
            _minimumRotateRate: 1.0 / 5000.0,
            _minimumUndergroundPickDistance: 2000.0,
            _maximumUndergroundPickDistance: 10000.0,
            _ellipsoid: ellipsoid,
            _rotateFactor: 1.0,
            _rotateRateRangeAdjustment: 1.0,
            _horizontalRotationAxis: None,
        }
    }
}
impl GlobeCameraControl {
    pub fn update(&mut self, camera: &mut GlobeCamera) {
        if camera.get_transform() != DMat4::IDENTITY {
            self._ellipsoid = Ellipsoid::UNIT_SPHERE;
        } else {
            self._ellipsoid = Ellipsoid::WGS84;
        }
        self._rotateFactor = 1.0 / self._ellipsoid.maximumRadius;
        self._rotateRateRangeAdjustment = self._ellipsoid.maximumRadius;
    }
}
