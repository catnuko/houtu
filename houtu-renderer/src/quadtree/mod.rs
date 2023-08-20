use bevy::{core::FrameCount, prelude::*, render::renderer::RenderDevice, window::PrimaryWindow};
use houtu_jobs::JobSpawner;
use houtu_scene::GeographicTilingScheme;
use rand::Rng;

use crate::xyz_imagery_provider::XYZImageryProvider;

use self::{
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

use super::{
    camera::GlobeCamera,
    wmts_imagery_provider::{WMTSImageryProvider, WMTSImageryProviderOptions},
};

pub mod create_terrain_mesh_job;
pub mod credit;
pub mod ellipsoid_terrain_provider;
pub mod globe_surface_tile;
pub mod globe_surface_tile_provider;
// pub mod imagery;
pub mod imagery_layer;
pub mod imagery_layer_storage;
pub mod imagery_provider;
pub mod imagery_storage;
pub mod indices_and_edges_cache;
pub mod quadtree_primitive;
pub mod quadtree_primitive_debug;
pub mod quadtree_tile;
pub mod quadtree_tile_storage;
pub mod reproject_texture;
// pub mod terrain_datasource;
pub mod terrain_provider;
pub mod texture_minification_filter;
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
        app.insert_resource(QuadtreePrimitive::new());
        app.insert_resource(ImageryLayerStorage::new());
        app.insert_resource(RootTraversalDetails::new());
        app.insert_resource(AllTraversalQuadDetails::new());
        app.insert_resource(IndicesAndEdgesCacheArc::new());
        app.insert_resource(ImageryStorage::new());
        app.add_system(render_system);
        app.add_system(process_terrain_state_machine_system.after(render_system));
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
    mut imagery_storage: ResMut<ImageryStorage>,
) {
    let Ok(window) = primary_query.get_single() else {
        return;
    };
    let mut globe_camera = globe_camera_query
        .get_single_mut()
        .expect("GlobeCamera不存在");
    primitive.begin_frame();
    primitive.render(
        &mut globe_camera,
        &frame_count,
        window,
        &mut all_traversal_quad_details,
        &mut root_traversal_details,
        &mut imagery_layer_storage,
        &mut imagery_storage,
    );

    primitive.end_frame(
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
        &mut imagery_storage,
    );
}
