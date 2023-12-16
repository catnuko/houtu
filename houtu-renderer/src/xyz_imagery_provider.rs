use std::collections::HashMap;

use bevy::prelude::warn;
use houtu_scene::{GeographicTilingScheme, Rectangle, TilingScheme, WebMercatorTilingScheme};
use new_string_template::template::Template;

use crate::{quadtree::imagery_provider::ImageryProvider, utils::get_subdomain};

pub struct XYZImageryProvider {
    pub tiling_scheme: Box<dyn TilingScheme>,
    pub rectangle: Rectangle,
    pub url: &'static str,
    pub subdomains: Option<Vec<&'static str>>,
    pub minimum_level: u32,
    pub maximum_level: u32,
    pub ready: bool,
    pub tile_width: u32,
    pub tile_height: u32,
}
impl Default for XYZImageryProvider {
    fn default() -> Self {
        let tiling_scheme = Box::new(WebMercatorTilingScheme::default());
        let rectangle = tiling_scheme.get_rectangle();
        Self {
            tiling_scheme: tiling_scheme,
            rectangle: rectangle,
            url: "",
            subdomains: None,
            minimum_level: 0,
            maximum_level: 17,
            ready: true,
            tile_width: 256,
            tile_height: 256,
        }
    }
}
impl ImageryProvider for XYZImageryProvider {
    fn get_maximum_level(&self) -> u32 {
        return 17;
    }
    fn get_minimum_level(&self) -> u32 {
        self.minimum_level
    }
    fn get_ready(&self) -> bool {
        self.ready
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
        self.tile_height
    }
    fn get_tile_width(&self) -> u32 {
        self.tile_width
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
        let template = Template::new(self.url);
        let mut args = HashMap::new();
        if let Some(subdomains) = self.subdomains.as_ref() {
            let subdomain = get_subdomain(&subdomains, key);
            args.insert("s", subdomain);
        }
        let level = key.level.to_string();
        let x = key.x.to_string();
        let y = key.y.to_string();
        args.insert("z", level.as_str());
        args.insert("x", x.as_str());
        args.insert("y", y.as_str());
        if let Ok(url) = template.render(&args) {
            bevy::log::info!("img url = {}", url);
            let image = asset_server.load(url);
            // let image = asset_server.load("icon.png");
            return Some(image);
        } else {
            warn!("extected a tile url");
            return None;
        }
    }
}
#[cfg(test)]
mod tests {
    use new_string_template::template::Template;
    use std::collections::HashMap;
    #[test]
    fn test_template_url() {
        let template = Template::new("https://{s}.tile.thunderforest.com/cycle/{z}/{x}/{y}.png");
        let mut args = HashMap::new();
        args.insert("s", "t1");
        args.insert("z", "10");
        args.insert("x", "20");
        args.insert("y", "30");
        let s = template.render(&args).expect("Expected Result to be Ok");
        assert!(s == "https://t1.tile.thunderforest.com/cycle/10/20/30.png");
    }
}
