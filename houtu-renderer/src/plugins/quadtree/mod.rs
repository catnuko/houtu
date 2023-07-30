use bevy::{core::FrameCount, prelude::*, window::PrimaryWindow};

use self::{
    imagery_layer_storage::ImageryLayerStorage,
    quadtree_primitive::QuadtreePrimitive,
    render_context::RenderContext,
    traversal_details::{AllTraversalQuadDetails, RootTraversalDetails},
};

use super::camera::GlobeCamera;

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
mod terrain_provider;
mod tile_availability;
mod tile_imagery;
mod tile_key;
mod tile_replacement_queue;
mod tile_selection_result;
mod traversal_details;
pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(QuadtreePrimitive::new());
        app.insert_resource(ImageryLayerStorage::new());
        app.insert_resource(RootTraversalDetails::new());
        app.insert_resource(AllTraversalQuadDetails::new());
        app.add_system(render_system);
    }
}
fn render_system(
    mut primitive: ResMut<QuadtreePrimitive>,
    mut imagery_layer_storage: ResMut<ImageryLayerStorage>,
    mut globe_camera_query: Query<(&mut GlobeCamera)>,
    frame_count: Res<FrameCount>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    mut all_traversal_quad_details: ResMut<AllTraversalQuadDetails>,
    mut root_traversal_details: ResMut<RootTraversalDetails>,
    time: Res<Time>,
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
    );
}
