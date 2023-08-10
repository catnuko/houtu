use bevy::{
    prelude::Resource,
    utils::{HashMap, Uuid},
};

use super::{
    imagery::{Imagery, ImageryKey},
    imagery_layer::ImageryLayer,
};
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
    pub fn get_imagery(&self, imagery_key: &ImageryKey) -> Option<&Imagery> {
        self.get(&imagery_key.layer_id)
            .and_then(|x| x.get_imagery(imagery_key))
    }
    pub fn get_imagery_mut(&mut self, imagery_key: &ImageryKey) -> Option<&mut Imagery> {
        self.get_mut(&imagery_key.layer_id)
            .and_then(|x| x.get_imagery_mut(imagery_key))
    }
}
