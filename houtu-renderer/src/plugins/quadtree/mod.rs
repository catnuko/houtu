use bevy::prelude::*;

use self::{quadtree_primitive::QuadtreePrimitive, render_context::RenderContext};

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
        app.insert_resource(RenderContext::new());
    }
}
