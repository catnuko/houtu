use crate::{
    imagery_layer::{ImageLayer, ImageLayerState},
    imagery_layer_collection::ImageryLayerCollection,
    wmts_imagery_layer::{WMTSImageryLayer, WMTSImageryLayerOptions},
};

use bevy::prelude::*;
pub struct ImageryLayerPlugin {}
impl bevy::app::Plugin for ImageryLayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ImageryLayerCollection::default());
        app.add_startup_system(setup);
        app.add_system(hanlde_create_imagery_layer);
        app.add_system(handle_render_imagery_layer);
    }
}
impl Default for ImageryLayerPlugin {
    fn default() -> Self {
        Self {}
    }
}
pub fn setup(
    mut commands: Commands,
    mut layers: ResMut<ImageryLayerCollection>,
    mut create_layer_event_writer: EventWriter<rgis_events::CreateLayerEvent>,
) {
    create_layer_event_writer.send(houtu_events::CreateLayerEvent {
        name: "天地图WMTS".to_string(),
        url: "http://t0.tianditu.gov.cn/img_c/wmts?tk=5aaa55b9147f14d9e34f00f1a110e9b9".to_string(),
        ogc_type: "WMTS".to_string(),
        source_crs: "EPSG:4326".to_string(),
    });
}
fn hanlde_create_imagery_layer(
    mut create_layer_events: ResMut<bevy::ecs::event::Events<rgis_events::CreateLayerEvent>>,
    mut layer_created_event_writer: EventWriter<rgis_events::LayerCreatedEvent>,
    mut layers: ResMut<ImageryLayerCollection>,
) {
    for event in create_layer_events.drain() {
        let new_layer = match event.ogc_type {
            "WMTS" => Some(WMTSImageryLayer::new(WMTSImageryLayerOptions {
                url: event.url,
                name: event.name,
                ..Default::default()
            })),
            _ => None,
        };
        if let Some(layer) = new_layer {
            layer_created_event_writer.send(houtu_events::LayerCreatedEvent(
                layer.image_layer.id.clone(),
            ));
            layers.add_layer(layer);
        } else {
            println!("Unknow ogc_type: {}", event.ogc_type);
        }
    }
}
fn handle_render_imagery_layer(mut layers: ResMut<ImageryLayerCollection>) {
    for layer in layers.layers.iter_mut() {
        render_imagery_layer(layer)
    }
}
fn render_imagery_layer(layer: &mut WMTSImageryLayer) {
    if layer.image_layer.state == ImageLayerState::UNLOADED {
        if !layer.imagery_provider.ready() {
            return;
        }
        if(layer.level_zero_tiles.is_none()){
            layer.level_zero_tiles = Some(layer.imagery_provider.getLevelZeroTiles());
        }
        layer.image_layer.state = ImageLayerState::TRANSITIONING;
        layer.imagery_provider.requestImage(0.0, 0.0, 0.0);
    }
}
