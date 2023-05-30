use bevy::prelude::*;
use houtu_scene::IndicesAndEdgesCache;
mod layer;
mod storage;
mod terrian_material;
mod tile;
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<terrian_material::TerrainMeshMaterial>::default());
        app.insert_resource(IndicesAndEdgesCache::new());
    }
}
