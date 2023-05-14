use bevy::prelude::*;

mod file;
mod network;
mod systems;

pub struct LoadPlugin;

impl Default for LoadPlugin {
    fn default() -> Self {
        Self
    }
}
impl bevy::app::Plugin for LoadPlugin {
    fn build(&self, app: &mut App) {
        // app.add_system(systems::handle_network_fetch_finished_jobs);
    }
}
