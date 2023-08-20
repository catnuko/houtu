use bevy::{
    core::FrameCount,
    math::{DVec2, DVec3, Vec4Swizzles},
    prelude::*,
    reflect::List,
    render::renderer::RenderDevice,
    window::PrimaryWindow,
};
use houtu_jobs::JobSpawner;
use houtu_scene::{GeographicTilingScheme, TerrainMesh, WebMercatorTilingScheme};
use rand::Rng;

use crate::xyz_imagery_provider::XYZImageryProvider;

use self::{terrain_render_pipeline::TerrainMaterialPlugin, terrian_material::TerrainMeshMaterial};

use super::quadtree::{
    globe_surface_tile::process_terrain_state_machine_system,
    imagery_layer::ImageryLayer,
    imagery_layer_storage::ImageryLayerStorage,
    imagery_storage::ImageryStorage,
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    quadtree_primitive::QuadtreePrimitive,
    quadtree_tile::QuadtreeTileLoadState,
    reproject_texture::ReprojectTextureTaskQueue,
    tile_key::TileKey,
    traversal_details::{AllTraversalQuadDetails, RootTraversalDetails},
};
mod terrain_bundle;
mod terrain_render_pipeline;
mod terrian_material;
use super::{
    camera::GlobeCamera,
    wmts_imagery_provider::{WMTSImageryProvider, WMTSImageryProviderOptions},
};
pub use terrian_material::ATTRIBUTE_WEB_MERCATOR_T;
/// 负责渲染quadtree调度后生成的瓦片
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(MaterialPlugin::<terrian_material::TerrainMeshMaterial>::default());
        // app.add_plugin(TerrainMaterialPlugin::<terrian_material::TerrainMeshMaterial>::default());
        app.add_startup_system(setup);
        app.add_system(real_render_system.after(process_terrain_state_machine_system));
    }
}
fn setup(
    mut imagery_layer_storage: ResMut<ImageryLayerStorage>,
    mut imagery_storage: ResMut<ImageryStorage>,
) {
    let xyz = XYZImageryProvider::new(Box::new(WebMercatorTilingScheme::default()));
    let mut imagery_layer = ImageryLayer::new(Box::new(xyz), &mut imagery_storage);
    imagery_layer.is_base_layer = true;
    imagery_layer_storage.add(imagery_layer)
}

