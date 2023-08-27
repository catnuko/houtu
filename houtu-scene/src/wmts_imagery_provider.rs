use std::collections::HashMap;

use crate::{geographic_tiling_scheme::GeographicTilingScheme, Rectangle};
// pub type String = &'static str;

pub struct WMTSImageryLayerProviderOptions {
    pub name: Option<&'static str>,
    pub url: &'static str,
    pub layer: &'static str,
    pub style: &'static str,
    pub format: Option<&'static str>,
    pub crs: Option<&'static str>,
    pub minimum_level: Option<u8>,
    pub maximum_level: Option<u8>,
    pub tile_width: Option<u32>,
    pub tile_height: Option<u32>,
    pub tile_matrix_labels: Option<Vec<&'static str>>,
    pub tile_matrix_set_id: &'static str,
    pub tiling_scheme: Option<GeographicTilingScheme>,
    pub subdomains: Option<Vec<&'static str>>,
    pub rectangle: Option<Rectangle>,
}
#[derive(Debug, Clone)]
pub struct WMTSImageryLayerProvider {
    pub name: String,
    pub url: String,
    pub layer: String,
    pub style: String,
    pub format: String,
    // pub matrix_set: String,
    // pub crs: String,
    pub tile_matrix_set_id: String,
    pub minimum_level: u8,
    pub maximum_level: u8,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_matrix_labels: Option<Vec<String>>,
    pub tiling_scheme: GeographicTilingScheme,
    pub subdomains: Vec<String>,
}
impl WMTSImageryLayerProvider {
    pub fn new(options: WMTSImageryLayerProviderOptions) -> Self {
        let subdomains: Vec<String> = {
            if let Some(real_subdomains) = options.subdomains {
                real_subdomains.iter().map(|x| x.to_string()).collect()
            } else {
                vec!["a".to_string(), "b".to_string(), "c".to_string()]
            }
        };
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
            tiling_scheme: options
                .tiling_scheme
                .unwrap_or(GeographicTilingScheme::default()),
            subdomains: subdomains,
        }
    }
    pub fn get_request_body(&self, col: u32, row: u32, level: u32) -> HashMap<String, String> {
        let tile_matrix;
        if let Some(real_labels) = &self.tile_matrix_labels {
            tile_matrix = real_labels[level as usize].clone();
        } else {
            tile_matrix = level.to_string();
        }
        let mut query: HashMap<String, String> = HashMap::new();
        query.insert("tilematrix".to_string(), tile_matrix);
        query.insert("layer".to_string(), self.layer.clone());
        query.insert("style".to_string(), self.style.clone());
        query.insert("tilerow".to_string(), row.to_string());
        query.insert("tilecol".to_string(), col.to_string());
        query.insert("tilematrixset".to_string(), self.tile_matrix_set_id.clone());
        query.insert("format".to_string(), self.format.clone());
        return query;
    }
}
