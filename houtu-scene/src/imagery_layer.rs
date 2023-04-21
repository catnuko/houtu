use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    math::prelude::*,
    prelude::Visibility,
    reflect::DynamicMap,
    transform::components::Transform,
};

use crate::layer_id::LayerId;
pub enum ImageLayerState {
    UNLOADED = 0,
    READY = 1,
}
#[derive(Component, Clone)]
pub struct ImageLayer {
    pub show: bool,
    pub index: i32,
    pub alpha: f32,
    pub night_alpha: f32,
    pub day_alpha: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub saturation: f32,
    pub gamma: f32,
    pub ready: bool,
    pub hue: f32,
    pub id: LayerId,
    pub name: String,
    pub ogc_type: String,
    pub url: String,
    pub state: ImageLayerState,
}
impl Default for ImageLayer {
    fn default() -> Self {
        Self {
            show: true,
            index: -1,
            alpha: 1.0,
            night_alpha: 1.0,
            day_alpha: 1.0,
            brightness: 1.0,
            contrast: 1.0,
            saturation: 1.0,
            gamma: 1.0,
            ready: false,
            hue: 0.0,
            id: LayerId::default(),
            name: "".to_string(),
            ogc_type: "".to_string(),
            url: "".to_string(),
            state: ImageLayerState::UNLOADED,
        }
    }
}
