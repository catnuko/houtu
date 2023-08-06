use super::camera_event_aggregator;

use bevy::math::{DMat3, DMat4, DQuat, DVec2, DVec3, DVec4};
use bevy::prelude::*;

use bevy::render::primitives::Frustum;

use houtu_scene::{
    acos_clamped, equals_epsilon, zero_to_two_pi, BoundingRectangle, Cartesian3, Cartographic,
    CullingVolume, Ellipsoid, EllipsoidGeodesic, GeographicProjection, HeadingPitchRoll,
    IntersectionTests, Matrix3, Matrix4, PerspectiveFrustum, Projection, Quaternion, Rectangle,
    Transforms, EPSILON10, EPSILON2, EPSILON3, EPSILON4, RADIANS_PER_DEGREE,
};
use std::f64::consts::{FRAC_PI_2, PI, TAU};
use std::f64::NEG_INFINITY;

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(camera_event_aggregator::Plugin);
        app.add_system(globe_camera_setup_system);
    }
}

/// Base system set to allow ordering of `GlobeCamera`
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[system_set(base)]
pub struct PanOrbitCameraSystemSet;

#[derive(Component)]
pub struct GlobeCamera {
    pub right: DVec3,
    pub _right: DVec3,
    pub _rightWC: DVec3,

    pub up: DVec3,
    pub _up: DVec3,
    pub _upWC: DVec3,

    pub direction: DVec3,
    pub _direction: DVec3,
    pub _directionWC: DVec3,

    pub _transform: DMat4,
    pub _invTransform: DMat4,
    pub _actualTransform: DMat4,
    pub _actualInvTransform: DMat4,
    pub _transformChanged: bool,

    pub position: DVec3,
    pub _position: DVec3,
    pub _positionWC: DVec3,
    pub _positionCartographic: Cartographic,
    pub _oldPositionWC: Option<DVec3>,

    pub positionWCDeltaMagnitude: f64,
    pub positionWCDeltaMagnitudeLastFrame: f64,
    pub timeSinceMoved: f64,
    pub _lastMovedTimestamp: f64,
    pub defaultLookAmount: f64,
    pub default_rotate_amount: f64,
    pub defaultZoomAmount: f64,
    pub defaultMoveAmount: f64,
    pub maximumZoomFactor: f64,
    pub percentageChanged: f64,
    pub _viewMatrix: DMat4,
    pub _invViewMatrix: DMat4,

    pub hpr: HeadingPitchRoll,
    pub _max_coord: DVec3,

    pub frustum: PerspectiveFrustum,
    pub inited: bool,
    pub constrained_axis: Option<DVec3>,

    pub viewport: BoundingRectangle,
}

impl Default for GlobeCamera {
    fn default() -> Self {
        let max_coord = GeographicProjection::WGS84.project(&Cartographic::new(PI, FRAC_PI_2, 0.));

        let me = Self {
            positionWCDeltaMagnitude: 0.0,
            positionWCDeltaMagnitudeLastFrame: 0.0,
            timeSinceMoved: 0.0,
            _lastMovedTimestamp: 0.0,
            defaultMoveAmount: 100000.0,
            defaultLookAmount: PI / 60.0,
            default_rotate_amount: PI / 3600.0,
            defaultZoomAmount: 100000.0,
            maximumZoomFactor: 1.5,
            percentageChanged: 0.5,
            _viewMatrix: DMat4::ZERO,
            _invViewMatrix: DMat4::ZERO,
            right: DVec3::ZERO,
            _right: DVec3::ZERO,
            _rightWC: DVec3::ZERO,

            up: DVec3::ZERO,
            _up: DVec3::ZERO,
            _upWC: DVec3::ZERO,

            direction: DVec3::ZERO,
            _direction: DVec3::ZERO,
            _directionWC: DVec3::ZERO,

            _transform: DMat4::IDENTITY,
            _invTransform: DMat4::IDENTITY,
            _actualTransform: DMat4::IDENTITY,
            _actualInvTransform: DMat4::IDENTITY,
            _transformChanged: false,

            position: DVec3::ZERO,
            _position: DVec3::ZERO,
            _positionWC: DVec3::ZERO,
            _positionCartographic: Cartographic::ZERO,
            _oldPositionWC: None,
            hpr: HeadingPitchRoll::default(),
            _max_coord: max_coord,
            frustum: PerspectiveFrustum::default(),
            inited: false,
            constrained_axis: None,
            viewport: BoundingRectangle::new(),
        };
        return me;
    }
}
fn globe_camera_setup_system(
    mut query: Query<
        (
            &mut GlobeCamera,
            &mut Transform,
            &Frustum,
            &bevy::prelude::Projection,
            &bevy::prelude::Camera,
        ),
        (With<Camera3d>, Changed<Transform>),
    >,
) {
    for (mut globe_camera, mut transform, _frustum, projection, camera) in &mut query {
        if globe_camera.inited == true {
            return;
        }
        globe_camera.inited = true;
        if let bevy::prelude::Projection::Perspective(projection) = projection {
            let frustum = &mut globe_camera.frustum;
            frustum.far = projection.far as f64;
            frustum.aspect_ratio = projection.aspect_ratio as f64;
            frustum.near = projection.near as f64;
            frustum.fov = projection.fov as f64;
            frustum.update_self();
        }
        globe_camera.position = DVec3::new(
            transform.translation.x as f64,
            transform.translation.x as f64,
            transform.translation.x as f64,
        );
        let rotMat = Mat3::from_quat(transform.rotation);
        let x_axis = rotMat.x_axis;
        let y_axis = rotMat.y_axis;
        globe_camera.direction = DVec3::new(x_axis.x as f64, x_axis.y as f64, x_axis.z as f64);
        globe_camera.up = DVec3::new(y_axis.x as f64, y_axis.y as f64, y_axis.z as f64);
        globe_camera.right = globe_camera.direction.cross(globe_camera.up);

        let window_size = camera
            .logical_viewport_size()
            .expect("logical_viewport_size is undifined");
        globe_camera.viewport.x = 0.;
        globe_camera.viewport.y = 0.;
        globe_camera.viewport.width = window_size.x as f64;
        globe_camera.viewport.height = window_size.y as f64;
        globe_camera.update_self();
        globe_camera.update_members();
        globe_camera.update_camera_matrix(&mut transform);
    }
}
pub enum LookAtTransformOffset {
    Cartesian3(DVec3),
    HeadingPitchRoll(HeadingPitchRoll),
}
pub struct DirectionUp {
    pub direction: DVec3,
    pub up: DVec3,
}
pub enum SetViewOrientation {
    HeadingPitchRoll(HeadingPitchRoll),
    DirectionUp(DirectionUp),
}
impl GlobeCamera {
    pub const DEFAULT_VIEW_RECTANGLE: Rectangle = Rectangle {
        west: -1.6580627893946132,
        south: -0.3490658503988659,
        east: -1.2217304763960306,
        north: 1.5707963267948966,
    };
    pub const DEFAULT_VIEW_FACTOR: f64 = 0.5;

