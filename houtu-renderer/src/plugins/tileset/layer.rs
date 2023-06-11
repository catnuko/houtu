use bevy::{math::UVec3, prelude::*};
use houtu_scene::{
    GeographicTilingScheme, HeightmapTerrainData, IndicesAndEdgesCache, TilingScheme,
};

use super::{
    storage::TileStorage,
    tile::{TileBundle, TileKey},
};

#[derive(Component, Reflect, Clone, Copy, Debug, Hash)]
#[reflect(Component)]
pub struct TileLayerId(pub Entity);

impl Default for TileLayerId {
    fn default() -> Self {
        Self(Entity::from_raw(0))
    }
}
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
#[derive(Component, Debug, Clone)]
pub enum TileLayerState {
    Start,
    Loading,
}
impl Default for TileLayerState {
    fn default() -> Self {
        Self::Start
    }
}
/// The default tilemap bundle. All of the components within are required.
#[derive(Bundle, Debug, Clone)]
pub struct TileLayerBundle {
    pub mark: TileLayerMark,
    // pub texture: TileLayerTexture,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
    pub tile_storage: TileStorage,
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
        }
    }
}
/// 推进LayerBundle的状态
///
/// 任务：
/// 1. 计算生成TileBundle所需的顶点数据
/// 2. 生成TileBundle
/// 3. 维护瓦片的四叉树
pub fn layer_system(
    mut command: Commands,
    mut query: Query<
        (
            &mut TileLayerState,
            &GeographicTilingScheme,
            &TileLayerId,
            &mut TileStorage,
        ),
        With<TileLayerMark>,
    >,
    mut indicesAndEdgesCache: ResMut<IndicesAndEdgesCache>,
) {
    for (mut state, tiling_scheme, tile_layer_id, mut tile_storage) in &mut query {
        match *state {
            TileLayerState::Start => {
                let numberOfLevelZeroTilesX = tiling_scheme.get_number_of_x_tiles_at_level(0);
                let numberOfLevelZeroTilesY = tiling_scheme.get_number_of_y_tiles_at_level(0);
                for y in 0..numberOfLevelZeroTilesY {
                    for x in 0..numberOfLevelZeroTilesX {
                        let width = 16;
                        let height = 16;
                        let buffer: Vec<f64> = vec![0.; (width * height) as usize];
                        let mut height_data = HeightmapTerrainData::new(
                            buffer, width, height, None, None, None, None, None, None, None,
                        );
                        let terrain_mesh = height_data._createMeshSync::<GeographicTilingScheme>(
                            &tiling_scheme,
                            x,
                            y,
                            0,
                            None,
                            None,
                            &mut indicesAndEdgesCache,
                        );
                        let tile_key = TileKey::new(x, y, 0);
                        let entity = command
                            .spawn(TileBundle {
                                key: tile_key.clone(),
                                visible: Visibility::Visible,
                                tile_layer_id: tile_layer_id.clone(),
                                terrain_mesh: super::tile::TerrainMeshWrap(Some(terrain_mesh)),
                                mark: super::tile::TileMark,
                                state: super::tile::TileState::Start,
                            })
                            .id();
                        tile_storage.set(&tile_key, entity);
                    }
                }
                *state = TileLayerState::Loading;
            }
            TileLayerState::Loading => {}
        }
    }
}
