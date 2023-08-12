use bevy::{prelude::Resource, utils::HashMap};

use super::{
    imagery_layer::{ImageryLayer, ImageryLayerId},
    imagery_storage::{Imagery, ImageryKey},
};
#[derive(Resource)]
pub struct ImageryLayerStorage {
    pub map: HashMap<ImageryLayerId, ImageryLayer>,
}
impl ImageryLayerStorage {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn add(&mut self, imagery_layer: ImageryLayer) {
        self.map.insert(imagery_layer.id.clone(), imagery_layer);
    }
    pub fn remove(&mut self, imagery_layer_id: &ImageryLayerId) {
        self.map
            .remove(imagery_layer_id)
            .and_then(|mut imagery_layer| {
                imagery_layer.destroy();
                Some(())
            });
    }
    pub fn get(&self, id: &ImageryLayerId) -> Option<&ImageryLayer> {
        return self.map.get(id);
    }
    pub fn get_mut(&mut self, id: &ImageryLayerId) -> Option<&mut ImageryLayer> {
        return self.map.get_mut(id);
    }
}
