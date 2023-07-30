use super::{imagery_layer_storage::ImageryLayerStorage, quadtree_primitive::QuadtreePrimitive};
use bevy::prelude::*;
#[derive(Resource)]
pub struct RenderContext {
    pub quadtree_primitive: QuadtreePrimitive,
    pub imagery_layer_storage: ImageryLayerStorage,
}
impl RenderContext {
    pub fn new() -> Self {
        Self {
            quadtree_primitive: QuadtreePrimitive::new(),
            imagery_layer_storage: ImageryLayerStorage::new(),
        }
    }
}
