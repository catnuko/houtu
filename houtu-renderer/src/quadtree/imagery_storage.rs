use bevy::{prelude::*, utils::HashMap};

use super::{
    imagery_layer::{ImageryLayer, ImageryLayerId},
    imagery_provider::ImageryProvider,
    indices_and_edges_cache::IndicesAndEdgesCacheArc,
    reproject_texture::ReprojectTextureTaskQueue,
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
    ) -> ImageryKey {
        let imagery_key = ImageryKey::new(tile_key.clone(), imagery_layer_id.clone());
        if self.map.contains_key(&imagery_key) {
            return imagery_key;
        } else {
            let cloned = imagery_key.clone();
            let rectangle =
                tiling_scheme.tile_x_y_to_rectange(tile_key.x, tile_key.y, tile_key.level);
            let mut new_imagery = Imagery::new(
                imagery_key,
                tile_key.parent().and_then(|x| {
                    let parent_key = ImageryKey::new(x, imagery_layer_id.clone());
                    self.add_reference(&parent_key);
                    Some(parent_key)
                }),
                rectangle,
            );
            new_imagery.add_reference();
            self.map.insert(new_imagery.key, new_imagery);
            // bevy::log::info!("add new imagery {:?}", imagery_key);
            return cloned;
        }
    }
    #[inline]
    pub fn remove(&mut self, key: &ImageryKey) -> Option<Imagery> {
        self.map.remove(key)
    }
    pub fn add_reference(&mut self, key: &ImageryKey) {
        if let Some(v) = self.get_mut(key) {
            v.add_reference();
        };
    }
    pub fn release_reference(&mut self, key: &ImageryKey) -> u32 {
        if let Some(v) = self.get_mut(key) {
            v.release_reference();
            if v.reference_count == 0 {
                if v.parent.is_some() {
                    let parent_key = v.parent.as_ref().unwrap().clone();
                    self.release_reference(&parent_key);
                }
                let v = self.get_mut(key).unwrap();
                if v.texture.is_some() {
                    //销毁v.texture
                }
                // TODO 还有很多没做，不确定会不会有问题
                self.remove(key);
                // bevy::log::info!("imagery is removed {:?}", key);
                return 0;
            }
            let v = self.get_mut(key).unwrap();
            return v.reference_count;
        } else {
            panic!("imagery is not existed,{:?}", key);
        };
    }
    pub fn create_placeholder(
        &mut self,
        imagery_layer_id: &ImageryLayerId,
        tiling_scheme: &Box<dyn TilingScheme>,
    ) -> ImageryKey {
        let key = self.add(
            &TileKey {
                x: 0,
                y: 0,
                level: 0,
            },
            imagery_layer_id,
            tiling_scheme,
        );
        let imagery = self.get_mut(&key).unwrap();
        imagery.state = ImageryState::PLACEHOLDER;
        return key.clone();
    }
}
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
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
        self.key == other.key
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
pub struct Imagery {
    pub state: ImageryState,
    pub image_url: Option<String>,
    pub texture: Option<Handle<Image>>,
    pub rectangle: Rectangle,
    pub reference_count: u32,
    pub parent: Option<ImageryKey>,
    pub key: ImageryKey,
}
impl Imagery {
    pub fn new(imagery_key: ImageryKey, parent: Option<ImageryKey>, rectangle: Rectangle) -> Self {
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
