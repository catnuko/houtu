use bevy::{math::UVec3, prelude::*};
use houtu_scene::{
    GeographicTilingScheme, HeightmapTerrainData, IndicesAndEdgesCache, TilingScheme,
};

use super::{
    tile_bundle::{TerrainMeshWrap, TileBundle},
    tile_key::TileKey,
    tile_layer_id::TileLayerId,
    tile_layer_state::TileLayerState,
    tile_storage::TileStorage,
};

#[derive(Component, Reflect, Clone, Debug, Hash, PartialEq, Eq)]
#[reflect(Component)]
pub struct TileLayerTexture(Handle<Image>);
impl Default for TileLayerTexture {
    fn default() -> Self {
        TileLayerTexture(Default::default())
    }
}

#[derive(Component, Debug, Default, Clone)]
pub struct TileLayerMark;

/// The default tilemap bundle. All of the components within are required.
#[derive(Bundle, Debug, Clone)]
pub struct TileLayerBundle {
    pub mark: TileLayerMark,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    tile_storage: TileStorage,
    quadtree: TileTree,
    pub tiling_scheme: GeographicTilingScheme,
    pub state: TileLayerState,
    pub id: TileLayerId,
}
impl Default for TileLayerBundle {
    fn default() -> Self {
        Self {
            mark: TileLayerMark,
            tile_storage: TileStorage::empty(),
            visibility: Visibility::Visible,
            computed_visibility: ComputedVisibility::default(),
            tiling_scheme: GeographicTilingScheme::default(),
            state: TileLayerState::Start,
            id: TileLayerId(Entity::PLACEHOLDER),
            quadtree: TileTree::default(),
        }
    }
}
impl TileLayerBundle {
    fn make_tile(&mut self, tile_key: TileKey) -> TileBundle {
        return TileBundle::new(self.id, tile_key, TerrainMeshWrap(None));
    }
    fn add_tile(&mut self, tile_key: TileKey, entity: Entity) {
        return self.tile_storage.set(&tile_key, entity);
    }
    fn get_tile(&self, tile_key: TileKey) -> Option<Entity> {
        return self.tile_storage[tile_key];
    }
}
