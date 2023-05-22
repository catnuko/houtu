use bevy::prelude::*;

pub struct GlobeMapCameraPlugin;

impl Plugin for GlobeMapCamera {
    fn build(&self, app: &mut bevy::prelude::App) {}
}
#[derive(Component, Copy, Clone, Debug, PartialEq)]
pub struct GlobeMapCamera {
    pub rotationMouseDeltaFactor: f64,
    pub orbitingMouseDeltaFactor: f64,
    pub orbitingTouchDeltaFactor: f64,
    pub enabled: bool,
    pub zoomEnabled: bool,
    pub panEnabled: bool,
    pub tiltEnabled: bool,
    pub rotateEnabled: bool,
    pub inertiaEnabled: bool,
    pub zoomInertiaDampingDuration: f64,
    pub panInertiaDampingDuration: f64,
    pub tiltToggleDuration: f64,
    pub tiltAngle: f64,
    pub northResetAnimationDuration: f64,
    m_zoomLevelDeltaOnMouseWheel: f64,
    pub zoomLevelDeltaOnControl: f64,
    pub minZoomLevel: f64,
    pub maxZoomLevel: f64,
    pub minCameraHeight: f64,
    pub zoomLevelDeltaOnDoubleClick: f64,
    pub doubleTapTime: f64,
}
impl GlobeMapCamera {
    pub fn getZoomLevelDeltaOnMouseWheel(&self) -> f64 {
        0.
    }
    pub fn setZoomLevelDeltaOnMouseWheel(&self, delta: f64) {}
}
