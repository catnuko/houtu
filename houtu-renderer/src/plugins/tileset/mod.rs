use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use houtu_scene::IndicesAndEdgesCache;

mod create_terrain_mesh_job;
mod globe_surface_tile;
mod imagery;
mod imagery_layer;
mod quadtree_tile;
mod renderer;
mod reproject_texture;
mod terrain_datasource;
mod terrian_material;
mod tile_key;
mod tile_quad_tree;
mod tile_replacement_queue;
mod tile_selection_result;
mod traversal_details;
mod upsample_job;
pub use tile_key::TileKey;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<terrian_material::TerrainMeshMaterial>::default());
        app.add_plugin(tile_quad_tree::Plugin);
    }
}
