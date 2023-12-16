use crate::{
    quadtree::{imagery_provider::ImageryProvider, tile_key::TileKey},
    utils::{get_subdomain, key_value_iter_to_param_str, map_to_param_str},
};
use bevy::prelude::*;
use bevy_web_asset::AssetPathExtension;
use houtu_scene::{GeographicTilingScheme, Rectangle, TilingScheme};
use new_string_template::template::Template;
use std::collections::HashMap;
use url::Url;
/// web map tile service iamgery provider
///
///
pub struct WMTSImageryProvider {
    pub name: &'static str,
    pub url: &'static str,
    pub layer: &'static str,
    pub style: &'static str,
    pub format: &'static str,
    pub tile_matrix_set_id: &'static str,
    pub minimum_level: u32,
    pub maximum_level: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tile_matrix_labels: Option<Vec<String>>,
    pub tiling_scheme: Box<dyn TilingScheme>,
    pub subdomains: Option<Vec<&'static str>>,
    pub rectangle: Rectangle,
    pub params: Option<Vec<(&'static str, &'static str)>>,
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
        return self
            .build_url(key)
            .and_then(|url| Some(asset_server.load(AssetPathExtension::from_png(url))));
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
    // pub fn new(options: WMTSImageryProviderOptions) -> Self {
    //     let subdomains: Vec<String> = {
    //         if let Some(real_subdomains) = options.subdomains {
    //             real_subdomains.iter().map(|x| x.to_string()).collect()
    //         } else {
    //             vec!["a".to_string(), "b".to_string(), "c".to_string()]
    //         }
    //     };
    //     let tiling_scheme = options
    //         .tiling_scheme
    //         .unwrap_or(Box::new(GeographicTilingScheme::default()));
    //     Self {
    //         name: {
    //             if let Some(v) = options.name {
    //                 v.to_string()
    //             } else {
    //                 "".to_string()
    //             }
    //         },
    //         url: options.url.to_string(),
    //         layer: options.layer.to_string(),
    //         style: options.style.to_string(),
    //         format: {
    //             if let Some(v) = options.format {
    //                 v.to_string()
    //             } else {
    //                 "image/jpeg".to_string()
    //             }
    //         },
    //         rectangle: options.rectangle.unwrap_or(tiling_scheme.get_rectangle()),
    //         // matrix_set: options.matrix_set,
    //         // crs: options.crs,
    //         tile_matrix_set_id: options.tile_matrix_set_id.to_string(),
    //         minimum_level: options.minimum_level.unwrap_or(0),
    //         maximum_level: options.maximum_level.unwrap_or(19),
    //         tile_width: options.tile_width.unwrap_or(256),
    //         tile_height: options.tile_height.unwrap_or(256),
    //         tile_matrix_labels: {
    //             if let Some(v) = options.tile_matrix_labels {
    //                 Some(v.iter().map(|x| x.to_string()).collect())
    //             } else {
    //                 None
    //             }
    //         },
    //         // tile_matrix_set: options.tile_matrix_set,
    //         tiling_scheme: tiling_scheme,
    //         subdomains: subdomains,
    //     }
    // }
    pub fn build_url(&self, key: &TileKey) -> Option<String> {
        let template = Template::new(self.url);
        let mut args = HashMap::new();
        if let Some(subdomains) = self.subdomains.as_ref() {
            let subdomain = get_subdomain(&subdomains, key);
            args.insert("s", subdomain);
        }
        let url_with_domain;
        if let Ok(url) = template.render(&args) {
            url_with_domain = url;
        } else {
            warn!("extected a tile url");
            return None;
        }
        let mut url_with_domain = Url::parse(&url_with_domain).unwrap();

        let tile_matrix;
        if let Some(real_labels) = &self.tile_matrix_labels {
            tile_matrix = real_labels[key.level as usize].clone();
        } else {
            tile_matrix = key.level.to_string();
        }
        let tilerow = key.y.to_string();
        let tilecol = key.x.to_string();
        let mut args: Vec<(&str, &str)> = vec![
            ("service", "wmts"),
            ("request", "GetTile"),
            ("version", "1.0.0"),
            ("tilematrix", &tile_matrix),
            ("layer", self.layer),
            ("style", self.style),
            ("tilerow", &tilerow),
            ("tilecol", &tilecol),
            ("tilematrixset", self.tile_matrix_set_id),
            ("format", self.format),
        ];
        if let Some(params) = self.params.as_ref() {
            let v = params.iter();
            v.for_each(|(k, v)| args.push((k, v)))
        }
        let params_str = key_value_iter_to_param_str(&args);
        url_with_domain.set_query(Some(params_str.as_str()));
        return Some(url_with_domain.as_str().to_string());
    }
}

