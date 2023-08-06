use std::f64::{
    consts::{FRAC_PI_2, PI},
    MAX,
};

use bevy::{
    math::{DMat4, DVec2, DVec3},
    prelude::*,
    window::PrimaryWindow,
};
mod camera_event_aggregator;
mod egui;
mod globe_camra;
mod pan_orbit;
use self::{globe_camra::CameraControlPlugin, pan_orbit::pan_orbit_camera};
pub use camera_event_aggregator::MouseEvent;
pub use globe_camra::GlobeCamera;

use houtu_scene::{Projection, *};

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
    let _x = ellipsoid.semimajor_axis() as f32;
    // let perspective_projection = PerspectiveProjection {
    //     fov: (60.0 as f32).to_radians(),
    //     aspect_ratio: primary.width() / primary.height(),
    //     near: 0.1,
    //     far: 10000000000.0,
    // };
    // frustum construction code copied from Bevy
    // let view_projection = perspective_projection.get_projection_matrix();
    // let transform = Transform::default();
    // let projection = bevy::prelude::Projection::Perspective(perspective_projection);
    // let frustum = Frustum::from_view_projection(&view_projection);
    commands.spawn((
        Camera3dBundle {
            projection: bevy::prelude::Projection::Perspective(PerspectiveProjection {
                fov: (60.0 as f32).to_radians(),
                aspect_ratio: primary.width() / primary.height(),
                near: 0.1,
                far: 10000000000.0,
            }),
            // transform,
            // frustum,
            // transform: Transform::from_translation(Vec3 {
            //     x: x + 10000000.,
            //     y: x + 10000000.,
            //     z: x + 10000000.,
            // })
            // .looking_at(Vec3::ZERO, Vec3::Z),
            ..Default::default()
        },
        GlobeCamera::default(),
        GlobeCameraControl::default(),
    ));
}
#[derive(Component)]
pub struct GlobeCameraControl {
    pub drawing_buffer_width: u32,
    pub drawing_buffer_height: u32,
    pub pixel_ratio: f64,
    pub minimum_zoom_distance: f64,
    pub maximum_zoom_distance: f64,
    pub _minimum_zoom_rate: f64,
    pub _maximum_zoom_rate: f64,
    pub _maximum_rotate_rate: f64,
    pub _minimum_rotate_rate: f64,
    pub _minimum_underground_pick_distance: f64,
    pub _maximum_underground_pick_distance: f64,
    pub enable_collision_detection: bool,
    pub _max_coord: DVec3,
    pub _zoom_factor: f64,
    pub maximum_movement_ratio: f64,
    pub bounce_animation_time: f64,
    pub _tilt_center_mouse_position: DVec2,
    pub _tilt_center: DVec2,
    pub _rotate_mouse_position: DVec2,
    pub _rotate_start_position: DVec3,
    pub _strafe_start_position: DVec3,
    pub _strafe_mouse_position: DVec2,
    pub _strafe_end_mouse_position: DVec2,
    pub _zoom_mouse_start: DVec2,
    pub _zoom_world_position: DVec3,
    pub minimum_track_ball_height: f64,
    pub _minimum_track_ball_height: f64,
    pub minimum_collision_terrain_height: f64,
    pub _minimum_collision_terrain_height: f64,
    pub minimum_picking_terrain_height: f64,
    pub _minimum_picking_terrain_height: f64,
    pub minimum_picking_terrain_distance_with_inertia: f64,
    pub _use_zoom_world_position: bool,
    pub _tilt_cv_off_map: bool,
    pub _looking: bool,
    pub _rotating: bool,
    pub _strafing: bool,
    pub _zooming_on_vector: bool,
    pub _zooming_underground: bool,
    pub _rotating_zoom: bool,
    pub _adjusted_height_for_terrain: bool,
    pub _camera_underground: bool,
    pub _tilt_on_ellipsoid: bool,
    pub _rotate_factor: f64,
    pub _rotate_rate_range_adjustment: f64,
    pub _horizontal_rotation_axis: Option<DVec3>,
    pub _ellipsoid: Ellipsoid,
}
impl Default for GlobeCameraControl {
    fn default() -> Self {
        let max_coord = GeographicProjection::WGS84.project(&Cartographic::new(PI, FRAC_PI_2, 0.));
        let ellipsoid = Ellipsoid::WGS84;
        Self {
            pixel_ratio: 1.0,
            drawing_buffer_width: 0,
            drawing_buffer_height: 0,
            _camera_underground: false,
            _zoom_factor: 5.0,
            maximum_movement_ratio: 1.0,
            bounce_animation_time: 3.0,
            minimum_zoom_distance: 1.0,
            maximum_zoom_distance: MAX,
            _minimum_zoom_rate: 20.0,
            _maximum_zoom_rate: 5906376272000.0,
            enable_collision_detection: true,
            _rotating_zoom: false,
            _zooming_on_vector: false,
            _use_zoom_world_position: false,
            _zooming_underground: false,
            _max_coord: max_coord,
            _tilt_center_mouse_position: DVec2::new(-1.0, -1.0),
            _tilt_center: DVec2::new(0.0, 0.0),
            _rotate_mouse_position: DVec2::new(-1.0, -1.0),
            _rotate_start_position: DVec3::new(0.0, 0.0, 0.0),
            _strafe_start_position: DVec3::new(0.0, 0.0, 0.0),
            _strafe_mouse_position: DVec2::new(0.0, 0.0),
            _strafe_end_mouse_position: DVec2::new(0.0, 0.0),
            _zoom_mouse_start: DVec2::new(-1.0, -1.0),
            _zoom_world_position: DVec3::new(0.0, 0.0, 0.0),
            minimum_track_ball_height: 7500000.0,
            _minimum_track_ball_height: 7500000.0,
            minimum_collision_terrain_height: 150000.0,
            _minimum_collision_terrain_height: 150000.0,
            minimum_picking_terrain_height: 150000.0,
            _minimum_picking_terrain_height: 150000.0,
            minimum_picking_terrain_distance_with_inertia: 4000.0,
            _tilt_cv_off_map: false,
            _looking: false,
            _rotating: false,
            _strafing: false,
            _adjusted_height_for_terrain: false,
            _tilt_on_ellipsoid: false,
            _maximum_rotate_rate: 1.77,
            _minimum_rotate_rate: 1.0 / 5000.0,
            _minimum_underground_pick_distance: 2000.0,
            _maximum_underground_pick_distance: 10000.0,
            _ellipsoid: ellipsoid,
            _rotate_factor: 1.0,
            _rotate_rate_range_adjustment: 1.0,
            _horizontal_rotation_axis: None,
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
        self._rotate_factor = 1.0 / self._ellipsoid.maximum_radius;
        self._rotate_rate_range_adjustment = self._ellipsoid.maximum_radius;
    }
}
