use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use houtu_scene::IndicesAndEdgesCache;

use self::tile_layer_id::TileLayerId;
mod create_terrain_mesh_job;
mod ellipsoid_terrain_provider;
mod globe_surface_tile;
mod imagery;
mod imagery_layer;
mod imagery_layer_collection;
mod label;
mod quadtree_tile;
mod reproject_texture;
mod terrian_material;
mod tile_datasource;
mod tile_id;
mod tile_key;
mod tile_layer_bundle;
mod tile_layer_id;
mod tile_layer_state;
mod tile_layer_system;
mod tile_quad_tree;
mod tile_replacement_queue;
mod tile_selection_result;
mod tile_state;
mod tile_system;
mod unsample_job;
pub use tile_key::TileKey;
#[derive(Resource)]
pub struct IndicesAndEdgesCacheArc(pub Arc<Mutex<IndicesAndEdgesCache>>);
impl IndicesAndEdgesCacheArc {
    fn new() -> Self {
        IndicesAndEdgesCacheArc(Arc::new(Mutex::new(IndicesAndEdgesCache::new())))
    }
    fn get_cloned_cache(&self) -> Arc<Mutex<IndicesAndEdgesCache>> {
        return self.0.clone();
    }
}
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(label::Plugin);
        app.add_plugin(MaterialPlugin::<terrian_material::TerrainMeshMaterial>::default());
        app.insert_resource(IndicesAndEdgesCacheArc::new());
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
