use bevy::{
    asset::AssetLoader,
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task},
    utils::HashMap,
};
#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct NoExtensionAsset {}
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum NoExtensionAssetLoaderError {
    /// An [IO](std::io) Error
    #[error("Could load shader: {0}")]
    Io(#[from] std::io::Error),
    /// A [RON](ron) Error
    #[error("Could not parse RON: {0}")]
    RonSpannedError(#[from] ron::error::SpannedError),
}
#[derive(Default)]
pub struct NoExtensionAssetLoader;

impl AssetLoader for NoExtensionAssetLoader {
    type Assets = NoExtensionAsset;
    type Settings = ();
    type Error = NoExtensionAssetLoaderError;
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::utils::BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        
    }
    fn extensions(&self) -> &[&str] {
        &["noextension"]
    }
}
