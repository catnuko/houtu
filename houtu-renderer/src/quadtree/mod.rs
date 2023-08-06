use bevy::{core::FrameCount, prelude::*, render::renderer::RenderDevice, window::PrimaryWindow};
use houtu_jobs::JobSpawner;
use houtu_scene::GeographicTilingScheme;
use rand::Rng;

use crate::xyz_imagery_provider::XYZImageryProvider;

use self::{
    globe_surface_tile::process_terrain_state_machine_system,
    imagery_layer::ImageryLayer,
    imagery_layer_storage::ImageryLayerStorage,
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

pub mod create_terrain_mesh_job;
pub mod credit;
pub mod ellipsoid_terrain_provider;
pub mod globe_surface_tile;
pub mod globe_surface_tile_provider;
pub mod imagery;
pub mod imagery_layer;
pub mod imagery_layer_storage;
pub mod imagery_provider;
pub mod indices_and_edges_cache;
pub mod quadtree_primitive;
pub mod quadtree_primitive_debug;
pub mod quadtree_tile;
pub mod quadtree_tile_storage;
pub mod render_context;
pub mod reproject_texture;
// pub mod terrain_datasource;
pub mod terrain_provider;
pub mod terrian_material;
pub mod tile_availability;
pub mod tile_imagery;
pub mod tile_key;
pub mod tile_replacement_queue;
pub mod tile_selection_result;
pub mod traversal_details;
pub mod upsample_job;
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(reproject_texture::Plugin);
        app.add_plugin(MaterialPlugin::<terrian_material::TerrainMeshMaterial>::default());
        app.insert_resource(QuadtreePrimitive::new());
        app.insert_resource(ImageryLayerStorage::new());
        app.insert_resource(RootTraversalDetails::new());
        app.insert_resource(AllTraversalQuadDetails::new());
        app.insert_resource(IndicesAndEdgesCacheArc::new());
        app.add_startup_system(setup);
        app.add_system(render_system);
        app.add_system(process_terrain_state_machine_system.after(render_system));
        app.add_system(real_render_system.after(process_terrain_state_machine_system));
        app.add_system(imagery_layer::finish_reproject_texture_system);
    }
}
fn setup(mut imagery_layer_storage: ResMut<ImageryLayerStorage>) {
    let xyz = XYZImageryProvider::new(Box::new(GeographicTilingScheme::default()));
    imagery_layer_storage.add(ImageryLayer::new(Box::new(xyz)))
}

fn render_system(
    mut primitive: ResMut<QuadtreePrimitive>,
    mut imagery_layer_storage: ResMut<ImageryLayerStorage>,
    mut globe_camera_query: Query<&mut GlobeCamera>,
    frame_count: Res<FrameCount>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    mut all_traversal_quad_details: ResMut<AllTraversalQuadDetails>,
    mut root_traversal_details: ResMut<RootTraversalDetails>,
    time: Res<Time>,
    mut job_spawner: JobSpawner,
    indices_and_edges_cache: Res<IndicesAndEdgesCacheArc>,
    mut render_world_queue: ResMut<ReprojectTextureTaskQueue>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    render_device: Res<RenderDevice>,
) {
    let Ok(window) = primary_query.get_single() else {
        return;
    };
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
    primitive.beginFrame();
    primitive.render(
        &mut globe_camera,
        &frame_count,
        window,
        &mut all_traversal_quad_details,
        &mut root_traversal_details,
        &mut imagery_layer_storage,
    );

    primitive.endFrame(
        &frame_count,
        &time,
        &mut globe_camera,
        &mut imagery_layer_storage,
        &mut job_spawner,
        &indices_and_edges_cache,
        &asset_server,
        &mut images,
        &mut render_world_queue,
        &render_device,
    );
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
                        image: Some(asset_server.load("icon.png")),
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
