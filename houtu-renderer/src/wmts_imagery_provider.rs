use bevy::prelude::*;
use std::collections::HashMap;

use houtu_scene::{GeographicTilingScheme, Rectangle, TileKey, TilingScheme};

use crate::quadtree::imagery_provider::ImageryProvider;
#[derive(Default)]
pub struct WMTSImageryProviderOptions {
    pub name: Option<&'static str>,
    pub url: &'static str,
    pub layer: &'static str,
    pub style: &'static str,
    pub format: Option<&'static str>,
    pub crs: Option<&'static str>,
    pub minimum_level: Option<u32>,
    pub maximum_level: Option<u32>,
    pub tile_width: Option<u32>,
    pub tile_height: Option<u32>,
    pub tile_matrix_labels: Option<Vec<&'static str>>,
    pub tile_matrix_set_id: &'static str,
    pub tiling_scheme: Option<Box<dyn TilingScheme>>,
    pub subdomains: Option<Vec<&'static str>>,
    pub rectangle: Option<Rectangle>,
}
pub struct WMTSImageryProvider {
    pub name: String,
    pub url: String,
    pub layer: String,
    pub style: String,
    pub format: String,
    pub tile_matrix_set_id: String,
    pub minimum_level: u32,
    pub maximum_level: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_matrix_labels: Option<Vec<String>>,
    pub tiling_scheme: Box<dyn TilingScheme>,
    pub subdomains: Vec<String>,
    pub rectangle: Rectangle,
}
impl ImageryProvider for WMTSImageryProvider {
    fn get_maximum_level(&self) -> u32 {
        self.maximum_level
    }
    fn get_minimum_level(&self) -> u32 {
        self.minimum_level
    }
    fn get_ready(&self) -> bool {
        true
    }
    fn get_tile_credits(
        &self,
        key: &crate::quadtree::tile_key::TileKey,
    ) -> Option<Vec<crate::quadtree::credit::Credit>> {
        None
    }
    fn get_tile_height(&self) -> u32 {
        self.tile_height
    }
    fn get_tile_width(&self) -> u32 {
        self.tile_width
    }
    fn request_image(
        &self,
        key: &crate::quadtree::tile_key::TileKey,
        asset_server: &AssetServer,
    ) -> Option<Handle<Image>> {
        // asset_server.load()
        None
    }
    fn load_image(&self, url: String) {}
    fn pick_features(
        &self,
        key: &crate::quadtree::tile_key::TileKey,
        longitude: f64,
        latitude: f64,
    ) {
    }
    fn get_tiling_scheme(&self) -> &Box<dyn TilingScheme> {
        &self.tiling_scheme
    }
    fn get_rectangle(&self) -> &Rectangle {
        &self.rectangle
    }
}
impl WMTSImageryProvider {
    pub fn new(options: WMTSImageryProviderOptions) -> Self {
        let subdomains: Vec<String> = {
            if let Some(real_subdomains) = options.subdomains {
                real_subdomains.iter().map(|x| x.to_string()).collect()
            } else {
                vec!["a".to_string(), "b".to_string(), "c".to_string()]
            }
        };
        let tiling_scheme = options
            .tiling_scheme
            .unwrap_or(Box::new(GeographicTilingScheme::default()));
        Self {
            name: {
                if let Some(v) = options.name {
                    v.to_string()
                } else {
                    "".to_string()
                }
            },
            url: options.url.to_string(),
            layer: options.layer.to_string(),
            style: options.style.to_string(),
            format: {
                if let Some(v) = options.format {
                    v.to_string()
                } else {
                    "image/jpeg".to_string()
                }
            },
            rectangle: options.rectangle.unwrap_or(tiling_scheme.get_rectangle()),
            // matrix_set: options.matrix_set,
            // crs: options.crs,
            tile_matrix_set_id: options.tile_matrix_set_id.to_string(),
            minimum_level: options.minimum_level.unwrap_or(0),
            maximum_level: options.maximum_level.unwrap_or(19),
            tile_width: options.tile_width.unwrap_or(256),
            tile_height: options.tile_height.unwrap_or(256),
            tile_matrix_labels: {
                if let Some(v) = options.tile_matrix_labels {
                    Some(v.iter().map(|x| x.to_string()).collect())
                } else {
                    None
                }
            },
            // tile_matrix_set: options.tile_matrix_set,
            tiling_scheme: tiling_scheme,
            subdomains: subdomains,
        }
    }
    pub fn getParams(&self, col: u32, row: u32, level: u32) -> HashMap<String, String> {
        let tileMatrix;
        if let Some(real_labels) = &self.tile_matrix_labels {
            tileMatrix = real_labels[level as usize].clone();
        } else {
            tileMatrix = level.to_string();
        }
        let mut query: HashMap<String, String> = HashMap::new();
        query.insert("tilematrix".to_string(), tileMatrix);
        query.insert("layer".to_string(), self.layer.clone());
        query.insert("style".to_string(), self.style.clone());
        query.insert("tilerow".to_string(), row.to_string());
        query.insert("tilecol".to_string(), col.to_string());
        query.insert("tilematrixset".to_string(), self.tile_matrix_set_id.clone());
        query.insert("format".to_string(), self.format.clone());
        return query;
    }
    pub fn getParamsByTileKey(&self, tile_key: &TileKey) -> HashMap<String, String> {
        return self.getParams(tile_key.column, tile_key.row, tile_key.level);
    }
    pub fn build_url(&self, tile_key: &TileKey) -> String {
        let params = self.getParamsByTileKey(tile_key);
        let mut params_str = String::new();
        params.iter().for_each(|(k, v)| {
            let param = format!("{}={}", k, v);
            if params_str == "" {
                params_str = format!("{}", param);
            } else {
                params_str = format!("{}&{}", params_str, param);
            }
        });
        return format!("{}&{}", self.url, params_str);
    }
}
