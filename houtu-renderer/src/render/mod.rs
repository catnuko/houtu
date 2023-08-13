use bevy::{core::FrameCount, prelude::*, render::renderer::RenderDevice, window::PrimaryWindow};
use houtu_jobs::JobSpawner;
use houtu_scene::{GeographicTilingScheme, WebMercatorTilingScheme};
use rand::Rng;

use crate::xyz_imagery_provider::XYZImageryProvider;

use super::quadtree::{
    globe_surface_tile::process_terrain_state_machine_system,
    imagery_layer::ImageryLayer,
    imagery_layer_storage::ImageryLayerStorage,
    imagery_storage::ImageryStorage,
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    quadtree_primitive::QuadtreePrimitive,
    quadtree_tile::QuadtreeTileLoadState,
    reproject_texture::ReprojectTextureTaskQueue,
    terrian_material::TerrainMeshMaterial,
    tile_key::TileKey,
    traversal_details::{AllTraversalQuadDetails, RootTraversalDetails},
};

use super::{
    camera::GlobeCamera,
    wmts_imagery_provider::{WMTSImageryProvider, WMTSImageryProviderOptions},
};
/// 负责渲染quadtree调度后生成的瓦片
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup);
        app.add_system(real_render_system.after(process_terrain_state_machine_system));
    }
}
fn setup(mut imagery_layer_storage: ResMut<ImageryLayerStorage>) {
    let xyz = XYZImageryProvider::new(Box::new(WebMercatorTilingScheme::default()));
    imagery_layer_storage.add(ImageryLayer::new(Box::new(xyz)))
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
        if tile.state == QuadtreeTileLoadState::DONE && tile.entity.is_none() {
            // info!("render tile key={:?}", key);
            info!("{:?} length is {}", tile.key, tile.data.imagery.len());
            if tile.data.imagery.len() != 0 {
                let imagery = tile.data.imagery.get(0).unwrap();
                let ready_imagery = imagery_storage
                    .get(imagery.ready_imagery.as_ref().unwrap())
                    .unwrap();
                info!("ready imagery {:?}", ready_imagery.key);
                let mut rng = rand::thread_rng();
                let r: f32 = rng.gen();
                let g: f32 = rng.gen();
                let b: f32 = rng.gen();
                let rendered_entity = commands.spawn((
                    MaterialMeshBundle {
                        mesh: meshes.add(
                            tile.data
                                .get_cloned_terrain_data()
                                .lock()
                                .unwrap()
                                .get_mesh()
                                .unwrap()
                                .into(),
                        ),
                        material: terrain_materials.add(TerrainMeshMaterial {
                            color: Color::rgba(r, g, b, 1.0),
                            image: ready_imagery.texture.clone(),
                            // image: Some(asset_server.load("icon.png")),
                            // image:Some( asset_server.load(format!(
                            //         "https://maps.omniscale.net/v2/houtu-earth-f1ad0341/style.default/{}/{}/{}.png",
                            //         key.level, key.x, key.y,
                            //     ))),
                        }),
                        // material: standard_materials.add(Color::rgba(r, g, b, 1.0).into()),
                        ..Default::default()
                    },
                    TileRendered(tile.key),
                ));
                let entity = rendered_entity.id();
                tile.entity = Some(entity);
            }
        }
    }
}
