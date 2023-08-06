use bevy::prelude::*;
#[derive(Resource)]
pub struct UiState {
    pub label: String,
    //genera
    pub show_xyz: bool,
    pub show_frustum: bool,
    pub show_frustum_planes: bool,
    pub frustum_planes_entity: Option<Entity>,
    pub show_performance: bool,
    //terrain
    pub show_wireframe: bool,
    pub suspend_lod_update: bool,
    pub show_tile_coordinates: bool,
    //camera
    pub debug_camera_position: bool,
    pub debug_camera_dur: bool,
}
impl Default for UiState {
    fn default() -> Self {
        UiState {
            label: "test".to_string(),
            show_xyz: true,
            show_frustum: false,
            show_frustum_planes: false,
            frustum_planes_entity: None,
            show_performance: false,
            show_wireframe: false,
            suspend_lod_update: false,
            show_tile_coordinates: false,
            debug_camera_position: false,
            debug_camera_dur: false,
        }
    }
}
