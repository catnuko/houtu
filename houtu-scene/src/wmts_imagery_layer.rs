use bevy::{
    app::prelude::*,
    ecs::{bundle::Bundle, prelude::*},
    math::prelude::*,
    prelude::Visibility,
    reflect::DynamicMap,
    transform::components::Transform,
};
use geodesy::Ellipsoid;

use crate::{
    geographic_tiling_scheme::GeographicTilingScheme,
    imagery::Imagery,
    imagery_layer::ImageLayer,
    imagery_provider::{self, ImageryProvider},
    wmts_imagery_provider::WMTSImageryLayerProvider,
};
use crate::{layer_id::LayerId, tiling_scheme::TilingScheme};
pub enum ImageLayerState {
    UNLOADED = 0,
    TRANSITIONING = 1,
    RECEIVED = 2,
    TEXTURE_LOADED = 3,
    READY = 4,
    FAILED = 5,
    INVALID = 6,
    PLACEHOLDER = 7,
}
#[derive(Bundle)]
pub struct WMTSImageryLayer<T = WMTSImageryLayerProvider>
where
    T: ImageryProvider + std::marker::Send + std::marker::Sync + 'static,
{
    pub image_layer: ImageLayer,
    pub transform: Transform,
    pub visibility: Visibility,
    pub imagery_provider: T,
    pub level_zero_tiles: Option<Vec<Imagery>>,
}
impl Default for WMTSImageryLayer<T>
where
    T: ImageryProvider,
{
    fn default() -> Self {
        Self {
            image_layer: ImageLayer::default(),
            transform: Transform::default(),
            visibility: true,
            imagery_provider: WMTSImageryLayerProvider::default(),
            level_zero_tiles: None,
        }
    }
}

impl WMTSImageryLayer<T>
where
    T: ImageryProvider,
{
    pub fn new(options: WMTSImageryLayerOptions) -> Self {
        Self {
            image_layer: ImageLayer {
                name: options.name,
                url: options.url,
                ..Default::default()
            },
            visibility: true,
            transform: Transform::default(),
            imagery_provider: WMTSImageryLayerProvider::from_wmts_imagery_layer_option(options),
            level_zero_tiles: None,
        }
    }
    pub fn from_url(url: &str) -> Self {
        let mut image_layer = ImageLayer::default();
        image_layer.url = url.to_string();
        Self {
            image_layer,
            ..Default::default()
        }
    }
}

pub struct WMTSImageryLayerOptions<T = GeographicTilingScheme>
where
    T: TilingScheme,
{
    pub name: String,
    pub url: String,
    pub layer: String,
    pub style: String,
    pub format: String,
    pub matrix_set: String,
    pub crs: String,
    pub min_zoom: u32,
    pub max_zoom: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_matrix_labels: Vec<String>,
    pub tile_matrix_set: Vec<TileMatrix>,
    pub tiling_scheme: T,
}
pub struct TileMatrix {
    pub identifier: String,
    pub scale_denominator: f64,
    pub top_left_corner: Vec<f64>,
    pub tile_width: u32,
    pub tile_height: u32,
    pub matrix_width: u32,
    pub matrix_height: u32,
}
impl Default for WMTSImageryLayerOptions {
    fn default() -> Self {
        Self {
            name: "".to_string(),
            url: "".to_string(),
            layer: "".to_string(),
            style: "".to_string(),
            format: "".to_string(),
            matrix_set: "".to_string(),
            crs: "".to_string(),
            min_zoom: 0,
            max_zoom: 0,
            tile_width: 0,
            tile_height: 0,
            tile_matrix_labels: vec![],
            tile_matrix_set: vec![],
            tiling_scheme: GeographicTilingScheme::default(),
        }
    }
}
