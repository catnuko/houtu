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
        app.add_plugins(WebAssetPlugin::default())
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
            .add_plugins(helpers::Plugin)
            .add_plugins(WireframePlugin)
            .add_plugins(WorldInspectorPlugin::new())
            .add_plugins(houtu_jobs::Plugin)
            .add_plugins(globe::GlobePlugin)
            .add_plugins(camera::CameraPlugin)
            .add_plugins(quadtree::Plugin)
            .add_plugins(render::Plugin);
        // .add_plugin(plugins::wmts::WMTSPlugin);
    }
}
