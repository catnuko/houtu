use std::collections::HashMap;

use self::terrain_bundle::TerrainBundle;
use crate::wmts_imagery_provider::WMTSImageryProvider;
use crate::xyz_imagery_provider::XYZImageryProvider;
use bevy::asset::{embedded_asset, load_internal_asset};
use bevy::prelude::*;
use houtu_scene::{GeographicTilingScheme, TilingScheme};

use super::quadtree::{
    globe_surface_tile::process_terrain_state_machine_system, imagery_layer::ImageryLayer,
    imagery_layer_storage::ImageryLayerStorage, imagery_storage::ImageryStorage,
    quadtree_primitive::QuadtreePrimitive, quadtree_tile::QuadtreeTileLoadState, tile_key::TileKey,
};
mod terrain_bundle;
mod terrain_pipeline;
mod terrain_plugin;
mod terrian_material;
mod wrap_terrain_mesh;
use super::camera::GlobeCamera;
pub const TERRAIN_MATERIAN_SHADER_HANDLE: Handle<Shader> = Handle::weak_from_u128(9275037169799534);
/// 负责渲染quadtree调度后生成的瓦片
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(terrain_plugin::Plugin);
        app.add_systems(Startup, setup);
        // embedded_asset!(app, "terrain_material.wgsl");
        load_internal_asset!(
            app,
            TERRAIN_MATERIAN_SHADER_HANDLE,
            "terrain_material.wgsl",
            Shader::from_wgsl
        );
        app.add_systems(
            Update,
            real_render_system.after(process_terrain_state_machine_system),
        );
    }
}
fn setup(
    mut imagery_layer_storage: ResMut<ImageryLayerStorage>,
    mut imagery_storage: ResMut<ImageryStorage>,
) {
    let provider = XYZImageryProvider {
        // url: "https://maps.omniscale.net/v2/houtuearth-4781e785/style.default/{z}/{x}/{y}.png",
        // url: "http://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png",
        // subdomains: Some(vec!["a", "b", "c"]),
        url: "icon.png",
        // url: "https://api.maptiler.com/maps/basic-v2/256/{z}/{x}/{y}.png?key=Modv7lN1eXX1gmlqW0wY",
        ..Default::default()
    };
    // let tiling_scheme = GeographicTilingScheme::default();
    // let rectangle = tiling_scheme.get_rectangle().clone();
    // let provider = WMTSImageryProvider {
    //     name: "test",
    //     url: "https://{s}.tianditu.gov.cn/img_c/wmts",
    //     layer: "img_c",
    //     style: "default",
    //     format: "tiles",
    //     tile_matrix_set_id: "w",
    //     subdomains: Some(vec!["t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7"]),
    //     tiling_scheme: Box::new(tiling_scheme),
    //     minimum_level: 0,
    //     maximum_level: 17,
    //     rectangle: rectangle,
    //     tile_width: 256,
    //     tile_height: 256,
    //     tile_matrix_labels: None,
    //     params: Some(vec![("tk", "b931d6faa76fc3fbe622bddd6522e57b")]),
    // };
    let mut imagery_layer = ImageryLayer::new(Box::new(provider), &mut imagery_storage);
    imagery_layer.is_base_layer = true;
    imagery_layer_storage.add(imagery_layer)
}

#[derive(Component)]
pub struct TileRendered(TileKey);
fn real_render_system(
    mut primitive: ResMut<QuadtreePrimitive>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    rendered_query: Query<(Entity, &TileRendered)>,
    mut images: ResMut<Assets<Image>>,
    imagery_storage: Res<ImageryStorage>,
    imagery_layer_storage: Res<ImageryLayerStorage>,
    mut globe_camera_query: Query<&mut GlobeCamera>,
) {
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
    //清除已经渲染但不需要渲染的瓦片
    for (entity, tile_rendered) in rendered_query.iter() {
        commands.entity(entity).despawn();
        let tile = primitive.storage.get_mut(&tile_rendered.0).unwrap();
        tile.entity = None;
    }
    let mut tile_key_list = vec![];
    primitive
        .tiles_to_render
        .iter()
        .for_each(|x| tile_key_list.push(x.clone()));
    for key in tile_key_list.iter() {
        let tile = primitive.storage.get_mut(key).unwrap();
        if tile.state == QuadtreeTileLoadState::DONE
            && tile.entity.is_none()
            && tile.data.imagery.len() > 0
        {
            let terrain_bundle = TerrainBundle::from_quadtree_tile(
                tile,
                &mut globe_camera,
                &imagery_storage,
                &imagery_layer_storage,
                &mut meshes,
                &mut images,
            );
            let rendered_entity = commands.spawn(terrain_bundle);
            let entity = rendered_entity.id();
            tile.entity = Some(entity);
        }
    }
}
