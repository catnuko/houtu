use crate::{
    geographic_tiling_scheme::GeographicTilingScheme, tiling_scheme::TilingScheme, Rectangle,
};
pub type sstr = &'static str;

pub struct WMTSImageryLayerProviderOptions {
    pub name: Option<sstr>,
    pub url: sstr,
    pub layer: sstr,
    pub style: sstr,
    pub format: Option<sstr>,
    pub crs: Option<String>,
    pub minimumLevel: Option<u8>,
    pub maximumLevel: Option<u8>,
    pub tile_width: Option<u32>,
    pub tile_height: Option<u32>,
    pub tile_matrix_labels: Option<Vec<sstr>>,
    pub tile_matrix_set_id: Option<sstr>,
    pub tiling_scheme: Option<GeographicTilingScheme>,
    pub subdomains: Option<Vec<sstr>>,
    pub rectangle: Option<Rectangle>,
}
#[derive(Debug, Clone)]
pub struct WMTSImageryLayerProvider {
    pub name: sstr,
    pub url: sstr,
    pub layer: sstr,
    pub style: sstr,
    pub format: sstr,
    pub matrix_set: sstr,
    pub crs: sstr,
    pub minimumLevel: u8,
    pub maximumLevel: u8,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_matrix_labels: Option<Vec<sstr>>,
    pub tiling_scheme: GeographicTilingScheme,
    pub subdomains: Vec<sstr>,
}
impl WMTSImageryLayerProvider {
    pub fn from_wmts_imagery_layer_option(options: WMTSImageryLayerProviderOptions) -> Self {
        let subdomains: Vec<sstr> = {
            if let Some(real_subdomains) = options.subdomains {
                real_subdomains.clone()
            } else {
                vec!["a", "b", "c"]
            }
        };
        Self {
            name: options.name.unwrap_or(""),
            url: options.url,
            layer: options.layer,
            style: options.style,
            format: options.format.unwrap_or("image/jpeg"),
            // matrix_set: options.matrix_set,
            // crs: options.crs,
            minimumLevel: options.minimumLevel.unwrap_or(0),
            maximumLevel: options.maximumLevel.unwrap_or(19),
            tile_width: options.tile_width.unwrap_or(256),
            tile_height: options.tile_height.unwrap_or(256),
            tile_matrix_labels: options.tile_matrix_labels,
            // tile_matrix_set: options.tile_matrix_set,
            tiling_scheme: options
                .tiling_scheme
                .unwrap_or(GeographicTilingScheme::default()),
            subdomains: subdomains,
        }
    }
    pub fn requestImage(&self, col: u32, row: u32, level: u32) {
        let labels = self.tile_matrix_labels;
        let tileMatrix = {
            if let Some(real_labels) = labels {
                real_labels[level as usize]
            } else {
                level.to_string().as_str()
            }
        };
        let subdomains = self.subdomains;
    }
}
