use bevy::{
    pbr::{wireframe::WireframePlugin, PbrPlugin},
    prelude::*,
};

//https://github.com/valkum/terrain_tests
//https://github.com/Dimev/lodtree

use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_web_asset::WebAssetPlugin;

mod camera;
mod globe;

mod bing_maps_imagery_provider;
mod helpers;
mod image;
mod quadtree;
mod render;
mod wmts_imagery_provider;
mod xyz_imagery_provider;
mod quantized_mesh_terrain_data;
mod cesium_terrain_provider;
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
                            title: "后土地球!".into(),
                            canvas: Some("#rgis".into()), // selector
                            ..default()
                        }),
                        ..default()
                    }),
            )
            .add_plugins((
                helpers::Plugin,
                houtu_jobs::Plugin,
                camera::CameraPlugin,
                quadtree::Plugin,
                render::Plugin,
            )); //bevy_egui的插件会让wasm下canavas显示变成灰色，暂时先不用。
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(WorldInspectorPlugin::new());
        // .add_plugin(plugins::wmts::WMTSPlugin);
    }
}
