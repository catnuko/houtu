use houtu_scene::GeographicTilingScheme;

use super::quadtree::{credit::Credit, imagery_provider::ImageryProvider};
#[derive(PartialEq)]
pub enum BingMapsStyle {
    Aerial = 0,
    AerialWithLabels = 1,
    AerialWithLabelsOnDemand = 2,
    Road = 3,
    RoadOnDemand = 4,
    CanvasDark = 5,
    CanvasLight = 6,
    CanvasGray = 7,
    OrdnanceSurvey = 9,
    CollinsBart = 10,
}
pub struct BingMapsImageryProvider {
    pub tiling_scheme: GeographicTilingScheme,
    pub map_style: BingMapsStyle,
    pub credit: Credit,
    pub tile_width: u32,
    pub tile_height: u32,
    pub maximum_level: u32,
    pub image_url_template: String,
    pub image_url_subdomains: Vec<u8>,
}
impl BingMapsImageryProvider {}

// impl ImageryProvider for BingMapsImageryProvider {
//     fn get_maximum_level(&self) -> u32 {}
// }
