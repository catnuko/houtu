use bevy::prelude::*;

mod events;
mod jobs;
mod plugins;
mod systems;
mod z_index;
use z_index::ZIndex;

#[derive(Clone, Copy, Component, PartialEq, Eq)]
pub enum RenderEntityType {
    Polygon,
    LineString,
    Point,
    SelectedPolygon,
    SelectedLineString,
    SelectedPoint,
}

pub struct RendererPlugin;

impl Plugin for RendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(plugins::globe::GlobePlugin);
        app.add_plugin(plugins::camera::CameraPlugin);
    }
}
