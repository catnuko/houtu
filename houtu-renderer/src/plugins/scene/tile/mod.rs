use bevy::{math::DVec3, prelude::*, utils::Uuid};

mod create_tile_job;
mod layer_id;
mod terrian_material;
mod tile;
mod tile_key;
mod tile_layer;
mod tile_state;

use houtu_jobs::JobSpawner;
use houtu_scene::{Ellipsoid, GeographicTilingScheme, IndicesAndEdgesCache, TilingScheme};
use layer_id::*;
use rand::Rng;
use terrian_material::*;
use tile::*;
use tile_key::*;
use tile_layer::*;
use tile_state::*;

use crate::plugins::camera::GlobeMapCamera;

use self::create_tile_job::CreateTileJob;

pub struct TilePlugin;

impl Plugin for TilePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<TerrainMeshMaterial>::default())
            .register_type::<TerrainMeshMaterial>()
            .insert_resource(TileLayer::new(Some(GeographicTilingScheme::default())))
            .insert_resource(IndicesAndEdgesCache::new())
            .add_system(layer_system)
            .add_system(tile_state_system)
            .add_system(create_tile_job::handle_created_tile_system);
    }
    fn name(&self) -> &str {
        "houtu_tile_plugin"
    }
}
fn layer_system(
    mut tile_layer: ResMut<TileLayer>,
    mut commands: Commands,
    query: Query<&mut GlobeMapCamera>,
    mut job_spawner: houtu_jobs::JobSpawner,
) {
    match tile_layer.state {
        TileLayerState::Start => {
            for globe_map_camera in query.iter() {
                if let Some(position) = globe_map_camera.position_cartographic {
                    let level = get_zoom_level(position.height);
                    let num_of_x_tiles = tile_layer
                        .tiling_scheme
                        .get_number_of_x_tiles_at_level(level);
                    let num_of_y_tiles = tile_layer
                        .tiling_scheme
                        .get_number_of_y_tiles_at_level(level);
                    for y in 0..num_of_y_tiles {
                        for x in 0..num_of_x_tiles {
                            let width: u32 = 32;
                            let height: u32 = 32;
                            if !tile_layer.is_exist(x, y, level) {
                                let entity = commands
                                    .spawn(Tile::new(x, y, level, Some(width), Some(height)))
                                    .id();
                                tile_layer.add_tile(x, y, level, entity);
                            }
                        }
                    }
                }
            }
        }
    }
}
fn tile_state_system(
    mut tile_layer: ResMut<TileLayer>,
    mut query: Query<&mut Tile>,
    mut job_spawner: JobSpawner,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut terrain_materials: ResMut<Assets<TerrainMeshMaterial>>,
    mut standard_materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for mut tile in &mut query {
        match tile.state {
            TileState::START => {
                job_spawner.spawn(CreateTileJob {
                    x: tile.x,
                    y: tile.y,
                    level: tile.level,
                    width: tile.width,
                    height: tile.height,
                });
                tile.state = TileState::LOADING;
            }
            TileState::LOADING => {}
            TileState::READY => {
                let terrain_mesh = tile.terrain_mesh.as_ref().unwrap();
                let mesh = meshes.add(terrain_mesh.into());
                let mut rng = rand::thread_rng();
                let r: f32 = rng.gen();
                let g: f32 = rng.gen();
                let b: f32 = rng.gen();
                commands.spawn((
                    MaterialMeshBundle {
                        mesh: mesh,
                        material: terrain_materials.add(TerrainMeshMaterial {
                            color: Color::rgba(r, g, b, 1.0),
                            // image: asset_server.load("icon.png"),
                            // image: asset_server.load(format!("https://t5.tianditu.gov.cn/img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=vec&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={}&TILEROW={}&TILEMATRIX={}&tk=b931d6faa76fc3fbe622bddd6522e57b",x,y,level)),
                            // image: asset_server.load(format!("tile/{}/{}/{}.png", level, y, x,)),
                            image:Some( asset_server.load(format!(
                                "https://maps.omniscale.net/v2/houtu-b8084b0b/style.default/{}/{}/{}.png",
                                tile.level, tile.x, tile.y,
                            ))),
                            // image: None,
                        }),
                        // material: standard_materials.add(Color::rgba(r, g, b, 1.0).into()),
                        ..Default::default()
                    },
                    TileKey::new(tile.y, tile.x, tile.level),
                    // TileState::START,
                ));
            }
            _ => {}
        }
    }
}

pub fn get_zoom_level(h: f64) -> u32 {
    if (h <= 100.) {
        //0.01
        return 19;
    } else if (h <= 300.) {
        //0.02
        return 18;
    } else if (h <= 660.) {
        //0.05
        return 17;
    } else if (h <= 1300.) {
        //0.1
        return 16;
    } else if (h <= 2600.) {
        //0.2
        return 15;
    } else if (h <= 6400.) {
        //0.5
        return 14;
    } else if (h <= 13200.) {
        //1
        return 13;
    } else if (h <= 26000.) {
        //2
        return 12;
    } else if (h <= 67985.) {
        //5
        return 11;
    } else if (h <= 139780.) {
        //10
        return 10;
    } else if (h <= 250600.) {
        //20
        return 9;
    } else if (h <= 380000.) {
        //30
        return 8;
    } else if (h <= 640000.) {
        //50
        return 7;
    } else if (h <= 1280000.) {
        //100
        return 6;
    } else if (h <= 2600000.) {
        //200
        return 5;
    } else if (h <= 6100000.) {
        //500
        return 4;
    } else if (h <= 11900000.) {
        //1000
        return 3;
    } else {
        return 2;
    }
}
