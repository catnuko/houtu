use core::fmt;

use bevy::prelude::Entity;
use houtu_scene::{Cartographic, Rectangle};

use super::{
    node_children::NodeChildren, node_id::NodeId, node_neighbours::NodeNeighbours,
    terrain_quadtree_node::TerrainQuadtreeNode, Quadrant,
};

#[derive(Clone, PartialEq, Debug)]
pub struct TerrainQuadtreeInternal {
    pub(super) parent: TerrainQuadtreeNode,
    pub(super) location: Quadrant,
    pub(super) children: NodeChildren,
    pub(super) id: NodeId,
    pub(super) rectangle: Rectangle,
}
impl TerrainQuadtreeInternal {
    pub fn contains_point(&self, point: &Cartographic) -> bool {
        self.rectangle.contains(point)
    }
}
// impl fmt::Debug for TerrainQuadtreeInternal {
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
