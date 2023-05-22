mod wmts;
use bevy::prelude::*;
pub use wmts::*;

pub struct WMTSPlugin;
impl Plugin for WMTSPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WMTS::new(WMTSOptions {
            url: "http://t0.tianditu.gov.cn/img_c/wmts?tk=b931d6faa76fc3fbe622bddd6522e57b",
            layer: "img",
            format: Some("tiles"),
            tile_matrix_set_id: "c",
            ..Default::default()
        }));
    }
}
