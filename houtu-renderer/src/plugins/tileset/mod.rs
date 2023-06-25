use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use houtu_scene::IndicesAndEdgesCache;

mod create_terrain_mesh_job;
mod globe_surface_tile;
mod imagery;
mod imagery_layer;
mod quadtree_tile;
mod reproject_texture;
mod terrian_material;
mod tile_key;
mod tile_quad_tree;
mod tile_replacement_queue;
mod tile_selection_result;
mod unsample_job;
pub use tile_key::TileKey;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<terrian_material::TerrainMeshMaterial>::default());
        app.add_plugin(tile_quad_tree::Plugin);
        // app.add_system(layer_system);
        // app.add_startup_system(setup);
        // app.add_system(tile_system::tile_system);
    }
}
// fn setup(mut commands: Commands) {
//     let tilemap_entity = commands.spawn_empty().id();
//     commands.spawn(TileLayerBundle {
//         id: TileLayerId(tilemap_entity),
//         ..Default::default()
//     });
// }