    fn update_self(&mut self) {
        self.update_view_matrix();
        self.position = self
            .rectangle_camera_position_3d(&GlobeCamera::DEFAULT_VIEW_RECTANGLE, Some(true))
            .unwrap();
        self.position = DVec3::new(0., 0., 11347315.0);
        let mut mag = self.position.magnitude();
        mag += mag * Self::DEFAULT_VIEW_FACTOR;
        self.position = self.position.normalize().multiply_by_scalar(mag);
        self.direction = (DVec3::ZERO - self.position).normalize();
        self.constrained_axis = Some(DVec3::UNIT_Z);
    }
    pub fn get_position_wc(&mut self) -> DVec3 {
        self.update_members();
        return self._positionWC;
    }
    pub fn get_position_cartographic(&mut self) -> Cartographic {
        self.update_members();
        return self._positionCartographic;
    }
    pub fn get_direction_wc(&mut self) -> DVec3 {
        self.update_members();
        return self._directionWC;
    }

    pub fn get_up_wc(&mut self) -> DVec3 {
        self.update_members();
        return self._upWC;
    }

    pub fn get_right_wc(&mut self) -> DVec3 {
        self.update_members();
        return self._rightWC;
    }
    pub fn get_transform(&self) -> DMat4 {
        return self._transform;
    }
    pub fn get_inverse_transform(&mut self) -> DMat4 {
        self.update_members();
        return self._invTransform;
    }
    pub fn get_view_matrix(&mut self) -> DMat4 {
        self.update_members();
        return self._viewMatrix;
    }
    pub fn get_inverse_view_matrix(&mut self) -> DMat4 {
        self.update_members();
        return self._invViewMatrix;
    }
    pub fn get_heading(&mut self) -> f64 {
        let ellipsoid = Ellipsoid::WGS84;

        let old_transform = self._transform.clone();
        let transform =
            Transforms::eastNorthUpToFixedFrame(&self.get_position_wc(), Some(ellipsoid));
        self._setTransform(&transform);

        let heading = getHeading(&self.direction, &self.up);

        self._setTransform(&old_transform);

        return heading;
    }
    pub fn get_pitch(&mut self) -> f64 {
        let ellipsoid = Ellipsoid::WGS84;

        let old_transform = self._transform.clone();
        let transform =
            Transforms::eastNorthUpToFixedFrame(&self.get_position_wc(), Some(ellipsoid));
        self._setTransform(&transform);

        let pitch = getPitch(&self.direction);

        self._setTransform(&old_transform);

        return pitch;
    }
    pub fn get_culling_volume(&mut self) -> &CullingVolume {
        let p = self.get_position_wc();
        let d = self.get_direction_wc();
        let u = self.get_up_wc();

        return &self.frustum.computeCullingVolume(&p, &d, &u);
    }
    pub fn get_roll(&mut self) -> f64 {
        let ellipsoid = Ellipsoid::WGS84;

        let old_transform = self._transform.clone();
        let transform =
            Transforms::eastNorthUpToFixedFrame(&self.get_position_wc(), Some(ellipsoid));
        self._setTransform(&transform);

        let roll = getRoll(&self.direction, &self.up, &self.right);

        self._setTransform(&old_transform);

        return roll;
    }

    pub fn update_view_matrix(&mut self) {
        self._viewMatrix =
            DMat4::compute_view(&self.position, &self.direction, &self.up, &self.right);
        self._viewMatrix = self._viewMatrix * self._actualInvTransform;
        self._invViewMatrix = self._viewMatrix.inverse_transformation();
    }
    pub fn update_camera_matrix(&mut self, transform: &mut Transform) {
        *transform = Transform::from_matrix(self.get_inverse_view_matrix().to_mat4_32());
    }
    pub fn getRectangleCameraCoordinates(&mut self, rectangle: &Rectangle) -> Option<DVec3> {
        return self.rectangle_camera_position_3d(rectangle, None);
    }

