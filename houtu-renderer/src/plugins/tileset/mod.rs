use bevy::prelude::*;
use houtu_scene::IndicesAndEdgesCache;

use self::{
    tile_layer_bundle::TileLayerBundle, tile_layer_id::TileLayerId, tile_layer_system::layer_system,
};
mod globe_surface_tile;
mod quadtree_tile;
mod terrian_material;
mod tile_bundle;
mod tile_datasource;
mod tile_id;
mod tile_key;
mod tile_layer_bundle;
mod tile_layer_id;
mod tile_layer_state;
mod tile_layer_system;
mod tile_quad_tree;
mod tile_replacement_queue;
mod tile_state;
mod tile_storage;
mod tile_system;
mod tile_tree;
pub use tile_key::TileKey;
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<terrian_material::TerrainMeshMaterial>::default());
        app.insert_resource(IndicesAndEdgesCache::new());
        app.add_system(layer_system);
        app.add_startup_system(setup);
        app.add_system(tile_system::tile_system);
    }
}
fn setup(mut commands: Commands) {
    let tilemap_entity = commands.spawn_empty().id();
    commands.spawn(TileLayerBundle {
        id: TileLayerId(tilemap_entity),
        ..Default::default()
    });
}
