use bevy::{
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    window::WindowResolution,
};

mod events;
mod jobs;
mod plugins;
mod systems;
mod z_index;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_web_asset::WebAssetPlugin;
use houtu_scene::Ellipsoid;
use plugins::{helpers, pan_orbit_camera};
// use plugins::quadtree;
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
            // .add_plugin(quadtree::Plugin)
            .add_plugin(plugins::globe::GlobePlugin)
            .add_plugin(plugins::camera::CameraPlugin)
            .add_plugin(plugins::tileset::Plugin)
            // .add_plugin(plugins::scene::ScenePlugin)
            .add_startup_system(setup);
        // .add_plugin(plugins::wmts::WMTSPlugin);
    }
}
fn setup(mut commands: Commands, mut wireframe_config: ResMut<WireframeConfig>) {
    wireframe_config.global = false;
    // let ellipsoid = Ellipsoid::WGS84;
    // let x = ellipsoid.semimajor_axis() as f32;
    // commands.spawn((Camera3dBundle {
    //     transform: Transform::from_xyz(0., 0.0, 18478136.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..Default::default()
    // },));
}
