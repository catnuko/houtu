use std::{collections::VecDeque, fmt};

use super::tile_z::Tile;

pub struct TileReplacementQueue {
    pub queue: VecDeque<Tile>,
}
impl TileReplacementQueue {
    pub fn markTileRendered(&mut self, tile: &mut Tile) {
        // self.queue.
    }
}
