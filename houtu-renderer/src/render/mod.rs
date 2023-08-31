use bevy::{
    core::FrameCount,
    math::{DMat4, DVec2, DVec3, Vec4Swizzles},
    prelude::*,
    reflect::List,
    render::renderer::RenderDevice,
    window::PrimaryWindow,
};
use houtu_jobs::JobSpawner;
use houtu_scene::{
    GeographicTilingScheme, Matrix4, TerrainMesh, TerrainQuantization, WebMercatorTilingScheme,
};
use rand::Rng;

use crate::xyz_imagery_provider::XYZImageryProvider;

use self::{
    terrain_render_pipeline::TerrainMaterialPlugin, terrian_material::TerrainMeshMaterial,
    wrap_terrain_mesh::WrapTerrainMesh,
};

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
mod terrain_render_pipeline;
mod terrian_material;
mod wrap_terrain_mesh;
use super::{
    camera::GlobeCamera,
    wmts_imagery_provider::{WMTSImageryProvider, WMTSImageryProviderOptions},
};
/// 负责渲染quadtree调度后生成的瓦片
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MaterialPlugin::<terrian_material::TerrainMeshMaterial>::default());
        app.add_systems(Startup, setup);
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
    let xyz = XYZImageryProvider {
        // url: "https://maps.omniscale.net/v2/houtuearth-4781e785/style.default/{z}/{x}/{y}.png",
        url:"icon.png",
        ..Default::default()
    };
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
    mut globe_camera_query: Query<&mut GlobeCamera>,
) {
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
    //清除已经渲染但不需要渲染的瓦片
    for (entity, tile_rendered) in rendered_query.iter() {
        // if !primitive.tiles_to_render.contains(&tile_rendered.0) {
        //     commands.entity(entity).despawn();
        //     let tile = primitive.storage.get_mut(&tile_rendered.0).unwrap();
        //     tile.entity = None;
        // }
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
                    .and_then(|x| imagery_storage.get(x));//
                let imagery_layer_opt =
                    imagery_opt.and_then(|x| imagery_layer_storage.get(&x.key.layer_id));
                if let (None, None) = (imagery_opt, imagery_layer_opt) {
                    continue;
                }
                let imagery = imagery_opt.unwrap();
                imagery_key_list.push(&imagery.key.key);
                let imagery_layer = imagery_layer_opt.unwrap();//
                if imagery_layer.alpha == 0.0 {
                    continue;
                }
                let texture = match imagery.texture.as_ref() {//
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

            let data = tile.data.get_cloned_terrain_data();
            let surface_tile = data.lock().unwrap();
            let terrain_mesh = surface_tile.get_mesh().unwrap();
            let mvp = get_mvp(&mut globe_camera, &terrain_mesh.center);
            let material = TerrainMeshMaterial {
                quantization_bits12: terrain_mesh.encoding.quantization
                    == TerrainQuantization::BITS12,
                has_web_mercator_t: terrain_mesh.encoding.has_web_mercator_t,
                textures: texture_list,
                translation_and_scale: translation_and_scale_list,
                coordinate_rectangle: coordinate_rectangle_list,
                web_mercator_t: use_web_mercator_t_list,
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
                min_max_height: Vec2::new(
                    terrain_mesh.encoding.minimum_height as f32,
                    terrain_mesh.encoding.maximum_height as f32,
                ),
                scale_and_bias: terrain_mesh.encoding.matrix.as_mat4(),
                center_3d: terrain_mesh.center.as_vec3(),
                mvp: mvp.as_mat4(),
            };
            let wrap_terrain_mesh = WrapTerrainMesh(terrain_mesh);
            let rendered_entity = commands.spawn((
                MaterialMeshBundle {
                    mesh: meshes.add(wrap_terrain_mesh.into()),
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
fn get_mvp(globe_camera: &mut GlobeCamera, rtc: &DVec3) -> DMat4 {
    let view_matrix = globe_camera.get_view_matrix();
    let projection_matrix = globe_camera.frustum.get_projection_matrix().clone();
    // let center_eye = view_matrix.multiply_by_point(rtc);
    let mut mvp = view_matrix.clone();
    // mvp.set_translation(&center_eye);
    mvp = projection_matrix * mvp;
    return mvp;
}