    pub fn look_at_transform(&mut self, transform: &DMat4, offset: Option<LookAtTransformOffset>) {
        self._setTransform(transform);
        let offset = if let Some(v) = offset {
            v
        } else {
            return;
        };
        let cartesianOffset = match offset {
            LookAtTransformOffset::Cartesian3(v) => v,
            LookAtTransformOffset::HeadingPitchRoll(v) => {
                offsetFromHeadingPitchRange(v.heading, v.pitch, v.roll)
            }
        };

        self.position = cartesianOffset.clone();
        self.direction = self.position.negate().normalize();

        self.right = self.direction.cross(DVec3::UNIT_Z);

        if self.right.magnitude_squared() < EPSILON10 {
            self.right = DVec3::UNIT_X.clone();
        }

        self.right = self.right.normalize();
        self.up = self.right.cross(self.direction).normalize();
    }
    pub fn look(&mut self, axis: &DVec3, angle: Option<f64>) {
        let turn_angle = angle.unwrap_or(self.defaultLookAmount);
        let quaternion = DQuat::from_axis_angle(axis.clone(), -turn_angle);
        let rotation = DMat3::from_quaternion(&quaternion);

        let direction = self.direction;
        let up = self.up;
        let right = self.right;
        self.direction = rotation.multiply_by_vector(&direction);
        self.up = rotation.multiply_by_vector(&up);
        self.right = rotation.multiply_by_vector(&right);
    }
    pub fn look_up(&mut self, amount: Option<f64>) {
        let amount = amount.unwrap_or(self.defaultLookAmount);
        self.look(&self.right.clone(), Some(-amount));
    }
    pub fn look_down(&mut self, amount: Option<f64>) {
        let amount = amount.unwrap_or(self.defaultLookAmount);
        self.look(&self.right.clone(), Some(amount));
    }
    pub fn look_left(&mut self, amount: Option<f64>) {
        let amount = amount.unwrap_or(self.defaultLookAmount);
        self.look(&self.up.clone(), Some(-amount));
    }
    pub fn look_right(&mut self, amount: Option<f64>) {
        let amount = amount.unwrap_or(self.defaultLookAmount);
        self.look(&self.up.clone(), Some(amount));
    }
    pub fn look_at(&mut self, target: &DVec3, offset: LookAtTransformOffset) {
        let transform = Transforms::eastNorthUpToFixedFrame(target, None);
        self.look_at_transform(&transform, Some(offset));
    }
    pub fn set_view(
        &mut self,
        destination: Option<DVec3>,
        orientation: Option<SetViewOrientation>,
        end_transform: Option<DMat4>,
        convert: Option<bool>,
    ) {
        if end_transform.is_some() {
            self._setTransform(&end_transform.unwrap());
        }
        let _convert = convert.unwrap_or(true);
        let destination = destination.unwrap_or(self.get_position_wc().clone());
        let hpr = if let Some(orientation) = orientation {
            match orientation {
                SetViewOrientation::DirectionUp(v) => {
                    self.direction_up_to_heading_pitch_roll(&v.direction, &v)
                }
                SetViewOrientation::HeadingPitchRoll(z) => z,
            }
        } else {
            HeadingPitchRoll::new(0.0, -FRAC_PI_2, 0.0)
        };
        self.set_view_3d(&destination, &hpr);
    }
    fn set_view_3d(&mut self, position: &DVec3, hpr: &HeadingPitchRoll) {
        let current_transform = self.get_transform().clone();
        let local_transform = Transforms::eastNorthUpToFixedFrame(&position, None);
        self._setTransform(&local_transform);
        let mut hpr = hpr.clone();

        self.position = DVec3::ZERO.clone();
        hpr.heading = hpr.heading - FRAC_PI_2;

        let rot_quat = DQuat::from_heading_pitch_roll(&hpr);
        let rotMat = DMat3::from_quaternion(&rot_quat);

        self.direction = rotMat.col(0);
        self.up = rotMat.col(2);
        self.right = self.direction.cross(self.up);

        self._setTransform(&current_transform);
    }
    pub fn world_to_camera_coordinates(&mut self, cartesian: &DVec3) -> DVec3 {
        self.update_members();
        let vv4 = DVec4::new(cartesian.x, cartesian.y, cartesian.z, 1.);
        let v4 = self._actualInvTransform.multiply_by_vector(&vv4);
        return DVec3::new(v4.x, v4.y, v4.z);
    }
    pub fn camera_to_world_coordinates(&mut self, cartesian: &DVec3) -> DVec3 {
        self.update_members();
        let vv4 = DVec4::new(cartesian.x, cartesian.y, cartesian.z, 1.);
        let v4 = self._actualTransform.multiply_by_vector(&vv4);
        return DVec3::new(v4.x, v4.y, v4.z);
    }
    fn direction_up_to_heading_pitch_roll(
        &mut self,
        position: &DVec3,
        orientation: &DirectionUp,
    ) -> HeadingPitchRoll {
        let mut direction = orientation.direction.clone();
        let mut up = orientation.up.clone();

        let ellipsoid = Ellipsoid::WGS84;
        let transform = Transforms::eastNorthUpToFixedFrame(&position, Some(ellipsoid));
        let invTransform = transform.inverse_transformation();

        direction = invTransform.multiply_by_point_as_vector(&direction);
        up = invTransform.multiply_by_point_as_vector(&up);

        let right = direction.cross(up);
        let mut result = HeadingPitchRoll::default();
        result.heading = getHeading(&direction, &up);
        result.pitch = getPitch(&direction);
        result.roll = getRoll(&direction, &up, &right);

        return result;
    }
    pub fn _setTransform(&mut self, transform: &DMat4) {
        let position = self.get_position_wc().clone();
        let up = self.get_up_wc().clone();
        let direction = self.get_direction_wc().clone();

        self._transform = transform.clone();
        self._transformChanged = true;
        self.update_members();
        let inverse = self._actualInvTransform;

        self.position = inverse.multiply_by_point(&position);
        self.direction = inverse.multiply_by_point_as_vector(&direction);
        self.up = inverse.multiply_by_point_as_vector(&up);
        self.right = self.direction.cross(self.up);

        self.update_members();
    }
    fn update_members(&mut self) {
        // let mode = self._mode;

        let height_changed = false;
        let _height = 0.0;

        let positionChanged = !self._position.equals(self.position) || height_changed;
        if positionChanged {
            self._position = self.position.clone();
        }

        let directionChanged = !self._direction.equals(self.direction);
        if directionChanged {
            self.direction = self.direction.normalize();
            self._direction = self.direction.clone();
        }

        let upChanged = !self._up.equals(self.up);
        if upChanged {
            self.up = self.up.normalize();
            self._up = self.up.clone();
        }

        let rightChanged = !self._right.equals(self.right);
        if rightChanged {
            self.right = self.right.normalize();
            self._right = self.right.clone();
        }

        let transformChanged = self._transformChanged;
        self._transformChanged = false;

        if transformChanged {
            self._invTransform = self._transform.inverse_transformation();

            self._actualTransform = self._transform.clone();

            self._actualInvTransform = self._actualTransform.inverse_transformation();
        }

        let transform = self._actualTransform;

        if positionChanged || transformChanged {
            self._positionWC = transform.multiply_by_point(&self._position);

            // Compute the Cartographic position of the self.

            self._positionCartographic = Ellipsoid::WGS84
                .cartesianToCartographic(&self._positionWC)
                .unwrap_or(Cartographic::default());
        }

        if directionChanged || upChanged || rightChanged {
            let det = self._direction.dot(self._up.cross(self._right));
            if (1.0 - det).abs() > EPSILON2 {
                //orthonormalize axes
                let invUpMag = 1.0 / self._up.magnitude_squared();
                let scalar = self._up.dot(self._direction) * invUpMag;
                let w0 = self._direction.multiply_by_scalar(scalar);
                self._up = self._up.subtract(w0).normalize();
                self.up = self._up.clone();

                self._right = self._direction.cross(self._up);
                self.right = self._right.clone();
            }
        }

        if directionChanged || transformChanged {
            self._directionWC = transform
                .multiply_by_point_as_vector(&self._direction)
                .normalize();
        }

        if upChanged || transformChanged {
            self._upWC = transform.multiply_by_point_as_vector(&self._up).normalize();
        }

        if rightChanged || transformChanged {
            self._rightWC = transform
                .multiply_by_point_as_vector(&self._right)
                .normalize();
        }

        if positionChanged || directionChanged || upChanged || rightChanged || transformChanged {
            self.update_view_matrix();
        }
    }
    pub fn zoom_in(&mut self, amount: Option<f64>) {
        let amount = amount.unwrap_or(self.defaultZoomAmount);
        self.move_direction(&self.direction.clone(), amount);
    }
    pub fn zoom_out(&mut self, amount: Option<f64>) {
        let amount = amount.unwrap_or(self.defaultZoomAmount);
        self.move_direction(&self.direction.clone(), -amount);
    }
    pub fn move_direction(&mut self, direction: &DVec3, amount: f64) {
        let moveScratch = direction.multiply_by_scalar(amount);
        self.position = self.position + moveScratch;
    }
    pub fn rotate_horizotal(&mut self, angle: f64) {
        if self.constrained_axis.is_some() {
            self.rotate(self.constrained_axis.unwrap(), Some(angle));
        } else {
            self.rotate(self.up, Some(angle));
        }
    }
    pub fn rotate_vertical(&mut self, angle: f64) {
        let mut angle = angle;
        let position = self.position;
        if self.constrained_axis.is_some()
            && !self
                .position
                .equals_epsilon(DVec3::ZERO, Some(EPSILON2), None)
        {
            let constrained_axis = self.constrained_axis.unwrap();
            let p = position.normalize();
            let north_parallel = p.equals_epsilon(constrained_axis, Some(EPSILON2), None);
            let south_parallel = p.equals_epsilon(constrained_axis.negate(), Some(EPSILON2), None);
            if !north_parallel && !south_parallel {
                let constrained_axis = constrained_axis.normalize();

                let mut dot = p.dot(constrained_axis);
                let mut angle_to_axis = acos_clamped(dot);
                if angle > 0. && angle > angle_to_axis {
                    angle = angle_to_axis - EPSILON4;
                }

                dot = p.dot(constrained_axis.negate());
                angle_to_axis = acos_clamped(dot);
                if angle < 0. && -angle > angle_to_axis {
                    angle = -angle_to_axis + EPSILON4;
                }

                let tangent = constrained_axis.cross(p);
                self.rotate(tangent, Some(angle));
            } else if (north_parallel && angle < 0.) || (south_parallel && angle > 0.) {
                self.rotate(self.right, Some(angle));
            }
        } else {
            self.rotate(self.right, Some(angle));
        }
    }

