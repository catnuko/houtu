use bevy::{core::FrameCount, prelude::*, render::renderer::RenderDevice, window::PrimaryWindow};
use houtu_jobs::JobSpawner;
use rand::Rng;

use self::{
    globe_surface_tile::process_terrain_state_machine_system,
    imagery_layer_storage::ImageryLayerStorage,
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    quadtree_primitive::QuadtreePrimitive,
    quadtree_tile::QuadtreeTileLoadState,
    reproject_texture::ReprojectTextureTaskQueue,
    terrian_material::TerrainMeshMaterial,
    tile_key::TileKey,
    traversal_details::{AllTraversalQuadDetails, RootTraversalDetails},
};

use super::camera::GlobeCamera;

mod create_terrain_mesh_job;
mod credit;
mod ellipsoid_terrain_provider;
mod globe_surface_tile;
mod globe_surface_tile_provider;
mod imagery;
mod imagery_layer;
mod imagery_layer_storage;
mod imagery_provider;
mod indices_and_edges_cache;
mod quadtree_primitive;
mod quadtree_primitive_debug;
mod quadtree_tile;
mod quadtree_tile_storage;
mod render_context;
mod reproject_texture;
// mod terrain_datasource;
mod terrain_provider;
mod terrian_material;
mod tile_availability;
mod tile_imagery;
mod tile_key;
mod tile_replacement_queue;
mod tile_selection_result;
mod traversal_details;
mod upsample_job;
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
        app.add_system(render_system);
        app.add_system(process_terrain_state_machine_system.after(render_system));
        app.add_system(real_render_system.after(process_terrain_state_machine_system));
        app.add_system(imagery_layer::finish_reproject_texture_system);
    }
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
                        // image: Some(asset_server.load("icon.png")),
                        image:Some( asset_server.load(format!(
                                "https://maps.omniscale.net/v2/houtu-earth-f1ad0341/style.default/{}/{}/{}.png",
                                key.level, key.x, key.y,
                            ))),
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
//                 commands.spawn((
//                     MaterialMeshBundle {
//                         mesh: mesh,
//                         material: terrain_materials.add(TerrainMeshMaterial {
//                             color: Color::rgba(r, g, b, 1.0),
//                             image: Some(asset_server.load("icon.png")),
//                             // image: asset_server.load(format!("https://t5.tianditu.gov.cn/img_c/wmts?SERVICE=WMTS&REQUEST=GetTile&VERSION=1.0.0&LAYER=vec&STYLE=default&TILEMATRIXSET=w&FORMAT=tiles&TILECOL={}&TILEROW={}&TILEMATRIX={}&tk=b931d6faa76fc3fbe622bddd6522e57b",x,y,level)),
//                             // image: asset_server.load(format!("tile/{}/{}/{}.png", level, y, x,)),
//                             // image:Some( asset_server.load(format!(
//                             //     "https://maps.omniscale.net/v2/houtu-b8084b0b/style.default/{}/{}/{}.png",
//                             //     tile.level, tile.x, tile.y,
//                             // ))),
//                             // image: None,
//                         }),
//                         // material: standard_materials.add(Color::rgba(r, g, b, 1.0).into()),
//                         ..Default::default()
//                     },
//                     TileKey::new(tile.y, tile.x, tile.level),
//                     // TileState::START,
//                 ));
