use bevy::prelude::*;

#[derive(Component, Clone)]
pub enum TileState {
    START = 0,
    LOADING = 1,
    READY = 2,
    UPSAMPLED_ONLY = 3,
    ERROR = 4,
}

impl Default for TileState {
    fn default() -> Self {
        return Self::START;
    }
}