    pub fn rotate_right(&mut self, angle: Option<f64>) {
        let angle = angle.unwrap_or(self.default_rotate_amount);
        self.rotate_horizotal(-angle);
    }
    pub fn rotate_left(&mut self, angle: Option<f64>) {
        let angle = angle.unwrap_or(self.default_rotate_amount);
        self.rotate_horizotal(angle);
    }
    pub fn rotate_up(&mut self, angle: Option<f64>) {
        let angle = angle.unwrap_or(self.default_rotate_amount);
        self.rotate_vertical(-angle);
    }
    pub fn rotate_down(&mut self, angle: Option<f64>) {
        let angle = angle.unwrap_or(self.default_rotate_amount);
        self.rotate_vertical(angle);
    }
    pub fn rotate(&mut self, axis: DVec3, angle: Option<f64>) {
        let turn_angle = angle.unwrap_or(self.default_rotate_amount);
        let quaternion = DQuat::from_axis_angle(axis, -turn_angle);
        let rotation = DMat3::from_quat(quaternion);
        self.position = rotation.multiply_by_vector(&self.position);
        self.direction = rotation.multiply_by_vector(&self.direction);
        self.up = rotation.multiply_by_vector(&self.up);
        self.right = self.direction.cross(self.up);
        self.up = self.right.cross(self.direction);
    }
    pub fn getPickRay(&mut self, window_position: &DVec2, window_size: &DVec2) -> houtu_scene::Ray {
        return self.get_pick_ray_rerspective(window_position, window_size);
    }
    pub fn pick_ellipsoid(
        &mut self,
        window_position: &DVec2,
        window_size: &DVec2,
    ) -> Option<DVec3> {
        return self.pick_ellipsoid_3d(window_position, window_size);
    }
    pub fn pick_ellipsoid_3d(
        &mut self,
        window_position: &DVec2,
        window_size: &DVec2,
    ) -> Option<DVec3> {
        let _ellipsoid = Ellipsoid::WGS84;
        let ray = self.getPickRay(window_position, window_size);
        let intersection = IntersectionTests::rayEllipsoid(&ray, None);
        let intersection = if let Some(v) = intersection {
            v
        } else {
            return None;
        };
        let t = if intersection.start > 0.0 {
            intersection.start
        } else {
            intersection.stop
        };
        return Some(ray.getPoint(t));
    }
    pub fn get_pick_ray_rerspective(
        &mut self,
        window_position: &DVec2,
        window_size: &DVec2,
    ) -> houtu_scene::Ray {
        let mut result = houtu_scene::Ray::default();
        let width = window_size.x as f64;
        let height = window_size.y as f64;
        let aspect_ratio = width / height;
        let tanPhi = (self.frustum.get_fovy() as f64 * 0.5).tan();
        let tanTheta = aspect_ratio * tanPhi;
        let near = self.frustum.near as f64;

        let x = (2.0 / width) * window_position.x as f64 - 1.0;
        let y = (2.0 / height) * (height - window_position.y as f64) - 1.0;

        let position = self.get_position_wc();
        result.origin = position.clone();

        let mut near_center = self.get_direction_wc().multiply_by_scalar(near);
        near_center = position + near_center;
        let xDir = self.get_right_wc().multiply_by_scalar(x * near * tanTheta);
        let yDir = self.get_up_wc().multiply_by_scalar(y * near * tanPhi);
        result.direction = (near_center + xDir + yDir - position).normalize();
        return result;
    }
    pub fn rectangle_camera_position_3d(
        &mut self,
        rectangle: &Rectangle,
        update_camera: Option<bool>,
    ) -> Option<DVec3> {
        let ellipsoid = Ellipsoid::WGS84;
        let update_camera = update_camera.unwrap_or(false);
        let north = rectangle.north;
        let south = rectangle.south;
        let mut east = rectangle.east;
        let west = rectangle.west;

        // If we go across the International Date Line
        if west > east {
            east += TAU;
        }

        // Find the midpoint latitude.
        //
        // EllipsoidGeodesic will fail if the north and south edges are very close to being on opposite sides of the ellipsoid.
        // Ideally we'd just call EllipsoidGeodesic.setEndPoints and let it throw when it detects this case, but sadly it doesn't
        // even look for this case in optimized builds, so we have to test for it here instead.
        //
        // Fortunately, this case can only happen (here) when north is very close to the north pole and south is very close to the south pole,
        // so handle it just by using 0 latitude as the center.  It's certainliy possible to use a smaller tolerance
        // than one degree here, but one degree is safe and putting the center at 0 latitude should be good enough for any
        // rectangle that spans 178+ of the 180 degrees of latitude.
        let longitude = (west + east) * 0.5;
        let latitude;
        if south < -FRAC_PI_2 + RADIANS_PER_DEGREE && north > FRAC_PI_2 - RADIANS_PER_DEGREE {
            latitude = 0.0;
        } else {
            let north_cartographic = Cartographic::from_radians(longitude, north, 0.);
            let south_cartographic = Cartographic::from_radians(longitude, south, 0.);
            let mut ellipsoid_geodesic = EllipsoidGeodesic::default();
            ellipsoid_geodesic.setEndPoints(north_cartographic, south_cartographic);
            latitude = ellipsoid_geodesic.interpolateUsingFraction(0.5).latitude;
        }

        let center_cartographic = Cartographic::from_radians(longitude, latitude, 0.0);

        let center = ellipsoid.cartographicToCartesian(&center_cartographic);

        let mut cart = Cartographic::default();
        cart.longitude = east;
        cart.latitude = north;
        let mut north_east = ellipsoid.cartographicToCartesian(&cart);
        cart.longitude = west;
        let mut north_west = ellipsoid.cartographicToCartesian(&cart);
        cart.longitude = longitude;
        let mut north_center = ellipsoid.cartographicToCartesian(&cart);
        cart.latitude = south;
        let mut south_center = ellipsoid.cartographicToCartesian(&cart);
        cart.longitude = east;
        let mut south_east = ellipsoid.cartographicToCartesian(&cart);
        cart.longitude = west;
        let mut south_west = ellipsoid.cartographicToCartesian(&cart);

        north_west = north_west.subtract(center);
        south_east = south_east.subtract(center);
        north_east = north_east.subtract(center);
        south_west = south_west.subtract(center);
        north_center = north_center.subtract(center);
        south_center = south_center.subtract(center);

        let direction = ellipsoid.geodeticSurfaceNormal(&center);
        let mut direction = if let Some(v) = direction {
            v
        } else {
            return None;
        };
        direction = direction.negate();
        let right = direction.cross(DVec3::UNIT_Z).normalize();
        let up = right.cross(direction);
        if update_camera {
            self.direction = direction.clone();
            self.up = up.clone();
            self.right = right.clone();
        }
        let mut d;
        let tanPhi = (self.frustum.get_fovy() * 0.5).tan(); //aspectradio:0.660105980317941
        let tanTheta = self.frustum.aspect_ratio * tanPhi;

        d = [
            computeD(&direction, &up, &north_west, tanPhi),
            computeD(&direction, &up, &south_east, tanPhi),
            computeD(&direction, &up, &north_east, tanPhi),
            computeD(&direction, &up, &south_west, tanPhi),
            computeD(&direction, &up, &north_center, tanPhi),
            computeD(&direction, &up, &south_center, tanPhi),
            computeD(&direction, &right, &north_west, tanTheta),
            computeD(&direction, &right, &south_east, tanTheta),
            computeD(&direction, &right, &north_east, tanTheta),
            computeD(&direction, &right, &south_west, tanTheta),
            computeD(&direction, &right, &north_center, tanTheta),
            computeD(&direction, &right, &south_center, tanTheta),
        ]
        .iter()
        .fold(NEG_INFINITY, |a, &b| a.max(b));

        // If the rectangle crosses the equator, compute D at the equator, too, because that's the
        // widest part of the rectangle when projected onto the globe.
        if south < 0. && north > 0. {
            let mut equatorCartographic = Cartographic::from_radians(west, 0.0, 0.0);
            let mut equatorPosition = ellipsoid.cartographicToCartesian(&equatorCartographic);
            equatorPosition = equatorPosition.subtract(center);
            d = [
                d,
                computeD(&direction, &up, &equatorPosition, tanPhi),
                computeD(&direction, &right, &equatorPosition, tanTheta),
            ]
            .iter()
            .fold(NEG_INFINITY, |a, &b| a.max(b));

            equatorCartographic.longitude = east;
            equatorPosition = ellipsoid.cartographicToCartesian(&equatorCartographic);
            equatorPosition = equatorPosition.subtract(center);
            d = [
                d,
                computeD(&direction, &up, &equatorPosition, tanPhi),
                computeD(&direction, &right, &equatorPosition, tanTheta),
            ]
            .iter()
            .fold(NEG_INFINITY, |a, &b| a.max(b));
        }
        return Some(center + direction.multiply_by_scalar(-d));
    }
}
fn computeD(direction: &DVec3, upOrRight: &DVec3, corner: &DVec3, tanThetaOrPhi: f64) -> f64 {
    let opposite = upOrRight.dot(*corner).abs();
    return opposite / tanThetaOrPhi - direction.dot(*corner);
}
fn getHeading(direction: &DVec3, up: &DVec3) -> f64 {
    let heading;
    if !equals_epsilon(direction.z.abs(), 1.0, Some(EPSILON3), None) {
        heading = direction.y.atan2(direction.x) - FRAC_PI_2;
    } else {
        heading = up.y.atan2(up.x) - FRAC_PI_2;
    }

    return TAU - zero_to_two_pi(heading);
}

