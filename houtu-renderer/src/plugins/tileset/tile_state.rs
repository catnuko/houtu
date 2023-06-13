use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub enum TileState {
    Start,
    Loading,
    Done,
    Failed,
}
impl Default for TileState {
    fn default() -> Self {
        Self::Start
    }
}
