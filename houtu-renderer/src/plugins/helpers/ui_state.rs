use bevy::prelude::*;
#[derive(Default, Resource)]
pub struct UiState {
    pub label: String,
    //genera
    pub show_xyz: bool,
    pub show_frustum: bool,
    pub show_frustum_planes: bool,
    pub show_performance: bool,
    //terrain
    pub show_wireframe: bool,
    pub suspend_lod_update: bool,
    pub show_tile_coordinates: bool,
    //camera
    pub debug_camera_position: bool,
    pub debug_camera_dur: bool,
}
