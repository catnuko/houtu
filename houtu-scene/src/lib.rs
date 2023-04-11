use bevy::prelude::*;
use std::f32::consts::PI;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use geodesy::preamble::*;

mod globe;
mod ellipsoid;
mod camera;
pub use globe::Shape;
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(bevy::pbr::PbrPlugin::default());
        app.add_plugin(globe::GlobePlugin::default());
    }
}