fn getPitch(direction: &DVec3) -> f64 {
    return FRAC_PI_2 - acos_clamped(direction.z);
}

fn getRoll(direction: &DVec3, up: &DVec3, right: &DVec3) -> f64 {
    let mut roll = 0.0;
    if !equals_epsilon(direction.z.abs(), 1.0, Some(EPSILON3), None) {
        roll = (-right.z).atan2(up.z);
        roll = zero_to_two_pi(roll + TAU);
    }

    return roll;
}
fn offsetFromHeadingPitchRange(heading: f64, pitch: f64, range: f64) -> DVec3 {
    let pitch = pitch.clamp(-FRAC_PI_2, FRAC_PI_2);
    let heading = zero_to_two_pi(heading) - FRAC_PI_2;

    let pitch_quat = DQuat::from_axis_angle(DVec3::UNIT_Y, -pitch);
    let heading_quat = DQuat::from_axis_angle(DVec3::UNIT_Z, -heading);
    let rot_quat = heading_quat.mul_quat(pitch_quat);
    let rot_matrix = DMat3::from_quaternion(&rot_quat);

    let mut offset = DVec3::UNIT_X.clone();
    offset = rot_matrix.multiply_by_vector(&offset);
    offset = offset.negate();
    offset = offset.multiply_by_scalar(range);
    return offset;
}

