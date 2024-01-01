use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

use bevy::{prelude::*, utils::HashMap};

use super::{
    imagery_layer::{ImageryLayer, ImageryLayerId},
    tile_key::TileKey,
};
use houtu_scene::{Rectangle, TilingScheme};
#[derive(Resource)]
pub struct ImageryStorage {
    map: HashMap<ImageryKey, Imagery>,
}
impl ImageryStorage {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
    #[inline]
    pub fn get_cloned(&self, key: &ImageryKey) -> Option<Imagery> {
        return self.map.get(key).and_then(|x| Some(x.clone()));
    }
    #[inline]
    pub fn get(&self, key: &ImageryKey) -> Option<&Imagery> {
        return self.map.get(key);
    }
    #[inline]
    pub fn get_mut(&mut self, key: &ImageryKey) -> Option<&mut Imagery> {
        return self.map.get_mut(key);
    }
    pub fn add(
        &mut self,
        tile_key: &TileKey,
        imagery_layer_id: &ImageryLayerId,
        tiling_scheme: &Box<dyn TilingScheme>,
    ) -> Imagery {
        let imagery_key = ImageryKey::new(tile_key.clone(), imagery_layer_id.clone());
        if let Some(v) = self.get(&imagery_key) {
            return v.clone();
        } else {
            let rectangle =
                tiling_scheme.tile_x_y_to_rectange(tile_key.x, tile_key.y, tile_key.level);
            let mut new_imagery = Imagery::new(
                imagery_key,
                tile_key.parent().and_then(|x| {
                    let parent_key = ImageryKey::new(x, imagery_layer_id.clone());
                    self.get_cloned(&parent_key)
                }),
                rectangle,
            );
            self.map.insert(imagery_key, new_imagery.clone());
            // bevy::log::info!("add new imagery {:?}", imagery_key);
            return new_imagery;
        }
    }
    #[inline]
    pub fn remove(&mut self, key: &ImageryKey) -> Option<Imagery> {
        self.map.remove(key)
    }
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum ImageryState {
    UNLOADED = 0,
    TRANSITIONING = 1,
    RECEIVED = 2,
    TEXTURE_LOADED = 3,
    READY = 4,
    FAILED = 5,
    INVALID = 6,
    PLACEHOLDER = 7,
    REQUESTING = 8,
}
impl PartialEq for Imagery {
    fn eq(&self, other: &Self) -> bool {
        self.read().key == other.read().key
    }
}
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct ImageryKey {
    pub key: TileKey,
    pub layer_id: ImageryLayerId,
}

impl ImageryKey {
    pub fn new(key: TileKey, layer_id: ImageryLayerId) -> Self {
        return Self { key, layer_id };
    }
}
#[derive(Clone)]
pub struct Imagery(pub Arc<RwLock<ImageryInternal>>);
impl Imagery {
    pub fn new(imagery_key: ImageryKey, parent: Option<Imagery>, rectangle: Rectangle) -> Self {
        Self(Arc::new(RwLock::new(ImageryInternal::new(
            imagery_key,
            parent,
            rectangle,
        ))))
    }
    pub fn read(&self) -> RwLockReadGuard<'_, ImageryInternal> {
        self.0.read().unwrap()
    }
    pub fn write(&self) -> RwLockWriteGuard<'_, ImageryInternal> {
        self.0.write().unwrap()
    }
    pub fn get_state(&self) -> ImageryState {
        return self.read().state;
    }
    pub fn get_layer_id(&self) -> ImageryLayerId {
        return self.read().key.layer_id.clone();
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
    pub fn get_layer_id(&self) -> &ImageryLayerId {
        return &self.key.layer_id;
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
