use crate::{
    geographic_tiling_scheme::GeographicTilingScheme, imagery_provider::ImageryProvider,
    tiling_scheme::TilingScheme, wmts_imagery_layer::WMTSImageryLayerOptions,
};

#[derive(Debug, Clone)]
pub struct WMTSImageryLayerProvider<T = GeographicTilingScheme>
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

impl ImageryProvider for WMTSImageryLayerProvider {
    fn requestImage(x: f64, y: f64, z: f64) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
impl WMTSImageryLayerProvider<T>
where
    T: TilingScheme,
{
    pub fn from_wmts_imagery_layer_option(options: WMTSImageryLayerOptions) -> Self {
        Self {
            name: options.name,
            url: options.url,
            layer: options.layer,
            style: options.style,
            format: options.format,
            matrix_set: options.matrix_set,
            crs: options.crs,
            min_zoom: options.min_zoom,
            max_zoom: options.max_zoom,
            tile_width: options.tile_width,
            tile_height: options.tile_height,
            tile_matrix_labels: options.tile_matrix_labels,
            tile_matrix_set: options.tile_matrix_set,
            tiling_scheme: options.tiling_scheme,
        }
    }
}
