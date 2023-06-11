use std::f64::consts::PI;

use bevy::{
    math::{DVec2, DVec3},
    prelude::*,
    window::PrimaryWindow,
};
use bevy_prototype_debug_lines::*;
use houtu_scene::{Cartesian2, Cartesian3, Ellipsoid, HeadingPitchRoll};
pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(DebugLinesPlugin::with_depth_test(true));
        app.add_startup_system(setup);
    }
}
fn setup(mut commands: Commands, mut lines: ResMut<DebugLines>) {
    let length = (Ellipsoid::WGS84.maximumRadius as f32) + 10000000.0;
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
