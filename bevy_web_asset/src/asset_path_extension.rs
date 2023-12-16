use bevy::{asset::AssetPath, prelude::*};
/// In order not to include the extension in the path, but to make use of the extension's decoder
///
/// with some extension
/// ```
/// let path = AssetPathExtension{
///     path:"https://xxxx.com/test/a".to_string(),
///     extension:Some("png".to_string()),
/// };
/// // request url is https://xxxx.com/test/a.png
/// let bevy_asset_path = "https://xxxx.com/test/a.noextension.png";
/// let string:String = path.into();
/// assert!(string,bevy_asset_path);
/// ```
/// no extension equal to bevy asset path
/// ```
/// let path = AssetPathExtension{
///     path:"https://xxxx.com/test/a.png".to_string(),
///     extension:None,
/// };
/// // request url is https://xxxx.com/test/a.png
/// let bevy_asset_path = "https://xxxx.com/test/a.png";
/// let string:String = path.into();
/// assert!(string,bevy_asset_path);
/// ```
pub struct AssetPathExtension {
    /// bevy asset path
    pub path: String,
    /// enable No extension
    pub extension: Option<String>,
}
impl AssetPathExtension {
    pub fn from_png(path: String) -> Self {
        return AssetPathExtension {
            path: path,
            extension: Some("png".to_string()),
        };
    }
}
impl Into<String> for AssetPathExtension {
    fn into(self) -> String {
        if let Some(extension) = self.extension {
            return format!("{}.noextension.{}", self.path, extension);
        } else {
            return self.path;
        }
    }
}
impl From<AssetPathExtension> for AssetPath<'static> {
    #[inline]
    fn from(houtu_asset_path: AssetPathExtension) -> Self {
        let string: String = houtu_asset_path.into();
        AssetPath::parse(string.as_str()).into_owned()
    }
}
