use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
};

mod events;
mod jobs;
mod plugins;
mod systems;
mod z_index;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_web_asset::WebAssetPlugin;
use plugins::quadtree;
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
        app.add_plugin(WebAssetPlugin::default())
            .add_plugins(DefaultPlugins.build().disable::<AssetPlugin>())
            .add_plugin(WireframePlugin)
            .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(houtu_jobs::Plugin)
            .add_plugin(quadtree::Plugin)
            .add_plugin(plugins::globe::GlobePlugin)
            .add_plugin(plugins::camera::CameraPlugin)
            .add_plugin(plugins::scene::ScenePlugin)
            .add_system(setup);
        // .add_plugin(plugins::wmts::WMTSPlugin);
    }
}
fn setup(mut wireframe_config: ResMut<WireframeConfig>) {
    wireframe_config.global = false
}
