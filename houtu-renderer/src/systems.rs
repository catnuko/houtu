use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use std::f32::consts::PI;
use houtu_scene::Shape;

use crate::{jobs::MeshBuildingJob, RenderEntityType};

pub fn system_set(app:&mut App) {
    app.add_system(rotate);
}
fn rotate(mut query: Query<&mut Transform, With<Shape>>, time: Res<Time>) {
    for mut transform in &mut query {
        transform.rotate_y(time.delta_seconds() / 2.);
    }
}