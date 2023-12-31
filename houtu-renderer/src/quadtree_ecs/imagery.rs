use std::{
    clone,
    sync::{Arc, RwLock},
};

use bevy::{
    prelude::{Component, Entity, Handle, Image},
    utils::HashMap,
};
use houtu_scene::Rectangle;

use crate::{
    image,
    quadtree::{imagery_storage::ImageryState, tile_key::TileKey},
};

use super::imagery_layer::{ImageryLayer, ImageryLayerId, ImageryProviderCom};

#[derive(Component, Default)]
pub struct ImageryCache {
    map: HashMap<ImageryKey, Imagery>,
}
impl ImageryCache {
    pub fn remove(&mut self, key: &ImageryKey) -> Option<Imagery> {
        self.map.remove(key)
    }
    pub fn add(
        &mut self,
        layer_entity: Entity,
        provider: &ImageryProviderCom,
        tile_key: &TileKey,
    ) -> Imagery {
        let mut parent = None;
        if tile_key.level != 0 {
            parent = tile_key.parent().and_then(|parent_key| {
                Some(
                    self.get_cloned(&ImageryKey::new(parent_key.clone(), layer_entity))
                        .unwrap(),
                )
            })
        }
        //TODO imagery ready
        let rectangle = provider.0.get_tiling_scheme().tile_x_y_to_rectange(
            tile_key.x,
            tile_key.y,
            tile_key.level,
        );
        let key = ImageryKey {
            key: tile_key.clone(),
            layer_id: layer_entity,
        };
        let imagery = Imagery(Arc::new(RwLock::new(ImageryInternal::new(
            key.clone(),
            parent,
            rectangle,
        ))));
        let cloned = imagery.clone();
        self.map.insert(key, imagery);
        return cloned;
    }
    pub fn get_cloned(&self, key: &ImageryKey) -> Option<Imagery> {
        self.map.get(key).and_then(|x| Some(x.clone()))
    }
}
#[derive(Clone)]
pub struct Imagery(pub Arc<RwLock<ImageryInternal>>);
impl Imagery {
    pub fn get_layer_id(&self) -> Entity {
        return self.0.read().unwrap().get_layer_id();
    }
    pub fn get_state(&self) -> ImageryState {
        return self.0.read().unwrap().state;
    }
}
impl PartialEq for Imagery {
    fn eq(&self, other: &Self) -> bool {
        self.0.read().unwrap().key == other.0.read().unwrap().key
    }
}
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct ImageryKey {
    pub key: TileKey,
    pub layer_id: Entity,
}

impl ImageryKey {
    pub fn new(key: TileKey, layer_id: Entity) -> Self {
        return Self { key, layer_id };
    }
}
pub struct ImageryInternal {
    pub state: ImageryState,
    pub image_url: Option<String>,
    pub texture: Option<Handle<Image>>,
    pub rectangle: Rectangle,
    pub reference_count: u32,
    pub parent: Option<Imagery>,
    pub key: ImageryKey,
}
impl ImageryInternal {
    pub fn new(imagery_key: ImageryKey, parent: Option<Imagery>, rectangle: Rectangle) -> Self {
        Self {
            key: imagery_key,
            state: ImageryState::UNLOADED,
            texture: None,
            image_url: None,
            rectangle: rectangle,
            reference_count: 0,
            parent,
        }
    }
    pub fn get_tile_key(&self) -> &TileKey {
        return &self.key.key;
    }
    pub fn get_layer_id(&self) -> Entity {
        return self.key.layer_id;
    }
    #[inline]
    pub fn set_texture(&mut self, new_texture: Handle<Image>) {
        self.texture = Some(new_texture);
    }
    #[inline]
    fn add_reference(&mut self) {
        self.reference_count += 1;
    }
    #[inline]
    fn release_reference(&mut self) {
        self.reference_count -= 1;
    }
}
