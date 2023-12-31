use bevy::{
    asset::load_internal_asset,
    pbr::{
        wireframe::{Wireframe, WireframeColor, WireframeConfig},
        MaterialPipeline, MaterialPipelineKey,
    },
    prelude::*,
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, RenderPipelineDescriptor, ShaderRef, SpecializedMeshPipelineError,
        },
        view::NoFrustumCulling,
    },
};
use bevy_reflect_derive::TypeUuid;
use houtu_scene::{Rectangle, TerrainQuantization};
use rand::Rng;
use wgpu::PolygonMode;

use crate::{
    camera::GlobeCamera,
    quadtree::tile_key::TileKey,
    render::{
        wrap_terrain_mesh::WrapTerrainMesh, TerrainAttachment, TerrainBundle, TerrainConfig,
        TileRendered,
    },
};

use super::{
    height_map_terrain_data::HeightmapTerrainDataCom,
    imagery_layer::{ImageryLayer, ImageryProviderCom},
    quadtree_tile::{QuadtreeTile, Renderable, TileVisibility},
    tile_imagery::TileImageryVec,
};
pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (render_system, update_uniform_system));
    }
}
pub fn update_uniform_system(
    mut query: Query<(&mut TerrainConfig)>,
    mut globe_camera_query: Query<&mut GlobeCamera>,
) {
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
    let mvp = globe_camera.get_mvp();
    for (mut config) in query.iter_mut(){
        config.mvp = mvp.as_mat4();
    }
}
/// 增加PbrBundle
pub fn render_system(
    mut commands: Commands,
    mut quadtree_tile_query: Query<
        (
            Entity,
            &QuadtreeTile,
            &TileKey,
            &HeightmapTerrainDataCom,
            &mut TileImageryVec,
            &Rectangle,
        ),
        With<Renderable>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut globe_camera_query: Query<&mut GlobeCamera>,
    layer_query: Query<(&ImageryLayer, &ImageryProviderCom)>,
) {
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
    let mvp = globe_camera.get_mvp();
    for (entity, quadtree_tile, tile_key, terrain_data_com, mut tile_iamgery_list, rectangle) in
        &mut quadtree_tile_query
    {
        commands.entity(entity).remove::<Renderable>();
        let terrain_mesh = terrain_data_com.0._mesh.as_ref().unwrap();
        let wrap_terrain_mesh = WrapTerrainMesh(terrain_mesh);
        let mesh: Mesh = wrap_terrain_mesh.into();
        let mut attachments = vec![];
        for tile_imagery in tile_iamgery_list.0.iter_mut() {
            let ready_imagery = tile_imagery.ready_imagery.as_ref().unwrap().clone();
            let imagery = ready_imagery.0.read().unwrap();
            let (imagery_layer, imagery_provider) = layer_query.get(imagery.key.layer_id).unwrap();
            if imagery_layer.alpha == 0.0 {
                continue;
            }
            let texture = match imagery.texture.as_ref() {
                Some(v) => v.clone(),
                None => panic!("readyImagery is not actually ready!"),
            };
            // let image = images
            //     .get_mut(&texture)
            //     .expect("expect gpu image of imagery");
            // image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
            if tile_imagery.texture_translation_and_scale.is_none() {
                tile_imagery.texture_translation_and_scale =
                    Some(ImageryLayer::calculate_texture_translation_and_scale(
                        imagery_provider,
                        rectangle.clone(),
                        tile_imagery,
                    ))
            }
            let attachment = TerrainAttachment {
                handle: texture,
                translation_and_scale: tile_imagery
                    .texture_translation_and_scale
                    .unwrap()
                    .as_vec4(),
                coordinate_rectangle: tile_imagery.texture_coordinate_rectangle.unwrap().as_vec4(),
                web_mercator_t: if tile_imagery.use_web_mercator_t {
                    1.0
                } else {
                    0.0
                },
                alpha: imagery_layer.alpha as f32,
                night_alpha: imagery_layer.night_alpha as f32,
                day_alpha: imagery_layer.day_alpha as f32,
                brightness: imagery_layer.brightness as f32,
                contrast: imagery_layer.contrast as f32,
                hue: imagery_layer.hue as f32,
                saturation: imagery_layer.saturation as f32,
                one_over_gamma: imagery_layer.gamma as f32,
                width: imagery_provider.0.get_tile_width(),
                height: imagery_provider.0.get_tile_height(),
            };
            attachments.push(attachment);
        }
        let terrain_config = TerrainConfig {
            scale_and_bias: terrain_mesh.encoding.matrix.as_mat4(),
            center_3d: terrain_mesh.center.as_vec3(),
            mvp: mvp.as_mat4(),
            minimum_height: terrain_mesh.encoding.minimum_height as f32,
            maximum_height: terrain_mesh.encoding.maximum_height as f32,
            quantization_bits12: terrain_mesh.encoding.quantization == TerrainQuantization::BITS12,
            has_web_mercator_t: terrain_mesh.encoding.has_web_mercator_t,
            attachments: attachments,
            tile_key: tile_key.clone(),
        };
        let bundle = TerrainBundle {
            config: terrain_config,
            mesh: meshes.add(mesh),
            tile_rendered: TileRendered(tile_key.clone()),
            spatial_bundle: SpatialBundle::INHERITED_IDENTITY,
            no_frustum_culling: NoFrustumCulling,
        };
        commands.entity(entity).insert(bundle);
    }
}