#[derive(Component)]
pub struct TileRendered(TileKey);
fn real_render_system(
    mut primitive: ResMut<QuadtreePrimitive>,
    mut commands: Commands,
    mut terrain_materials: ResMut<Assets<TerrainMeshMaterial>>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    rendered_query: Query<(Entity, &TileRendered)>,
    _images: ResMut<Assets<Image>>,
    imagery_storage: Res<ImageryStorage>,
    imagery_layer_storage: Res<ImageryLayerStorage>,
) {
    //清除已经渲染但不需要渲染的瓦片
    for (entity, tile_rendered) in rendered_query.iter() {
        if !primitive.tiles_to_render.contains(&tile_rendered.0) {
            commands.entity(entity).despawn();
            let tile = primitive.storage.get_mut(&tile_rendered.0).unwrap();
            tile.entity = None;
        }
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
            // info!("render tile key={:?}", key);
            // info!("{:?} length is {}", tile.key, tile.data.imagery.len());
            let mut texture_list = vec![];
            let mut translation_and_scale_list = vec![];
            let mut coordinate_rectangle_list = vec![];
            let mut use_web_mercator_t_list = vec![];
            let mut alpha_list = vec![];
            let mut night_alpha_list = vec![];
            let mut day_alpha_list = vec![];
            let mut brightness_list = vec![];
            let mut contrast_list = vec![];
            let mut hue_list = vec![];
            let mut saturation_list = vec![];
            let mut one_over_gamma_list = vec![];
            let mut imagery_key_list = vec![];
            for tile_imagery in tile.data.imagery.iter_mut() {
                let imagery_opt = tile_imagery
                    .ready_imagery
                    .as_ref()
                    .and_then(|x| imagery_storage.get(x));
                let imagery_layer_opt =
                    imagery_opt.and_then(|x| imagery_layer_storage.get(&x.key.layer_id));
                if let (None, None) = (imagery_opt, imagery_layer_opt) {
                    continue;
                }
                let imagery = imagery_opt.unwrap();
                imagery_key_list.push(&imagery.key.key);
                let imagery_layer = imagery_layer_opt.unwrap();
                if imagery_layer.alpha == 0.0 {
                    continue;
                }
                let texture = match imagery.texture.as_ref() {
                    Some(v) => v.clone(),
                    None => panic!("readyImagery is not actually ready!"),
                };
                if tile_imagery.texture_translation_and_scale.is_none() {
                    tile_imagery.texture_translation_and_scale =
                        Some(imagery_layer.calculate_texture_translation_and_scale(
                            tile.rectangle.clone(),
                            tile_imagery,
                            imagery.rectangle.clone(),
                        ))
                }
                texture_list.push(texture);
                translation_and_scale_list.push(
                    tile_imagery
                        .texture_translation_and_scale
                        .unwrap()
                        .as_vec4(),
                );
                coordinate_rectangle_list
                    .push(tile_imagery.texture_coordinate_rectangle.unwrap().as_vec4());
                use_web_mercator_t_list.push(if tile_imagery.use_web_mercator_t {
                    1.0
                } else {
                    0.0
                });
                alpha_list.push(imagery_layer.alpha as f32);
                night_alpha_list.push(imagery_layer.night_alpha as f32);
                day_alpha_list.push(imagery_layer.day_alpha as f32);
                brightness_list.push(imagery_layer.brightness as f32);
                contrast_list.push(imagery_layer.contrast as f32);
                hue_list.push(imagery_layer.hue as f32);
                saturation_list.push(imagery_layer.saturation as f32);
                one_over_gamma_list.push(imagery_layer.gamma as f32);
            }
            let material = TerrainMeshMaterial {
                textures: texture_list,
                translation_and_scale: translation_and_scale_list,
                coordinate_rectangle: coordinate_rectangle_list,
                use_web_mercator_t: use_web_mercator_t_list,
                alpha: alpha_list,
                night_alpha: night_alpha_list,
                day_alpha: day_alpha_list,
                brightness: brightness_list,
                contrast: contrast_list,
                hue: hue_list,
                saturation: saturation_list,
                one_over_gamma: one_over_gamma_list,
                texture_width: 256,
                texture_height: 256,
            };
            let data = tile.data.get_cloned_terrain_data();
            let surface_tile = data.lock().unwrap();
            let terrain_mesh = surface_tile.get_mesh().unwrap();
            debug_terrain_material(&terrain_mesh, &material, key, imagery_key_list);
            let rendered_entity = commands.spawn((
                MaterialMeshBundle {
                    mesh: meshes.add(terrain_mesh.into()),
                    material: terrain_materials.add(material),
                    ..Default::default()
                },
                TileRendered(tile.key),
            ));
            let entity = rendered_entity.id();
            tile.entity = Some(entity);
        }
    }
}
fn debug_terrain_material(
    mesh: &TerrainMesh,
    material: &TerrainMeshMaterial,
    quad_tile_key: &TileKey,
    imagery_key_list: Vec<&TileKey>,
) {
    // for (i, _) in mesh.positions.iter().enumerate() {
    //     let uv = mesh.uvs[i].as_vec2();
    //     let web_mercator_t = mesh.web_mecator_t[i] as f32;
    //     let texture_coordinates =
    //         Vec3::new(uv.x, uv.y, web_mercator_t).clamp(Vec3::ZERO, Vec3::ONE);
    //     for (texture_index, _) in material.textures.iter().enumerate() {
    //         let translation_and_scale = material.translation_and_scale[texture_index];
    //         let translation = translation_and_scale.xy();
    //         let scale = translation_and_scale.zw();
    //         let use_web_mercator_t = material.use_web_mercator_t[texture_index];
    //         let tile_texture_coordinates = if use_web_mercator_t == 1.0 {
    //             Vec2::new(texture_coordinates.x, texture_coordinates.z)
    //         } else {
    //             Vec2::new(texture_coordinates.x, texture_coordinates.y)
    //         };
    //         let texture_coordinates = tile_texture_coordinates * scale + translation;
    //         info!("uv is {:?}", texture_coordinates);
    //     }
    // }
    for (texture_index, _) in material.textures.iter().enumerate() {
        let translation_and_scale = material.translation_and_scale[texture_index];
        let translation = translation_and_scale.xy();
        let scale = translation_and_scale.zw();
        let use_web_mercator_t = material.use_web_mercator_t[texture_index];
        let texture_coordinate_rectangle = material.coordinate_rectangle[texture_index];
        // let tile_texture_coordinates = if use_web_mercator_t == 1.0 {
        //     Vec2::new(texture_coordinates.x, texture_coordinates.z)
        // } else {
        //     Vec2::new(texture_coordinates.x, texture_coordinates.y)
        // };
        // let texture_coordinates = tile_texture_coordinates * scale + translation;
        // info!("uv is {:?}", texture_coordinates);
        info!(
            "{},{:?},{:?},{:?}",
            use_web_mercator_t, imagery_key_list[texture_index], translation_and_scale,texture_coordinate_rectangle
        );
    }
}
