use bevy::prelude::*;
use std::sync;

use crate::{
    geographic_projection::GeographicProjection,
    geographic_tiling_scheme::GeographicTilingScheme,
    imagery_layer::ImageLayer,
    layer_id::{self, LayerId},
    wmts_imagery_layer::WMTSImageryLayer,
    wmts_imagery_provider,
};
#[derive(Clone, Debug, Resource)]
pub struct ImageryLayerCollection {
    pub layers: Vec<WMTSImageryLayer>,
    pub selected_layer_id: Option<layer_id::LayerId>,
}
impl Default for ImageryLayerCollection {
    fn default() -> Self {
        Self::new()
    }
}
impl ImageryLayerCollection {
    pub fn new() -> ImageryLayerCollection {
        return Self {
            layers: vec![],
            selected_layer_id: None,
        };
    }
    pub fn add_layer(&mut self, layer: WMTSImageryLayer) {
        self.layers.push(layer);
    }
    pub fn get_layer(
        &self,
        layer_id: LayerId,
    ) -> Option<
        &WMTSImageryLayer<wmts_imagery_provider::WMTSImageryLayerProvider<GeographicTilingScheme>>,
    > {
        self.layers.iter().find(|layer| layer.id == layer_id)
    }
    pub fn iter_layer(&self) -> impl Iterator<Item = &WMTSImageryLayer> {
        self.layers.iter()
    }
}
