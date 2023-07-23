use std::{collections::VecDeque, fmt};

use super::tile_z::Tile;

pub struct TileReplacementQueue {
    pub queue: VecDeque<Tile>,
}
impl TileReplacementQueue {
    pub fn mark_tile_rendered(&mut self, tile: &mut Tile) {
        // self.queue.
    }
}
