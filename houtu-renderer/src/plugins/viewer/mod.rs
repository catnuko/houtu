use bevy::prelude::*;
use houtu_scene::GeographicProjection;

pub struct ViewerPlugin;

impl bevy::app::Plugin for ViewerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
}