#[cfg(test)]

mod tests {

    use super::*;
    #[test]
    fn test_url() {
        let mut url = Url::parse("https://example.net").unwrap();
        url.set_query(Some("key1=value1&key2=value2"));
        assert!(url.query() == Some("key1=value1&key2=value2"));
        let string = url.as_str();
        assert!(string == "https://example.net/?key1=value1&key2=value2");

        let a = "test";
        let b = &a;
        assert!(format!("{}", b) == "test".to_string());
    }
    #[test]
    fn test_build_url() {
        let tiling_scheme = GeographicTilingScheme::default();
        let rectangle = tiling_scheme.get_rectangle().clone();
        let provider = WMTSImageryProvider {
            name: "test",
            url: "https://{s}.tianditu.gov.cn/img_w/wmts",
            layer: "img_w",
            style: "default",
            format: "tiles",
            tile_matrix_set_id: "c",
            subdomains: Some(vec!["t0", "t1", "t2", "t3", "t4", "t5", "t6", "t7"]),
            tiling_scheme: Box::new(tiling_scheme),
            minimum_level: 0,
            maximum_level: 17,
            rectangle: rectangle,
            tile_width: 256,
            tile_height: 256,
            tile_matrix_labels: None,
            params: Some(vec![("tk", "xxxxxxxxxxxxxxxxxxxxxxxx")]),
        };
        let key = TileKey {
            x: 1,
            y: 2,
            level: 3,
        };
        let url = provider.build_url(&key).unwrap();
        let subdomain = get_subdomain(&provider.subdomains.unwrap(), &key);
        let expect_url = format!("https://{}.tianditu.gov.cn/img_w/wmts?service=wmts&request=GetTile&version=1.0.0&tilematrix=3&layer=img_w&style=default&tilerow=2&tilecol=1&tilematrixset=c&format=tiles&tk=xxxxxxxxxxxxxxxxxxxxxxxx", subdomain);
        assert!(url == expect_url);
    }
    #[test]
    fn test_build_url_no_subdomains() {
        let tiling_scheme = GeographicTilingScheme::default();
        let rectangle = tiling_scheme.get_rectangle().clone();
        let provider = WMTSImageryProvider {
            name: "test",
            url: "https://t0.tianditu.gov.cn/img_w/wmts",
            layer: "img_w",
            style: "default",
            format: "tiles",
            tile_matrix_set_id: "c",
            subdomains: None,
            tiling_scheme: Box::new(tiling_scheme),
            minimum_level: 0,
            maximum_level: 17,
            rectangle: rectangle,
            tile_width: 256,
            tile_height: 256,
            tile_matrix_labels: None,
            params: None,
        };
        let key = TileKey {
            x: 1,
            y: 2,
            level: 3,
        };
        let url = provider.build_url(&key).unwrap();
        assert!(url == "https://t0.tianditu.gov.cn/img_w/wmts?service=wmts&request=GetTile&version=1.0.0&tilematrix=3&layer=img_w&style=default&tilerow=2&tilecol=1&tilematrixset=c&format=tiles");
    }
}
