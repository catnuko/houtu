#[cfg(not(target_arch = "wasm32"))]
use bevy::asset::FileAssetIo;
use bevy::prelude::*;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

use super::filesystem_watcher::FilesystemWatcher;
use super::WebAssetIo;

/// Add this plugin to bevy to support loading http and https urls.
///
/// Needs to be added before Bevy's `DefaultPlugins`.
/// Also, make sure `AssetPlugin` is not loaded through `DefaultPlugins`.
///
/// # Example
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_web_asset::WebAssetPlugin;
///
/// let mut app = App::new();
/// // The web asset plugin should be added instead of the `AssetPlugin`
/// // Internally, WebAssetPlugin will create an AssetPlugin and hook into
/// // it in the right places
/// app.add_plugins(WebAssetPlugin::default());
/// app.add_plugins(DefaultPlugins.build().disable::<AssetPlugin>());
/// ```
///});
#[derive(Default)]
pub struct WebAssetPlugin {
    /// Settings for the underlying (regular) AssetPlugin
    pub asset_plugin: AssetPlugin,
}

impl Plugin for WebAssetPlugin {
    fn build(&self, app: &mut App) {
        // First, configure the underlying plugin
        // We use out own watcher, so `watch_for_changes` is always false
        let asset_plugin = AssetPlugin {
            asset_folder: self.asset_plugin.asset_folder.clone(),
            watch_for_changes: None,
        };

        // Create the `FileAssetIo` wrapper
        let asset_io = {
            // This makes calling `WebAssetIo::watch_for_changes` redundant
            let filesystem_watcher = match self.asset_plugin.watch_for_changes {
                Some(_) => Arc::new(RwLock::new(Some(FilesystemWatcher::default()))),
                None => Arc::new(RwLock::new(None)),
            };

            // Create the `FileAssetIo`
            let default_io = asset_plugin.create_platform_default_asset_io();

            // The method doesn't change, so we just use `FileAssetIo`'s
            #[cfg(not(target_arch = "wasm32"))]
            let root_path = FileAssetIo::get_base_path().join(&self.asset_plugin.asset_folder);
            #[cfg(target_arch = "wasm32")]
            let root_path = PathBuf::new();
            WebAssetIo {
                default_io,
                root_path,
                filesystem_watcher,
            }
        };

        // Add the asset server with our `WebAssetIo` wrapping `FileAssetIo`
        app.insert_resource(AssetServer::new(asset_io));

        // Add the asset plugin
        app.add_plugins(asset_plugin);
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Optionally add the filesystem watcher system
            if self.asset_plugin.watch_for_changes.is_some() {
                app.add_systems(
                    PostUpdate,
                    super::filesystem_watcher::filesystem_watcher_system,
                );
            }
        }
    }
}
