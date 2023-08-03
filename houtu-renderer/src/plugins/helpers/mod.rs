use std::f64::consts::PI;

use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_prototype_debug_lines::*;
use houtu_scene::{Cartesian2, Cartesian3, Ellipsoid, HeadingPitchRoll};
mod font;
mod toggle_switch;
mod ui_state;
pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .add_plugin(DebugLinesPlugin::with_depth_test(true))
            .add_startup_system(font::config_ctx)
            .add_startup_system(ui_state::configure_ui_state_system)
            .add_system(ui_example_system)
            .add_startup_system(setup);
    }
}

fn ui_example_system(mut contexts: EguiContexts) {
    let ctx = contexts.ctx_mut();
    egui::Window::new("后土地球")
        .resizable(true)
        .default_pos([700.0, 100.0])
        .default_width(280.0)
        .show(ctx, |ui| {
            egui::Grid::new("my_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    if ui.button("获取经纬度1").clicked() {
                        println!("获取经纬度1")
                    }
                    if ui.button("获取经纬度2").clicked() {
                        println!("获取经纬度2")
                    }
                    ui.end_row();
                });
        });
}

fn setup(mut commands: Commands, mut lines: ResMut<DebugLines>) {
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
