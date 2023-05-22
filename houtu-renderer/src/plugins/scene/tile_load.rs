use bevy::prelude::*;
use houtu_scene::TileKey;

use crate::plugins::wmts::WMTS;

use super::TerrainMeshMaterial;

#[derive(Component)]
pub enum TileState {
    START = 0,
    LOADING = 1,
    READY = 2,
}
pub fn upd_level(mut commands: Commands) {}
pub fn upd_tile_load(
    mut commands: Commands,
    mut terrain_materials: ResMut<Assets<TerrainMeshMaterial>>,
    mut query: Query<(
        &mut TileKey,
        &mut TileState,
        &mut Handle<TerrainMeshMaterial>,
    )>,
    asset_server: Res<AssetServer>,
    wmts: Res<WMTS>,
) {
    for (tile_key, mut tile_state, mut material) in query.iter() {
        match tile_state {
            TileState::START => {
                // let url = wmts.build_url(tile_key);
                let handler: Handle<Image> = asset_server.load("icon.png");
                material = &terrain_materials.add(TerrainMeshMaterial {
                    color: Color::WHITE,
                    image: Some(handler),
                });
                tile_state = &TileState::LOADING;
            }
            TileState::LOADING => {}
            TileState::READY => {}

            _ => {}
        }
    }
}
pub fn get_zoom_level(h: f64) -> u8 {
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
