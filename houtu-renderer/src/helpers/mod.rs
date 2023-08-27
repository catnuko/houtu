use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_prototype_debug_lines::*;
use houtu_scene::{Cartesian2, Cartesian3, Ellipsoid};

use self::ui_state::UiState;
mod camera;
mod font;
mod genera;
mod ui_state;
pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin)
            // .add_plugin(bevy_screen_diags::ScreenDiagsTextPlugin)
            .add_plugins(DebugLinesPlugin::with_depth_test(true))
            .insert_resource(UiState::default())
            .add_systems(Startup, (font::config_ctx, setup))
            .add_systems(
                Update,
                (
                    camera::debug_system,
                    ui_example_system,
                    genera::debug_system,
                ),
            );
    }
}

fn ui_example_system(mut contexts: EguiContexts, mut state: ResMut<UiState>) {
    let ctx = contexts.ctx_mut();
    egui::Window::new("后土地球")
        .default_pos([1600.0, 100.0])
        .resizable(false)
        .default_width(280.0)
        .default_height(700.0)
        .auto_sized()
        .show(ctx, |ui| {
            ui.collapsing("General", |ui| {
                // if ui.button("获取经纬度1").clicked() {
                //     println!("获取经纬度1")
                // }
                // if ui.button("获取经纬度2").clicked() {
                //     println!("获取经纬度2")
                // }
                ui.end_row();
                ui.checkbox(&mut state.show_frustum, "Show frustums");
                ui.end_row();

                ui.checkbox(&mut state.show_frustum_planes, "Show frustum planes");
                ui.end_row();
                ui.checkbox(&mut state.show_performance, "Show performance");
                ui.end_row();
            });
            ui.collapsing("Terrain", |ui| {
                ui.checkbox(&mut state.show_wireframe, "Wireframe");
                ui.end_row();

                ui.checkbox(&mut state.suspend_lod_update, "Suspend LOD update");
                ui.end_row();
                ui.checkbox(&mut state.show_tile_coordinates, "Show tile coordinates");
                ui.end_row();
            });
            ui.collapsing("Camera", |ui| {
                ui.checkbox(&mut state.debug_camera_position, "Debug camera position");
                ui.checkbox(&mut state.debug_camera_dur, "Debug camera DUR");
            })
        });
}

fn setup(_commands: Commands, mut lines: ResMut<DebugLines>) {
    let length = (Ellipsoid::WGS84.maximum_radius as f32) + 10000000.0;
    // A line that stays on screen 9 seconds
    lines.line_colored(
        Vec3::ZERO,
        Vec3::new(length, 0.0, 0.0),
        100000000.,
        Color::RED,
    );
    lines.line_colored(
        Vec3::ZERO,
        Vec3::new(0.0, length, 0.0),
        100000000.,
        Color::GREEN,
    );
    lines.line_colored(
        Vec3::ZERO,
        Vec3::new(0.0, 0.0, length),
        100000000.,
        Color::BLUE,
    );
}
