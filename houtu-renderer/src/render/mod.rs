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
    terrain_data::TerrainBundle, terrian_material::TerrainMeshMaterial,
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
mod node_atlas;
mod node_atlas_render;
mod terrain_data;
mod terrain_data_render;
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
        app.add_plugins(terrain_render_pipeline::TerrainRenderPlugin);
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
        url: "https://maps.omniscale.net/v2/houtuearth-4781e785/style.default/{z}/{x}/{y}.png",
        // url: "icon.png",
        // url: "https://api.maptiler.com/maps/basic-v2/256/{z}/{x}/{y}.png?key=Modv7lN1eXX1gmlqW0wY",
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
