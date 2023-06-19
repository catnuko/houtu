use bevy::prelude::*;
use houtu_scene::Rectangle;

use super::label;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {}
}
#[derive(Debug, Component)]
pub struct ImageryLayerCollectionMark;

#[derive(Debug, Bundle)]
pub struct ImageryLayerCollection {
    mark: ImageryLayerCollectionMark,
    visibility: Visibility,
}
impl ImageryLayerCollection {
    pub fn new() -> Self {
        Self {
            mark: ImageryLayerCollectionMark,
            visibility: Visibility::Visible,
        }
    }
    pub fn add_layer(commands: &mut Commands, collection_entity: Entity, layer: ImageryLayer) {
        let layer_entity = commands.spawn(layer).set_parent(collection_entity).id();
        commands.entity(collection_entity).add_child(layer_entity);
    }
    pub fn remove_layer(commands: &mut Commands, layer_entity: Entity) {
        commands.entity(layer_entity).despawn();
    }
    pub fn set_visibility(
        commands: &mut Commands,
        collection_entity: Entity,
        visibility: &mut Visibility,
        new_visibility: Visibility,
    ) {
        *visibility = new_visibility;
    }
    pub fn remove_all_children(commands: &mut Commands, collection_entity: Entity) {
        commands.entity(collection_entity).despawn_descendants();
    }
    pub fn remove_all_self(commands: &mut Commands, collection_entity: Entity) {
        commands.entity(collection_entity).despawn_recursive();
    }
}
#[derive(Debug, Component)]
pub struct ZIndex(u8);
#[derive(Debug, Component)]
pub struct ColorState {
    alpha: f64,
    nightAlpha: f64,
    dayAlpha: f64,
    brightness: f64,
    contrast: f64,
    hue: f64,
    saturation: f64,
    gamma: f64,
}
impl Default for ColorState {
    fn default() -> Self {
        Self {
            alpha: 1.0,
            nightAlpha: 1.0,
            dayAlpha: 1.0,
            brightness: 1.0,
            contrast: 1.0,
            hue: 0.0,
            saturation: 1.0,
            gamma: 1.0,
        }
    }
}
#[derive(Component, Debug, DerefMut, Deref)]
pub struct Ready(pub bool);

#[derive(Debug, Component)]
pub struct ImageryLayerOtherState {
    pub minimumTerrainLevel: Option<u32>,
    pub maximumTerrainLevel: Option<u32>,
    datasource: &'static str,
}
#[derive(Debug, Bundle)]
pub struct ImageryLayer {
    // z_index: ZIndex,
    color_state: ColorState,
    visibility: Visibility,
    rectangle: Rectangle,
    other_state: ImageryLayerOtherState,
    ready: Ready,
}
impl ImageryLayer {
    pub fn new(datasource: &'static str) -> Self {
        Self {
            color_state: ColorState::default(),
            visibility: Visibility::Inherited,
            rectangle: Rectangle::MAX_VALUE,
            other_state: ImageryLayerOtherState {
                minimumTerrainLevel: Some(0),
                maximumTerrainLevel: Some(31),
                datasource,
            },
            ready: Ready(false),
        }
    }
    // pub fn _createTileImagerySkeletons() -> bool {}
}
