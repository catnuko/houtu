// use bevy::{math::UVec3, prelude::*};
// use houtu_scene::{
//     GeographicTilingScheme, HeightmapTerrainData, IndicesAndEdgesCache, TilingScheme,
// };

// use super::{
//     tile_bundle::{TileBundle, TileMark},
//     tile_key::TileKey,
//     tile_layer_bundle::TileLayerMark,
//     tile_layer_id::TileLayerId,
//     tile_layer_state::TileLayerState,
//     tile_state::TileState,
//     tile_storage::TileStorage,
// };
// /// 推进LayerBundle的状态
// ///
// /// 任务：
// /// 1. 计算生成TileBundle所需的顶点数据
// /// 2. 生成TileBundle
// /// 3. 维护瓦片的四叉树
// pub fn layer_system(
//     mut command: Commands,
//     mut query: Query<
//         (
//             &mut TileLayerState,
//             &GeographicTilingScheme,
//             &TileLayerId,
//             &mut TileStorage,
//         ),
//         With<TileLayerMark>,
//     >,
//     mut indicesAndEdgesCache: ResMut<IndicesAndEdgesCache>,
// ) {
//     for (mut state, tiling_scheme, tile_layer_id, mut tile_storage) in &mut query {
//         match *state {
//             TileLayerState::Start => {
//                 let numberOfLevelZeroTilesX = tiling_scheme.get_number_of_x_tiles_at_level(0);
//                 let numberOfLevelZeroTilesY = tiling_scheme.get_number_of_y_tiles_at_level(0);
//                 for y in 0..numberOfLevelZeroTilesY {
//                     for x in 0..numberOfLevelZeroTilesX {
//                         let width = 16;
//                         let height = 16;
//                         let buffer: Vec<f64> = vec![0.; (width * height) as usize];
//                         let mut height_data = HeightmapTerrainData::new(
//                             buffer, width, height, None, None, None, None, None, None, None,
//                         );
//                         let terrain_mesh = height_data._createMeshSync::<GeographicTilingScheme>(
//                             &tiling_scheme,
//                             x,
//                             y,
//                             0,
//                             None,
//                             None,
//                             &mut indicesAndEdgesCache,
//                         );
//                         let tile_key = TileKey::new(x, y, 0);
//                         let entity = command
//                             .spawn(TileBundle {
//                                 key: tile_key.clone(),
//                                 visible: Visibility::Visible,
//                                 tile_layer_id: tile_layer_id.clone(),
//                                 terrain_mesh: super::tile_bundle::TerrainMeshWrap(Some(
//                                     terrain_mesh,
//                                 )),
//                                 mark: TileMark,
//                                 state: TileState::Start,
//                                 load_priority:super::tile_bundle::LoadPriority::None
//                             })
//                             .id();
//                         tile_storage.set(&tile_key, entity);
//                     }
//                 }
//                 *state = TileLayerState::Loading;
//             }
//             TileLayerState::Loading => {}
//         }
//     }
// }
