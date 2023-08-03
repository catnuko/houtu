use std::array::IntoIter;

use bevy::{
    prelude::Resource,
    utils::{HashMap, Uuid},
};

use super::{imagery_layer::ImageryLayer, imagery_provider::ImageryProvider};
#[derive(Resource)]
pub struct ImageryLayerStorage {
    pub map: HashMap<Uuid, ImageryLayer>,
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
    pub fn remove(&mut self, imagery_layer_id: &Uuid) {
        self.map
            .remove(imagery_layer_id)
            .and_then(|mut imagery_layer| {
                imagery_layer.destroy();
                Some(())
            });
    }
    pub fn get(&self, id: &Uuid) -> Option<&ImageryLayer> {
        return self.map.get(id);
    }
    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut ImageryLayer> {
        return self.map.get_mut(id);
    }
}
