#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

mod web_asset_plugin;
mod web_asset_source;
mod asset_path_extension;

pub use web_asset_plugin::WebAssetPlugin;
pub use web_asset_source::WebAssetReader;
pub use asset_path_extension::*;
