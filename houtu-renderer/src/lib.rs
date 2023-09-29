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
mod quadtree;
mod render;
mod image;
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
        // app.add_plugins(MinimalPlugins);
        // app.add_plugins((
        //     WebAssetPlugin::default(),
        //     WindowPlugin {
        //         primary_window: Some(Window {
        //             title: "houtu".to_string(),
        //             fit_canvas_to_parent: true,
        //             ..Default::default()
        //         }),
        //         ..Default::default()
        //     },
        //     bevy::a11y::AccessibilityPlugin,
        //     bevy::winit::WinitPlugin::default(),
        //     bevy::render::RenderPlugin::default(),
        //     bevy::render::texture::ImagePlugin::default(),
        //     bevy::log::LogPlugin::default(),
        //     bevy::input::InputPlugin::default(),
        //     bevy::core_pipeline::CorePipelinePlugin::default(),
        //     bevy::transform::TransformPlugin::default(),
        //     bevy::diagnostic::DiagnosticsPlugin,
        //     bevy::diagnostic::FrameTimeDiagnosticsPlugin,
        // ));
        // app.add_plugins((
        //     helpers::Plugin,
        //     houtu_jobs::Plugin,
        //     globe::GlobePlugin,
        //     camera::CameraPlugin,
        //     quadtree::Plugin,
        //     render::Plugin,
        // ));
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
            .add_plugins(helpers::Plugin) //bevy_egui的插件会让wasm下canavas显示变成灰色，暂时先不用。
            .add_plugins(houtu_jobs::Plugin)
            .add_plugins(camera::CameraPlugin)
            .add_plugins(quadtree::Plugin)
            .add_plugins(render::Plugin);
        #[cfg(not(target_arch = "wasm32"))]
        app.add_plugins(WorldInspectorPlugin::new());
        // .add_plugin(plugins::wmts::WMTSPlugin);
    }
}
