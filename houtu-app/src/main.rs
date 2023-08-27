//! A simple 3D scene with light shining over a cube sitting on a plane.

use bevy::prelude::*;

fn main() {
    App::new()
        // .add_plugins(DefaultPlugins)
        .add_plugin(houtu_renderer::RendererPlugin)
        .run();
}