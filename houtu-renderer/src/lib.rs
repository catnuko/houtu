use bevy::{pbr::{wireframe::WireframePlugin, PbrPlugin}, prelude::*};

//https://github.com/valkum/terrain_tests
//https://github.com/Dimev/lodtree

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_web_asset::WebAssetPlugin;

mod camera;
mod globe;

mod bing_maps_imagery_provider;
mod helpers;
mod quadtree;
mod render;
mod wmts_imagery_provider;
mod xyz_imagery_provider;
// use plugins::quadtree;

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
            .add_plugins(
                DefaultPlugins
                    .build()
                    .disable::<AssetPlugin>()
                    .set(WindowPlugin {
                        primary_window: Some(Window {
                            title: "houtu!".into(),
                            // resolution: WindowResolution::new(900., 900.0 / 0.660105980317941),
                            ..default()
                        }),
                        ..default()
                    }),
            )
            .add_plugin(helpers::Plugin)
            .add_plugin(WireframePlugin)
            .add_plugin(WorldInspectorPlugin::new())
            .add_plugin(houtu_jobs::Plugin)
            .add_plugin(globe::GlobePlugin)
            .add_plugin(camera::CameraPlugin)
            .add_plugin(quadtree::Plugin)
            .add_plugin(render::Plugin);
        // .add_plugin(plugins::wmts::WMTSPlugin);
    }
}
