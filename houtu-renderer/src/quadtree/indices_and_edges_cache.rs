use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use houtu_scene::IndicesAndEdgesCache;
#[derive(Resource)]
pub struct IndicesAndEdgesCacheArc(pub Arc<Mutex<IndicesAndEdgesCache>>);
impl IndicesAndEdgesCacheArc {
    pub fn new() -> Self {
        IndicesAndEdgesCacheArc(Arc::new(Mutex::new(IndicesAndEdgesCache::new())))
    }
    pub fn get_cloned_cache(&self) -> Arc<Mutex<IndicesAndEdgesCache>> {
        return self.0.clone();
    }
}
