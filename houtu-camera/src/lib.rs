use bevy::prelude::*;
use geodesy::preamble::*;

pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
    }
}
impl Default for Plugin {
    fn default() -> Self {
        Self {}
    }
}

fn setup(
    mut commands: Commands,
) {
    let ellipsoid = Ellipsoid::named("WGS84").unwrap();
    let x = ellipsoid.semimajor_axis() as f32;
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(x+10000000., x, x).looking_at(Vec3::new(0., 0., 0.), Vec3::Y),
        ..default()
    });
}