use super::pan_orbit::{pan_orbit_camera, to_mat4_32};
use super::{camera_event_aggregator, egui};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::{DMat3, DMat4, DQuat, DVec3};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::render::primitives::Frustum;
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_easings::Lerp;
use bevy_egui::EguiSet;
use egui::EguiWantsFocus;
use houtu_scene::{
    acos_clamped, equals_epsilon, zero_to_two_pi, Cartesian3, Cartographic, Ellipsoid,
    EllipsoidGeodesic, GeographicProjection, HeadingPitchRoll, IntersectionTests, Matrix3, Matrix4,
    Projection, Quaternion, Rectangle, Transforms, EPSILON2, EPSILON3, RADIANS_PER_DEGREE,
};
use std::f64::consts::{FRAC_PI_2, PI, TAU};
use std::f64::NEG_INFINITY;

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(camera_event_aggregator::Plugin);
        app.add_system(globe_camera_setup_system);
        // app.init_resource::<EguiWantsFocus>()
        //     .add_system(
        //         egui::check_egui_wants_focus
        //             .after(EguiSet::InitContexts)
        //             .before(PanOrbitCameraSystemSet),
        //     )
        //     .configure_set(
        //         PanOrbitCameraSystemSet.run_if(resource_equals(EguiWantsFocus {
        //             prev: false,
        //             curr: false,
        //         })),
        //     );
    }
}

/// Base system set to allow ordering of `GlobeCamera`
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[system_set(base)]
pub struct PanOrbitCameraSystemSet;

pub struct GlobeCameraFrustum {
    fov: f64,
    fovy: f64,
    near: f64,
    far: f64,
    xOffset: f64,
    yOffset: f64,
    aspectRatio: f64,
    _sseDenominator: f64,
}
impl Default for GlobeCameraFrustum {
    fn default() -> Self {
        let mut me = Self {
            fov: (60.0 as f64).to_radians(),
            near: 1.0,
            far: 500000000.0,
            xOffset: 0.0,
            yOffset: 0.0,
            aspectRatio: 1.,
            _sseDenominator: 0.,
            fovy: 0.,
        };
        me.update_self();
        return me;
    }
}
impl GlobeCameraFrustum {
    fn update_self(&mut self) {
        self._sseDenominator = (2.0 * (0.5 * self.fov)).tan();
        self.fovy = {
            if self.aspectRatio <= 1.0 {
                self.fov
            } else {
                ((self.fov * 0.5).tan() / self.aspectRatio).atan() * 2.0
            }
        };
    }
}

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
    pub defaultRotateAmount: f64,
    pub defaultZoomAmount: f64,
    pub defaultMoveAmount: f64,
    pub maximumZoomFactor: f64,
    pub percentageChanged: f64,
    pub _viewMatrix: DMat4,
    pub _invViewMatrix: DMat4,

    pub hpr: HeadingPitchRoll,
    pub _maxCoord: DVec3,

    pub frustum: GlobeCameraFrustum,
    inited: bool,
}

