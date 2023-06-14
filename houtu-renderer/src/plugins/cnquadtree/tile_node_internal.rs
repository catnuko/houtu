use core::fmt;

use bevy::prelude::Entity;
use houtu_scene::{Cartographic, Rectangle};

use super::{
    node_children::NodeChildren, node_neighbours::NodeNeighbours, tile_node::TileNode, Quadrant,
};
use crate::plugins::tileset::TileKey;

#[derive(Clone, PartialEq, Debug)]
pub struct TileNodeInternal {
    pub(super) parent: TileNode,
    pub(super) location: Quadrant,
    pub(super) children: NodeChildren,
    pub(super) key: TileKey,
    pub(super) rectangle: Rectangle,
}
impl TileNodeInternal {
    pub fn contains_point(&self, point: &Cartographic) -> bool {
        self.rectangle.contains(point)
    }
}
// impl fmt::Debug for TileNodeInternal {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "({:?}, ({:?}, {:?}), GREY, {:?}, {:?}, ({:?}), ({:?}))",
//             self.level,
//             self.bounds.min,
//             self.bounds.max,
//             self.location,
//             self.parent,
//             self.children,
//             self.neighbours,
//         )
//     }
// }
