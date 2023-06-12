use bevy::prelude::*;
use houtu_scene::IndicesAndEdgesCache;

use self::{
    layer::{layer_system, TileLayerBundle, TileLayerId},
    tile::tile_system,
};

use super::cnquadtree::TerrainQuadtree;
mod layer;
mod storage;
mod terrian_material;
mod tile;
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<terrian_material::TerrainMeshMaterial>::default());
        app.insert_resource(IndicesAndEdgesCache::new());
        app.insert_resource(TerrainQuadtree);
        app.add_system(layer_system);
        app.add_startup_system(setup);
        app.add_system(tile_system);
    }
}
fn setup(mut commands: Commands) {
    let tilemap_entity = commands.spawn_empty().id();
    commands.spawn(TileLayerBundle {
        id: TileLayerId(tilemap_entity),
        ..Default::default()
    });
}
