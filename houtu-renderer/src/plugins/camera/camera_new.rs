use super::pan_orbit::pan_orbit_camera;
use super::{camera_event_aggregator, egui};
use bevy::input::mouse::{MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::math::{DMat4, DVec3};
use bevy::prelude::*;
use bevy::render::camera::RenderTarget;
use bevy::window::{PrimaryWindow, WindowRef};
use bevy_easings::Lerp;
use bevy_egui::EguiSet;
use egui::EguiWantsFocus;
use houtu_scene::{
    Cartesian3, Cartographic, Ellipsoid, EllipsoidGeodesic, GeographicProjection, HeadingPitchRoll,
    IntersectionTests, Matrix4, Projection, Rectangle, RADIANS_PER_DEGREE,
};
use std::f64::consts::{FRAC_PI_2, PI, TAU};
use std::f64::NEG_INFINITY;

pub struct CameraControlPlugin;

impl Plugin for CameraControlPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(camera_event_aggregator::Plugin);
        app.init_resource::<EguiWantsFocus>()
            .add_system(
                egui::check_egui_wants_focus
                    .after(EguiSet::InitContexts)
                    .before(PanOrbitCameraSystemSet),
            )
            .configure_set(
                PanOrbitCameraSystemSet.run_if(resource_equals(EguiWantsFocus {
                    prev: false,
                    curr: false,
                })),
            );
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
        Self {
            fov: 0.,
            near: 1.0,
            far: 500000000.0,
            xOffset: 0.0,
            yOffset: 0.0,
            aspectRatio: 0.,
            _sseDenominator: 0.,
            ..Default::default()
        }
    }
}
impl GlobeCameraFrustum {}

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
        };
        me.updateViewMatrix();
        return me;
    }
}
impl GlobeCamera {
    pub fn get_positionWC(&mut self) -> DVec3 {
        self.updateViewMatrix();
        return self._positionWC;
    }
    pub fn updateMembers(&mut self) {}
    pub fn updateViewMatrix(&mut self) {
        self._viewMatrix =
            DMat4::compute_view(&self.position, &self.direction, &self.up, &self.right);
        self._viewMatrix = self._viewMatrix * self._actualInvTransform;
        self._invViewMatrix = self._viewMatrix.inverse_transformation();
    }
    pub fn set_view(
        &mut self,
        destination: Option<DVec3>,
        orientation: Option<HeadingPitchRoll>,
        endTransform: Option<DMat4>,
        convert: Option<bool>,
    ) {
    }
    pub fn zoom_in(&mut self, amout: Option<f64>) {}
    pub fn zoom_out(&mut self, amout: Option<f64>) {}
    pub fn move_direction(&mut self, direction: &DVec3, amout: f64) {
        let moveScratch = direction.multiply_by_scalar(amout);
        let cameraPosition = self.position + moveScratch;
    }
    pub fn rotate(&mut self, axis: DVec3, angle: f64) {}
    pub fn getPickRay(
        &mut self,
        windowPosition: &Vec2,
        window_size: &Vec2,
        projection: &PerspectiveProjection,
    ) -> Option<houtu_scene::Ray> {
        if window_size.x <= 0. && window_size.y <= 0. {
            return None;
        }
        return Some(self.getPickRayPerspective(windowPosition, window_size, projection));
    }
    pub fn pickEllipsoid(
        &self,
        windowPosition: &Vec2,
        window_size: &Vec2,
        projection: &PerspectiveProjection,
    ) -> Option<DVec3> {
        return self.pickEllipsoid3D(windowPosition, window_size, projection);
    }
    pub fn pickEllipsoid3D(
        &self,
        windowPosition: &Vec2,
        window_size: &Vec2,
        projection: &PerspectiveProjection,
    ) -> Option<DVec3> {
        let ellipsoid = Ellipsoid::WGS84;
        let ray = if let Some(v) = self.getPickRay(windowPosition, window_size, projection) {
            v
        } else {
            return None;
        };

        let intersection = IntersectionTests::rayEllipsoid(&ray);
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
        projection: &PerspectiveProjection,
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

        let position = self._positionWC;
        result.origin = position.clone();

        let mut nearCenter = self._directionWC.multiply_by_scalar(near);
        nearCenter = position + nearCenter;
        let xDir = self._rightWC.multiply_by_scalar(x * near * tanTheta);
        let yDir = self._upWC.multiply_by_scalar(y * near * tanPhi);
        let mut direction = nearCenter + xDir;
        direction = direction + yDir;
        direction = direction + position;
        direction = direction.normalize();
        result.direction = direction;
        return result;
    }
    pub fn rectangleCameraPosition3D(
        &mut self,
        rectangle: &Rectangle,
        updateCamera: bool,
    ) -> Option<DVec3> {
        let ellipsoid = Ellipsoid::WGS84;

        let north = rectangle.north;
        let south = rectangle.south;
        let east = rectangle.east;
        let west = rectangle.west;

        // If we go across the International Date Line
        if (west > east) {
            east += FRAC_PI_2;
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
            let ellipsoidGeodesic = EllipsoidGeodesic::default();
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
        let d;
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
            let equatorCartographic = Cartographic::from_radians(west, 0.0, 0.0);
            let equatorPosition = ellipsoid.cartographicToCartesian(&equatorCartographic);
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
