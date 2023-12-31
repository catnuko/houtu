use std::sync::{Arc, RwLock};

use bevy::{
    math::DVec4,
    prelude::{Component, Handle, Image},
    utils::HashMap,
};
use houtu_scene::Rectangle;

use super::imagery::Imagery;

use super::imagery_layer::{ImageryLayer, ImageryLayerId, ImageryProviderCom};

// pub struct TileImagery(Arc<RwLock<TileImageryInternal>>);\
#[derive(Component, Default)]
pub struct TileImageryVec(pub Vec<TileImagery>);
impl TileImageryVec {
    pub fn add(
        &mut self,
        loading_imagery: Imagery,
        texture_coordinate_rectangle: Option<DVec4>,
        use_web_mercator_t: bool,
        insertion_point: Option<usize>,
    ) {
        let tile_imagery = TileImagery::new(
            loading_imagery,
            texture_coordinate_rectangle,
            use_web_mercator_t,
        );
        if insertion_point.is_none() {
            self.0.push(tile_imagery);
        } else {
            let insertion_point = insertion_point.unwrap();
            self.0
                .splice(insertion_point..insertion_point, [tile_imagery]);
        }
    }
    pub fn remove(&mut self, index: usize) -> TileImagery {
        self.0.remove(index)
    }
    pub fn len(&self)->usize{
        self.0.len()
    }
}
pub struct TileImagery {
    pub texture_coordinate_rectangle: Option<DVec4>,
    pub texture_translation_and_scale: Option<DVec4>,
    pub use_web_mercator_t: bool,
    pub loading_imagery: Option<Imagery>,
    pub ready_imagery: Option<Imagery>,
}

impl TileImagery {
    pub fn new(
        loading_imagery: Imagery,
        texture_coordinate_rectangle: Option<DVec4>,
        use_web_mercator_t: bool,
    ) -> Self {
        Self {
            texture_coordinate_rectangle,
            use_web_mercator_t,
            texture_translation_and_scale: None,
            ready_imagery: None,
            loading_imagery: Some(loading_imagery),
        }
    }
}
