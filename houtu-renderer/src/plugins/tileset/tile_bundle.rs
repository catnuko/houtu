use bevy::{math::UVec3, prelude::*};
use houtu_scene::{GeographicTilingScheme, HeightmapTerrainData, TerrainMesh};
use rand::Rng;

use super::{
    terrian_material::TerrainMeshMaterial, tile_id::TileId, tile_key::TileKey,
    tile_layer_id::TileLayerId, tile_state::TileState, tile_storage::TileStorage,
};

/// A texture index into the atlas or texture array for a single tile. Indices in an atlas are horizontal based.
#[derive(Component, Reflect, Default, Clone, Copy, Debug, Hash)]
#[reflect(Component)]
pub struct TileTextureIndex(pub u32);
#[derive(Component, Default, Clone, Debug)]
pub struct TerrainMeshWrap(pub Option<TerrainMesh>);
#[derive(Component, Debug, Default, Clone)]
pub struct TileMark;

#[derive(Component, Debug, Default, Clone)]
pub struct ToRender;

#[derive(Component, Debug, Clone)]
pub enum LoadPriority {
    None,
    High,
    Medium,
    Low,
}
impl Default for LoadPriority {
    fn default() -> Self {
        Self::None
    }
}
/// 瓦片
///
/// 生成一个瓦片所需的数据
#[derive(Bundle, Clone, Debug)]
pub struct TileBundle {
    /// 标志位Tile
    pub mark: TileMark,
    /// TileBundle的唯一值，由x,y,z组成
    pub key: TileKey,
    pub visible: Visibility,
    /// 所属的TileBundle的id
    pub tile_layer_id: TileLayerId,
    /// 生成TileBundle所需的网格体
    pub terrain_mesh: TerrainMeshWrap,
    /// TileBundle的状态
    pub state: TileState,
    /// 加载优先级
    pub load_priority: LoadPriority,
}
impl TileBundle {
    pub fn new(
        tile_layer_id: TileLayerId,
        tile_key: TileKey,
        terrain_mesh: TerrainMeshWrap,
    ) -> Self {
        Self {
            mark: TileMark,
            key: tile_key,
            visible: Visibility::Visible,
            tile_layer_id,
            terrain_mesh,
            state: TileState::Start,
            load_priority: LoadPriority::default(),
        }
    }
    pub fn set_terrain_mesh(&mut self, terrain_mesh: TerrainMeshWrap) {
        self.terrain_mesh = terrain_mesh;
    }
}
