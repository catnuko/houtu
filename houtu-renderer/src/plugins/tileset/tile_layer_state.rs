use bevy::prelude::*;
#[derive(Component, Debug, Clone)]
pub enum TileLayerState {
    Start,
    Loading,
}
impl Default for TileLayerState {
    fn default() -> Self {
        Self::Start
    }
}
