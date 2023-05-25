use std::f64::consts::PI;

use bevy::math::{DMat4, DVec3};

use crate::{Cartographic, Matrix4};

pub struct Camera {
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
}
impl Default for Camera {
    fn default() -> Self {
        Self {
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
        }
    }
}

impl Camera {
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
    // pub fn updateCameraDeltas(&mut self) {
    //     if let Some(real_oldPositionWC) = self._oldPositionWC {
    //         self.positionWCDeltaMagnitudeLastFrame = self.positionWCDeltaMagnitude;
    //         let delta =
    //             Cartesian3.subtract(self.positionWC, self._oldPositionWC, self._oldPositionWC);
    //         self.positionWCDeltaMagnitude = Cartesian3.magnitude(delta);
    //         self._oldPositionWC = Cartesian3.clone(self.positionWC, self._oldPositionWC);

    //         // Update move timers
    //         if (self.positionWCDeltaMagnitude > 0.0) {
    //             self.timeSinceMoved = 0.0;
    //             self._lastMovedTimestamp = getTimestamp();
    //         } else {
    //             self.timeSinceMoved =
    //                 Math.max(getTimestamp() - self._lastMovedTimestamp, 0.0) / 1000.0;
    //         }
    //     } else {
    //         self._oldPositionWC = Some(self.get_positionWC().clone());
    //     }
    // }
}
