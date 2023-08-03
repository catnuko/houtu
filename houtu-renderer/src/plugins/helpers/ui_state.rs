use std::f64::consts::PI;

use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_egui::{egui, EguiContexts, EguiPlugin};
use bevy_prototype_debug_lines::*;
use houtu_scene::{Cartesian2, Cartesian3, Ellipsoid, HeadingPitchRoll};
#[derive(Default, Resource)]
pub struct UiState {
    pub label: String,
    pub value: f32,
    pub inverted: bool,
    pub egui_texture_handle: Option<egui::TextureHandle>,
    pub is_window_open: bool,
}
pub fn configure_ui_state_system(mut ui_state: ResMut<UiState>) {
    ui_state.is_window_open = true;
}
