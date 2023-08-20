use houtu_scene::{Rectangle, TilingScheme};

use crate::quadtree::imagery_provider::ImageryProvider;

pub struct XYZImageryProvider {
    pub tiling_scheme: Box<dyn TilingScheme>,
    pub rectangle: Rectangle,
    // pub url: &'static str,
}

impl XYZImageryProvider {
    pub fn new(tiling_scheme: Box<dyn TilingScheme>) -> Self {
        let rectangle = tiling_scheme.get_rectangle();
        Self {
            tiling_scheme,
            rectangle: rectangle,
            // url: url,
        }
    }
}
impl ImageryProvider for XYZImageryProvider {
    fn get_maximum_level(&self) -> u32 {
        return 17;
    }
    fn get_minimum_level(&self) -> u32 {
        0
    }
    fn get_ready(&self) -> bool {
        true
    }
    fn get_rectangle(&self) -> &houtu_scene::Rectangle {
        &self.rectangle
    }
    fn get_tile_credits(
        &self,
        key: &crate::quadtree::tile_key::TileKey,
    ) -> Option<Vec<crate::quadtree::credit::Credit>> {
        None
    }
    fn get_tile_height(&self) -> u32 {
        256
    }
    fn get_tile_width(&self) -> u32 {
        256
    }
    fn get_tiling_scheme(&self) -> &Box<dyn TilingScheme> {
        &self.tiling_scheme
    }
    fn load_image(&self, url: String) {}
    fn pick_features(
        &self,
        key: &crate::quadtree::tile_key::TileKey,
        longitude: f64,
        latitude: f64,
    ) {
    }
    fn request_image(
        &self,
        key: &crate::quadtree::tile_key::TileKey,
        asset_server: &bevy::prelude::AssetServer,
    ) -> Option<bevy::prelude::Handle<bevy::prelude::Image>> {
        // bevy::log::info!("xyz imagery provider is requeting image for tile {:?}", key);
        // let image = asset_server.load(format!(
        //     "https://maps.omniscale.net/v2/houtuearth-4781e785/style.default/{}/{}/{}.png",
        //     key.level, key.x, key.y,
        // ));
        let image = asset_server.load("icon.png");
        return Some(image);
    }
}
