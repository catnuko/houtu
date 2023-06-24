// use bevy::{math::UVec3, prelude::*};
// use rand::Rng;

// use super::{terrian_material::TerrainMeshMaterial, tile_state::TileState};

// pub fn tile_system(
//     mut commands: Commands,
//     mut meshes: ResMut<Assets<Mesh>>,
//     mut terrain_materials: ResMut<Assets<TerrainMeshMaterial>>,
//     mut query: Query<(&mut TerrainMeshWrap, &mut TileState), With<TileMark>>,
//     asset_server: Res<AssetServer>,
// ) {
//     for (mut terrain_mesh_warp, mut state) in &mut query {
//         if let None = terrain_mesh_warp.0 {
//             bevy::log::info!("没有terrain_mesh")
//         } else {
//             match *state {
//                 TileState::Start => {
//                     let mut rng = rand::thread_rng();
//                     let r: f32 = rng.gen();
//                     let g: f32 = rng.gen();
//                     let b: f32 = rng.gen();
//                     commands.spawn((MaterialMeshBundle {
//                         mesh: meshes.add(terrain_mesh_warp.0.as_ref().unwrap().into()),
//                         material: terrain_materials.add(TerrainMeshMaterial {
//                             image: Some(asset_server.load("icon.png")),
//                             color: Color::rgba(r, g, b, 1.0),
//                         }),
//                         ..Default::default()
//                     },));
//                     *state = TileState::Done;
//                 }
//                 _ => {}
//             }
//         }
//         process_state_machine(&mut state);
//     }
// }

// fn process_state_machine(state: &mut TileState) {
//     match state {
//         TileState::Start => {}
//         _ => {}
//     }
// }