#[cfg(test)]
mod tests {
    use std::f64::consts::FRAC_PI_4;

    use bevy::math::DVec2;
    use houtu_scene::{EPSILON10, EPSILON11, EPSILON14, EPSILON15, EPSILON6};

    use super::*;
    const client_width: f64 = 512.;
    const client_height: f64 = 384.;
    const drawing_buffer_width: f64 = 1024.;
    const drawing_buffer_height: f64 = 768.;

    const move_amount: f64 = 3.0;
    const turn_amount: f64 = FRAC_PI_2;
    const rotate_amount: f64 = FRAC_PI_2;
    const zoom_amount: f64 = 1.0;

    const position: DVec3 = DVec3::UNIT_Z;
    const up: DVec3 = DVec3::UNIT_Y;
    const direction: DVec3 = DVec3 {
        x: 0.,
        y: 0.,
        z: -1.0,
    };
    const right: DVec3 = DVec3::UNIT_X;

    #[test]
    fn pick_ray_perspective() {
        let mut camera = GlobeCamera::default();
        camera.update_self();
        camera.position = DVec3::UNIT_Z;
        camera.up = DVec3::UNIT_Y;
        camera.direction = DVec3::UNIT_Z.negate();
        camera.right = camera.direction.cross(camera.up);
        camera.frustum.aspect_ratio = (client_width / client_height) as f64;
        camera.frustum.update_self();
        // camera.update_members();
        let window_coord = DVec2::new(client_width as f64 / 2.0, client_height as f64);
        let ray = camera.getPickRay(
            &window_coord,
            &DVec2::new(client_width as f64, client_height as f64),
        );
        let window_height = camera.frustum.near * (camera.frustum.get_fovy() * 0.5).tan();
        let expected_direction = DVec3::new(0.0, -window_height, -1.0).normalize();
        assert!(ray.origin.equals(camera.position));
        assert!(ray
            .direction
            .equals_epsilon(expected_direction, Some(EPSILON15), None));
    }
    #[test]
    fn getRectangleCameraCoordinates() {
        let mut camera = get_camera();
        camera.frustum.update_self();
        let rectangle = Rectangle::new(-PI, -FRAC_PI_2, PI, FRAC_PI_2);
        let mut position_1 = camera.position.clone();
        let direction_1 = camera.direction.clone();
        let up_1 = camera.up.clone();
        let right_1 = camera.right.clone();
        position_1 = camera.getRectangleCameraCoordinates(&rectangle).unwrap();
        assert!(position_1.equals_epsilon(
            DVec3::new(14680290.639204923, 0.0, 0.0),
            Some(EPSILON6),
            None
        ));
        assert!(camera.direction.equals(direction_1));
        assert!(camera.up.equals(up_1));
        assert!(camera.right.equals(right_1));
    }
    #[test]
    fn move_test() {
        let mut camera = get_camera();
        let dir = DVec3::new(1.0, 1.0, 0.0).normalize();
        camera.move_direction(&dir, move_amount);
        assert!(camera.position.equals_epsilon(
            DVec3::new(dir.x * move_amount, dir.y * move_amount, 1.0),
            Some(EPSILON10),
            None
        ));
        assert!(camera.up.equals(up));
        assert!(camera.direction.equals(direction));
        assert!(camera.right.equals(right));
    }
    #[test]
    fn zoom_in() {
        let mut camera = get_camera();
        camera.zoom_in(Some(zoom_amount));
        assert!(camera.position.equals_epsilon(
            DVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0 - zoom_amount
            },
            Some(EPSILON10),
            None
        ));
        assert!(camera.up == up);
        assert!(camera.direction == direction);
        assert!(camera.right == right);
    }
    #[test]
    fn zoom_out() {
        let mut camera = get_camera();
        camera.zoom_out(Some(zoom_amount));
        assert!(camera.position.equals_epsilon(
            DVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0 + zoom_amount
            },
            Some(EPSILON10),
            None
        ));
        assert!(camera.up == up);
        assert!(camera.direction == direction);
        assert!(camera.right == right);
    }
    #[test]
    fn set_heading() {
        let mut camera = get_camera();
        camera.position = DVec3::UNIT_X;
        camera.direction = DVec3::UNIT_X.negate();
        camera.up = DVec3::UNIT_Z;
        camera.right = camera.direction.cross(camera.up);
        let heading = camera.get_heading();
        let new_heading = (45.0 as f64).to_radians();
        camera.set_view(
            None,
            Some(SetViewOrientation::HeadingPitchRoll(HeadingPitchRoll {
                heading: new_heading,
                pitch: 0.0,
                roll: 0.0,
            })),
            None,
            None,
        );
        assert!(camera.get_heading() != heading);
        assert!(equals_epsilon(
            camera.get_heading(),
            new_heading,
            Some(EPSILON14),
            None
        ));
    }
    fn get_camera() -> GlobeCamera {
        let mut camera = GlobeCamera::default();
        camera.position = DVec3::UNIT_Z;
        camera.up = DVec3::UNIT_Y;
        camera.direction = DVec3::UNIT_Z.negate();
        camera.right = camera.direction.cross(camera.up);
        camera.frustum.aspect_ratio = (client_width / client_height) as f64;
        // camera.update_self();
        return camera;
    }
    #[test]
    fn set_transform() {
        let mut camera = get_camera();
        camera._setTransform(&DMat4::from_cols(
            [5.0, 0.0, 0.0, 0.0].into(),
            [0.0, 5.0, 0.0, 0.0].into(),
            [0.0, 0.0, 5.0, 0.0].into(),
            [1.0, 2.0, 3.0, 1.0].into(),
        ));
        assert!(camera.get_transform() == camera.get_inverse_transform());
    }
    #[test]
    fn get_inverse_view_matrix() {
        let mut camera = get_camera();
        assert!(camera.get_view_matrix().inverse() == camera.get_inverse_view_matrix());
    }
    #[test]
    fn set_view_right_rotation_order() {
        let mut camera = get_camera();

        let position1 = DVec3::from_degrees(-117.16, 32.71, Some(0.0), None);
        let heading = (180.0 as f64).to_radians();
        let pitch = (0.0 as f64).to_radians();
        let roll = (45.0 as f64).to_radians();
        camera.set_view(
            Some(position1),
            Some(SetViewOrientation::HeadingPitchRoll(HeadingPitchRoll::new(
                heading, pitch, roll,
            ))),
            None,
            None,
        );
        assert!(camera
            .position
            .equals_epsilon(position1, Some(EPSILON6), None));
        assert!(equals_epsilon(
            camera.get_heading(),
            heading,
            Some(EPSILON6),
            None
        ));
        assert!(equals_epsilon(
            camera.get_pitch(),
            pitch,
            Some(EPSILON6),
            None
        ));
        assert!(equals_epsilon(
            camera.get_roll(),
            roll,
            Some(EPSILON6),
            None
        ));
    }
    #[test]
    fn word_to_camera_coordinates_transforms_to_the_cameras_reference_frame() {
        let mut camera = get_camera();
        camera._setTransform(&make_matrix4_from_row([
            0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]));
        assert!(
            camera.world_to_camera_coordinates(&DVec3::new(1.0, 0.0, 0.0))
                == DVec3::new(0.0, 0.0, 1.0,)
        )
    }
    #[test]
    fn camera_to_world_coordinates_transforms_from_the_cameras_reference_frame() {
        let mut camera = get_camera();
        camera._setTransform(&make_matrix4_from_row([
            0.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0,
        ]));
        assert!(
            camera.camera_to_world_coordinates(&DVec3::new(0.0, 0.0, 1.0,))
                == DVec3::new(1.0, 0.0, 0.0,)
        )
    }
    #[test]
    fn get_inverse_transform() {
        let mut camera = get_camera();
        camera._setTransform(&make_matrix4_from_row([
            5.0, 0.0, 0.0, 1.0, 0.0, 5.0, 0.0, 2.0, 0.0, 0.0, 5.0, 3.0, 0.0, 0.0, 0.0, 1.0,
        ]));
        let a = camera.get_transform().inverse_transformation();
        let b = camera.get_inverse_transform();
        assert!(a == b);
    }
    #[test]
    fn mat3_from_quat() {
        let sPiOver4 = FRAC_PI_4.sin();
        let cPiOver4 = FRAC_PI_4.cos();
        let sPiOver2 = FRAC_PI_2.sin();
        let cPiOver2 = FRAC_PI_2.cos();

        let tmp = DVec3::new(0.0, 0.0, 1.0).multiply_by_scalar(sPiOver4);
        let quaternion = DQuat {
            x: tmp.x,
            y: tmp.y,
            z: tmp.z,
            w: cPiOver4,
        };
        let expected = make_matrix3_from_row([
            cPiOver2, -sPiOver2, 0.0, sPiOver2, cPiOver2, 0.0, 0.0, 0.0, 1.0,
        ]);

        let returnedResult = DMat3::from_quat(quaternion);
        assert!(returnedResult.equals_epsilon(&expected, EPSILON15));
    }
    #[test]
    fn pick_ellipsoid() {
        let mut camera = get_camera();
        let ellipsoid = Ellipsoid::WGS84;
        let maxRadii = ellipsoid.maximum_radius;

        camera.position = DVec3::UNIT_X.multiply_by_scalar(2.0 * maxRadii);
        camera.direction = camera.position.negate().normalize();
        camera.up = DVec3::UNIT_Z.clone();
        camera.right = camera.direction.cross(camera.up);

        let frustum = &mut camera.frustum;
        frustum.fov = (60.0 as f64).to_radians();
        frustum.aspect_ratio = (drawing_buffer_width / drawing_buffer_height) as f64;
        frustum.near = 100.;
        frustum.far = 60.0 * maxRadii;

        let window_coord = DVec2::new((drawing_buffer_width) * 0.5, (drawing_buffer_height) * 0.5);

        let windowSize = DVec2::new((drawing_buffer_width), (drawing_buffer_height));

        let p = camera.pick_ellipsoid(&window_coord, &windowSize).unwrap();
        let c = ellipsoid.cartesianToCartographic(&p).unwrap();
        assert!(c == Cartographic::new(0.0, 0.0, 0.0));
        let q = camera.pick_ellipsoid(&DVec2::ZERO, &windowSize);
        assert!(q.is_none())
    }
    #[test]
    fn look_at() {
        let mut camera = get_camera();
        let target = DVec3::from_degrees(0.0, 0.0, None, None);
        let offset = DVec3::new(0.0, -1.0, 0.0);
        camera.look_at(&target, LookAtTransformOffset::Cartesian3(offset));
        assert!(camera
            .position
            .equals_epsilon(offset, Some(EPSILON11), None));
        assert!(camera.direction.equals_epsilon(
            offset.normalize().negate(),
            Some(EPSILON11),
            None
        ));
        assert!(camera.right.equals_epsilon(
            camera.direction.cross(DVec3::UNIT_Z),
            Some(EPSILON11),
            None
        ));
        assert!(camera.up.equals_epsilon(
            camera.right.cross(camera.direction),
            Some(EPSILON11),
            None
        ));
        assert!(1.0 - camera.direction.magnitude() < EPSILON14);
        assert!(1.0 - camera.up.magnitude() < EPSILON14);
        assert!(1.0 - camera.right.magnitude() < EPSILON14);
    }
    #[test]
    fn look_at_when_target_is_zero() {
        let mut camera = get_camera();
        let target = DVec3::ZERO;
        let offset = DVec3::new(0.0, -1.0, 0.0);
        camera.look_at(&target, LookAtTransformOffset::Cartesian3(offset));
        assert!(camera
            .position
            .equals_epsilon(offset, Some(EPSILON11), None));
        assert!(camera.direction.equals_epsilon(
            offset.normalize().negate(),
            Some(EPSILON11),
            None
        ));
        assert!(camera.right.equals_epsilon(
            camera.direction.cross(DVec3::UNIT_Z),
            Some(EPSILON11),
            None
        ));
        assert!(camera.up.equals_epsilon(
            camera.right.cross(camera.direction),
            Some(EPSILON11),
            None
        ));
    }
    #[test]
    fn look_at_when_target_and_offset_are_zero() {
        let mut camera = get_camera();
        let target = DVec3::ZERO;
        let offset = DVec3::new(0.0, -1.0, 0.0);
        camera.position = DVec3::ZERO;

        camera.look_at(&target, LookAtTransformOffset::Cartesian3(offset));
        assert!(camera
            .position
            .equals_epsilon(offset, Some(EPSILON11), None));
        assert!(camera.direction.equals_epsilon(
            offset.normalize().negate(),
            Some(EPSILON11),
            None
        ));
        assert!(camera.right.equals_epsilon(
            camera.direction.cross(DVec3::UNIT_Z),
            Some(EPSILON11),
            None
        ));
        assert!(camera.up.equals_epsilon(
            camera.right.cross(camera.direction),
            Some(EPSILON11),
            None
        ));
    }
    #[test]
    fn look_at_with_heading_pitch_roll() {
        let mut camera = get_camera();
        let target = DVec3::from_degrees(0.0, 0.0, None, None);
        let heading = (45.0 as f64).to_radians();
        let pitch = (-45.0 as f64).to_radians();
        let range = 2.0;
        camera.look_at(
            &target,
            LookAtTransformOffset::HeadingPitchRoll(HeadingPitchRoll::new(heading, pitch, range)),
        );
        camera.look_at_transform(&DMat4::IDENTITY, None);
        assert!(equals_epsilon(
            camera.position.distance(target),
            range,
            Some(EPSILON6),
            None
        ));

        assert!(equals_epsilon(
            camera.get_pitch(),
            pitch,
            Some(EPSILON6),
            None
        ));
        assert!(1.0 - camera.direction.magnitude() < EPSILON4);
        assert!(1.0 - camera.up.magnitude() < EPSILON4);
        assert!(1.0 - camera.right.magnitude() < EPSILON4);
    }
}
fn make_matrix4_from_row(slice: [f64; 16]) -> DMat4 {
    DMat4 {
        x_axis: [slice[0], slice[4], slice[8], slice[12]].into(),
        y_axis: [slice[1], slice[5], slice[9], slice[13]].into(),
        z_axis: [slice[2], slice[6], slice[10], slice[14]].into(),
        w_axis: [slice[3], slice[7], slice[11], slice[15]].into(),
    }
}
fn make_matrix3_from_row(slice: [f64; 9]) -> DMat3 {
    DMat3 {
        x_axis: [slice[0], slice[3], slice[6]].into(),
        y_axis: [slice[1], slice[4], slice[7]].into(),
        z_axis: [slice[2], slice[5], slice[8]].into(),
    }
}