impl Default for GlobeCamera {
    fn default() -> Self {
        let max_coord = GeographicProjection::WGS84.project(&Cartographic::new(PI, FRAC_PI_2, 0.));

        let mut me = Self {
            positionWCDeltaMagnitude: 0.0,
            positionWCDeltaMagnitudeLastFrame: 0.0,
            timeSinceMoved: 0.0,
            _lastMovedTimestamp: 0.0,
            defaultMoveAmount: 100000.0,
            defaultLookAmount: PI / 60.0,
            defaultRotateAmount: PI / 3600.0,
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
            _maxCoord: max_coord,
            frustum: GlobeCameraFrustum::default(),
            inited: false,
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
    for (mut globe_camera, mut transform, frustum, projection, camera) in &mut query {
        if globe_camera.inited == true {
            return;
        }
        globe_camera.inited = true;
        if let bevy::prelude::Projection::Perspective(projection) = projection {
            let frustum = &mut globe_camera.frustum;
            frustum.far = projection.far as f64;
            frustum.aspectRatio = projection.aspect_ratio as f64;
            frustum.near = projection.near as f64;
            frustum.fov = projection.fov as f64;
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
        globe_camera.update_self();
        globe_camera.update_camera_matrix(&mut transform);
    }
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
        self.frustum.update_self();
        self.updateViewMatrix();
        self.position = self
            .rectangleCameraPosition3D(&GlobeCamera::DEFAULT_VIEW_RECTANGLE, Some(true))
            .unwrap();

        let mut mag = self.position.magnitude();
        mag += mag * Self::DEFAULT_VIEW_FACTOR;
        self.position = self.position.normalize().multiply_by_scalar(mag);
        self.updateMembers();
    }
    pub fn get_position_wc(&mut self) -> DVec3 {
        self.updateMembers();
        return self._positionWC;
    }
    pub fn get_position_cartographic(&mut self) -> Cartographic {
        self.updateMembers();
        return self._positionCartographic;
    }
    pub fn get_direction_wc(&mut self) -> DVec3 {
        self.updateMembers();
        return self._directionWC;
    }

    pub fn get_up_wc(&mut self) -> DVec3 {
        self.updateMembers();
        return self._upWC;
    }

    pub fn get_right_wc(&mut self) -> DVec3 {
        self.updateMembers();
        return self._rightWC;
    }
    pub fn get_transform(&mut self) -> DMat4 {
        return self._transform;
    }
    pub fn get_inverse_transform(&mut self) -> DMat4 {
        self.updateMembers();
        return self._transform;
    }
    pub fn get_view_matrix(&mut self) -> DMat4 {
        self.updateMembers();
        return self._transform;
    }
    pub fn get_inverse_view_matrix(&mut self) -> DMat4 {
        self.updateMembers();
        return self._transform;
    }
    pub fn get_heading(&mut self) -> f64 {
        let ellipsoid = Ellipsoid::WGS84;

        let oldTransform = self._transform.clone();
        let transform =
            Transforms::eastNorthUpToFixedFrame(&self.get_position_wc(), Some(ellipsoid));
        self._setTransform(&transform);

        let heading = getHeading(&self.direction, &self.up);

        self._setTransform(&oldTransform);

        return heading;
    }
    pub fn get_pitch(&mut self) -> f64 {
        let ellipsoid = Ellipsoid::WGS84;

        let oldTransform = self._transform.clone();
        let transform =
            Transforms::eastNorthUpToFixedFrame(&self.get_position_wc(), Some(ellipsoid));
        self._setTransform(&transform);

        let pitch = getPitch(&self.direction);

        self._setTransform(&oldTransform);

        return pitch;
    }
    pub fn get_roll(&mut self) -> f64 {
        let ellipsoid = Ellipsoid::WGS84;

        let oldTransform = self._transform.clone();
        let transform =
            Transforms::eastNorthUpToFixedFrame(&self.get_position_wc(), Some(ellipsoid));
        self._setTransform(&transform);

        let roll = getRoll(&self.direction, &self.up, &self.right);

        self._setTransform(&oldTransform);

        return roll;
    }

    pub fn updateViewMatrix(&mut self) {
        self._viewMatrix =
            DMat4::compute_view(&self.position, &self.direction, &self.up, &self.right);
        self._viewMatrix = self._viewMatrix * self._actualInvTransform;
        self._invViewMatrix = self._viewMatrix.inverse_transformation();
    }
    pub fn update_camera_matrix(&self, transform: &mut Transform) {
        transform.translation = Vec3::new(
            self.position.x as f32,
            self.position.y as f32,
            self.position.z as f32,
        );
        transform.rotation = Quat::from_mat4(&to_mat4_32(&self._transform));
    }
    pub fn getRectangleCameraCoordinates(&mut self, rectangle: &Rectangle) -> Option<DVec3> {
        return self.rectangleCameraPosition3D(rectangle, None);
    }
    pub fn set_view(
        &mut self,
        destination: Option<DVec3>,
        orientation: Option<SetViewOrientation>,
        endTransform: Option<DMat4>,
        convert: Option<bool>,
    ) {
        if (endTransform.is_some()) {
            self._setTransform(&endTransform.unwrap());
        }
        let convert = convert.unwrap_or(true);
        let destination = destination.unwrap_or(self._positionWC.clone());
        let hpr = if let Some(orientation) = orientation {
            match orientation {
                SetViewOrientation::DirectionUp(v) => {
                    self.directionUpToHeadingPitchRoll(&v.direction, &v)
                }
                SetViewOrientation::HeadingPitchRoll(z) => z,
            }
        } else {
            HeadingPitchRoll::new(0.0, -FRAC_PI_2, 0.0)
        };
        self.set_view_3d(&destination, &hpr);
    }
    fn set_view_3d(&mut self, position: &DVec3, hpr: &HeadingPitchRoll) {
        let currentTransform = self._transform.clone();
        let localTransform = Transforms::eastNorthUpToFixedFrame(&position, None);
        self._setTransform(&localTransform);
        let mut hpr = hpr.clone();

        self.position = DVec3::ZERO.clone();
        hpr.heading = hpr.heading - FRAC_PI_2;

        let rotQuat = DQuat::from_heading_pitch_roll(&hpr);
        let rotMat = DMat3::from_quat(rotQuat);

        self.direction = rotMat.col(0);
        self.up = rotMat.col(2);
        self.right = self.direction.cross(self.up);

        self._setTransform(&currentTransform);
    }
    fn directionUpToHeadingPitchRoll(
        &mut self,
        position: &DVec3,
        orientation: &DirectionUp,
    ) -> HeadingPitchRoll {
        let mut direction = orientation.direction.clone();
        let mut up = orientation.up.clone();

        let ellipsoid = Ellipsoid::WGS84;
        let transform = Transforms::eastNorthUpToFixedFrame(&position, Some(ellipsoid));
        let mut invTransform = transform.inverse_transformation();

        direction = invTransform.multiply_by_point_as_vector(&direction);
        up = invTransform.multiply_by_point_as_vector(&up);

        let right = direction.cross(up);
        let mut result = HeadingPitchRoll::default();
        result.heading = getHeading(&direction, &up);
        result.pitch = getPitch(&direction);
        result.roll = getRoll(&direction, &up, &right);

        return result;
    }
    fn _setTransform(&mut self, transform: &DMat4) {
        let position = self._positionWC.clone();
        let up = self._upWC.clone();
        let direction = self._directionWC.clone();

        self._transform = transform.clone();
        self._transformChanged = true;
        self.updateMembers();
        let inverse = self._actualInvTransform;

        self.position = inverse.multiply_by_point(&position);
        self.direction = inverse.multiply_by_point_as_vector(&direction);
        self.up = inverse.multiply_by_point_as_vector(&up);
        self.right = self.direction.cross(self.up);

        self.updateMembers();
    }
    fn updateMembers(&mut self) {
        // let mode = self._mode;

        let heightChanged = false;
        let height = 0.0;

        let positionChanged = !self._position.equals(self.position) || heightChanged;
        if (positionChanged) {
            self._position = self.position.clone();
        }

        let directionChanged = !self._direction.equals(self.direction);
        if (directionChanged) {
            self._direction = self.direction.normalize();
        }

        let upChanged = !self._up.equals(self.up);
        if (upChanged) {
            self._up = self.up.normalize();
        }

        let rightChanged = !self._right.equals(self.right);
        if (rightChanged) {
            self._right = self.right.normalize();
        }

        let transformChanged = self._transformChanged;
        self._transformChanged = false;

        if (transformChanged) {
            self._invTransform = self._transform.inverse_transformation();

            self._actualTransform = self._transform.clone();

            self._actualInvTransform = self._actualTransform.inverse_transformation();
        }

        let transform = self._actualTransform;

        if (positionChanged || transformChanged) {
            self._positionWC = transform.multiply_by_point(&self._position);

            // Compute the Cartographic position of the self.

            self._positionCartographic = Ellipsoid::WGS84
                .cartesianToCartographic(&self._positionWC)
                .unwrap();
        }

        if (directionChanged || upChanged || rightChanged) {
            let det = self._direction.dot(self._up.cross(self._right));
            if ((1.0 - det).abs() > EPSILON2) {
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

        if (directionChanged || transformChanged) {
            self._directionWC = transform
                .multiply_by_point_as_vector(&self._direction)
                .normalize();
        }

        if (upChanged || transformChanged) {
            self._upWC = transform.multiply_by_point_as_vector(&self._up).normalize();
        }

        if (rightChanged || transformChanged) {
            self._rightWC = transform
                .multiply_by_point_as_vector(&self._right)
                .normalize();
        }

        if (positionChanged || directionChanged || upChanged || rightChanged || transformChanged) {
            self.updateViewMatrix();
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
        let cameraPosition = self.position + moveScratch;
    }
    pub fn rotate(&mut self, axis: DVec3, angle: Option<f64>) {
        let turnAngle = angle.unwrap_or(self.defaultRotateAmount);
        let quaternion = DQuat::from_axis_angle(axis, -turnAngle);
        let rotation = DMat3::from_quat(quaternion);
        self.position = rotation.multiply_by_vector(&self.position);
        self.direction = rotation.multiply_by_vector(&self.direction);
        self.up = rotation.multiply_by_vector(&self.up);
        self.right = self.direction.cross(self.up);
        self.up = self.right.cross(self.direction);
    }
    pub fn getPickRay(&mut self, windowPosition: &Vec2, window_size: &Vec2) -> houtu_scene::Ray {
        return self.getPickRayPerspective(windowPosition, window_size);
    }
    pub fn pickEllipsoid(&mut self, windowPosition: &Vec2, window_size: &Vec2) -> Option<DVec3> {
        return self.pickEllipsoid3D(windowPosition, window_size);
    }
    pub fn pickEllipsoid3D(&mut self, windowPosition: &Vec2, window_size: &Vec2) -> Option<DVec3> {
        let ellipsoid = Ellipsoid::WGS84;
        let ray = self.getPickRay(windowPosition, window_size);
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
    pub fn getPickRayPerspective(
        &mut self,
        windowPosition: &Vec2,
        window_size: &Vec2,
    ) -> houtu_scene::Ray {
        let mut result = houtu_scene::Ray::default();
        let width = window_size.x as f64;
        let height = window_size.y as f64;
        let aspectRatio = width / height;
        let tanPhi = (self.frustum.fovy as f64 * 0.5).tan();
        let tanTheta = aspectRatio * tanPhi;
        let near = self.frustum.near as f64;

        let x = (2.0 / width) * windowPosition.x as f64 - 1.0;
        let y = (2.0 / height) * (height - windowPosition.y as f64) - 1.0;

        let position = self.get_position_wc();
        result.origin = position.clone();

        let mut nearCenter = self.get_direction_wc().multiply_by_scalar(near);
        nearCenter = position + nearCenter;
        let xDir = self.get_right_wc().multiply_by_scalar(x * near * tanTheta);
        let yDir = self.get_up_wc().multiply_by_scalar(y * near * tanPhi);
        result.direction = (nearCenter + xDir + yDir - position).normalize();
        return result;
    }
    pub fn rectangleCameraPosition3D(
        &mut self,
        rectangle: &Rectangle,
        updateCamera: Option<bool>,
    ) -> Option<DVec3> {
        let ellipsoid = Ellipsoid::WGS84;
        let updateCamera = updateCamera.unwrap_or(false);
        let north = rectangle.north;
        let south = rectangle.south;
        let mut east = rectangle.east;
        let west = rectangle.west;

        // If we go across the International Date Line
        if (west > east) {
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
        if (south < -FRAC_PI_2 + RADIANS_PER_DEGREE && north > FRAC_PI_2 - RADIANS_PER_DEGREE) {
            latitude = 0.0;
        } else {
            let northCartographic = Cartographic::from_radians(longitude, north, 0.);
            let southCartographic = Cartographic::from_radians(longitude, south, 0.);
            let mut ellipsoidGeodesic = EllipsoidGeodesic::default();
            ellipsoidGeodesic.setEndPoints(northCartographic, southCartographic);
            latitude = ellipsoidGeodesic.interpolateUsingFraction(0.5).latitude;
        }

        let centerCartographic = Cartographic::from_radians(longitude, latitude, 0.0);

        let center = ellipsoid.cartographicToCartesian(&centerCartographic);

        let mut cart = Cartographic::default();
        cart.longitude = east;
        cart.latitude = north;
        let mut northEast = ellipsoid.cartographicToCartesian(&cart);
        cart.longitude = west;
        let mut northWest = ellipsoid.cartographicToCartesian(&cart);
        cart.longitude = longitude;
        let mut northCenter = ellipsoid.cartographicToCartesian(&cart);
        cart.latitude = south;
        let mut southCenter = ellipsoid.cartographicToCartesian(&cart);
        cart.longitude = east;
        let mut southEast = ellipsoid.cartographicToCartesian(&cart);
        cart.longitude = west;
        let mut southWest = ellipsoid.cartographicToCartesian(&cart);

        northWest = northWest.subtract(center);
        southEast = southEast.subtract(center);
        northEast = northEast.subtract(center);
        southWest = southWest.subtract(center);
        northCenter = northCenter.subtract(center);
        southCenter = southCenter.subtract(center);

        let mut direction = ellipsoid.geodeticSurfaceNormal(&center);
        let mut direction = if let Some(v) = direction {
            v
        } else {
            return None;
        };
        direction = direction.negate();
        let right = direction.cross(DVec3::UNIT_Z).normalize();
        let up = right.cross(direction);
        if updateCamera {
            self.direction = direction.clone();
            self.up = up.clone();
            self.right = right.clone();
        }
        let mut d;
        let tanPhi = (self.frustum.fovy * 0.5).tan();
        let tanTheta = self.frustum.aspectRatio * tanPhi;

        d = [
            computeD(&direction, &up, &northWest, tanPhi),
            computeD(&direction, &up, &southEast, tanPhi),
            computeD(&direction, &up, &northEast, tanPhi),
            computeD(&direction, &up, &southWest, tanPhi),
            computeD(&direction, &up, &northCenter, tanPhi),
            computeD(&direction, &up, &southCenter, tanPhi),
            computeD(&direction, &right, &northWest, tanTheta),
            computeD(&direction, &right, &southEast, tanTheta),
            computeD(&direction, &right, &northEast, tanTheta),
            computeD(&direction, &right, &southWest, tanTheta),
            computeD(&direction, &right, &northCenter, tanTheta),
            computeD(&direction, &right, &southCenter, tanTheta),
        ]
        .iter()
        .fold(NEG_INFINITY, |a, &b| a.max(b));

        // If the rectangle crosses the equator, compute D at the equator, too, because that's the
        // widest part of the rectangle when projected onto the globe.
        if (south < 0. && north > 0.) {
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
    if (!equals_epsilon(direction.z.abs(), 1.0, Some(EPSILON3), None)) {
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
    if (!equals_epsilon(direction.z.abs(), 1.0, Some(EPSILON3), None)) {
        roll = (-right.z).atan2(up.z);
        roll = zero_to_two_pi(roll + TAU);
    }

    return roll;
}

#[cfg(test)]
mod tests {
    use houtu_scene::{EPSILON10, EPSILON15, EPSILON6};

    use super::*;
    const clientWidth: f32 = 512.;
    const clientHeight: f32 = 384.;
    const drawingBufferWidth: f32 = 1024.;
    const drawingBufferHeight: f32 = 768.;

    const moveAmount: f64 = 3.0;
    const turnAmount: f64 = FRAC_PI_2;
    const rotateAmount: f64 = FRAC_PI_2;
    const zoomAmount: f64 = 1.0;

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
        camera.frustum.aspectRatio = (clientWidth / clientHeight) as f64;
        camera.frustum.update_self();
        // camera.updateMembers();
        let windowCoord = Vec2::new(clientWidth as f32 / 2.0, clientHeight as f32);
        let ray = camera.getPickRay(
            &windowCoord,
            &Vec2::new(clientWidth as f32, clientHeight as f32),
        );
        let windowHeight = camera.frustum.near * (camera.frustum.fovy * 0.5).tan();
        let expectedDirection = DVec3::new(0.0, -windowHeight, -1.0).normalize();
        assert!(ray.origin.equals(camera.position));
        assert!(ray
            .direction
            .equals_epsilon(expectedDirection, Some(EPSILON15), None));
    }
    #[test]
    fn getRectangleCameraCoordinates() {
        let mut camera = GlobeCamera::default();
        camera.position = DVec3::UNIT_Z;
        camera.up = DVec3::UNIT_Y;
        camera.direction = DVec3::UNIT_Z.negate();
        camera.right = camera.direction.cross(camera.up);
        camera.frustum.aspectRatio = (clientWidth / clientHeight) as f64;
        camera.update_self();
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
        let dir = DVec3::new(1.0, 1.0, 1.0).normalize();
        camera.move_direction(&dir, moveAmount);
        assert!(camera.position.equals_epsilon(
            DVec3::new(dir.x * moveAmount, dir.y * moveAmount, 1.0),
            Some(EPSILON10),
            None
        ));
        assert!(camera.up.equals(up));
        assert!(camera.direction.equals(direction));
        assert!(camera.right.equals(right));
    }
    fn get_camera() -> GlobeCamera {
        let mut camera = GlobeCamera::default();
        camera.update_self();
        camera.position = DVec3::UNIT_Z;
        camera.up = DVec3::UNIT_Y;
        camera.direction = DVec3::UNIT_Z.negate();
        camera.right = camera.direction.cross(camera.up);
        camera.frustum.aspectRatio = (clientWidth / clientHeight) as f64;
        camera.update_self();
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
}
