use bevy::{math::UVec3, prelude::*};
use houtu_scene::{GeographicTilingScheme, HeightmapTerrainData, TerrainMesh};
use rand::Rng;

use super::{layer::TileLayerId, storage::TileStorage, terrian_material::TerrainMeshMaterial};
#[derive(
    Component, Reflect, FromReflect, Default, Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd,
)]
#[reflect(Component)]
pub struct TileKey {
    x: u32,
    y: u32,
    level: u32,
}
impl TileKey {
    pub fn new(x: u32, y: u32, level: u32) -> Self {
        Self { x, y, level }
    }

    pub fn get_id(&self) -> String {
        format!("{}_{}_{}", self.x, self.y, self.level)
    }
}

impl From<TileKey> for UVec3 {
    fn from(pos: TileKey) -> Self {
        UVec3::new(pos.x, pos.y, pos.level)
    }
}

impl From<&TileKey> for UVec3 {
    fn from(pos: &TileKey) -> Self {
        UVec3::new(pos.x, pos.y, pos.level)
    }
}

impl From<UVec3> for TileKey {
    fn from(v: UVec3) -> Self {
        Self {
            x: v.x,
            y: v.y,
            level: v.z,
        }
    }
}
/// Hides or shows a tile based on the boolean. Default: True
#[derive(Component, Reflect, Clone, Copy, Debug, Hash)]
#[reflect(Component)]
pub struct TileVisible(pub bool);

impl Default for TileVisible {
    fn default() -> Self {
        Self(true)
    }
}
/// A texture index into the atlas or texture array for a single tile. Indices in an atlas are horizontal based.
#[derive(Component, Reflect, Default, Clone, Copy, Debug, Hash)]
#[reflect(Component)]
pub struct TileTextureIndex(pub u32);
#[derive(Component, Default, Clone, Debug)]
pub struct TerrainMeshWrap(pub Option<TerrainMesh>);
#[derive(Component, Debug, Default, Clone)]
pub struct TileMark;
#[derive(Component, Debug, Clone)]
pub enum TileState {
    Start,
    Creationg,
    Created,
    Rendered,
}
impl Default for TileState {
    fn default() -> Self {
        Self::Start
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
}
pub fn tile_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_materials: ResMut<Assets<TerrainMeshMaterial>>,
    mut query: Query<(&mut TerrainMeshWrap, &mut TileState), With<TileMark>>,
    asset_server: Res<AssetServer>,
) {
    for (mut terrain_mesh_warp, mut state) in &mut query {
        if let None = terrain_mesh_warp.0 {
            bevy::log::info!("没有terrain_mesh")
        } else {
            match *state {
                TileState::Start => {
                    let mut rng = rand::thread_rng();
                    let r: f32 = rng.gen();
                    let g: f32 = rng.gen();
                    let b: f32 = rng.gen();
                    commands.spawn((MaterialMeshBundle {
                        mesh: meshes.add(terrain_mesh_warp.0.as_ref().unwrap().into()),
                        material: terrain_materials.add(TerrainMeshMaterial {
                            image: Some(asset_server.load("icon.png")),
                            color: Color::rgba(r, g, b, 1.0),
                        }),
                        ..Default::default()
                    },));
                    *state = TileState::Rendered;
                }
                _ => {}
            }
        }
    }
}

pub fn create_tile_system() {}
