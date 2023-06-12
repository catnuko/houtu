use std::fmt;

use houtu_scene::{Cartographic, Rectangle};

use super::{
    node_id::NodeId, node_neighbours::NodeNeighbours, terrain_quadtree_node::TerrainQuadtreeNode,
    Quadrant,
};

#[derive(Clone, PartialEq)]
pub struct TerrainQuadtreeLeaf {
    pub(super) parent: TerrainQuadtreeNode,
    pub(super) location: Quadrant,
    pub(super) neighbours: NodeNeighbours,
    pub(super) id: NodeId,
    pub(super) rectangle: Rectangle,
}

impl TerrainQuadtreeLeaf {
    pub fn contains_point(&self, point: &Cartographic) -> bool {
        self.rectangle.contains(point)
    }

    pub fn origin(self) -> [f32; 2] {
        self.bounds.center()
    }

    pub fn half_extents(self) -> [f32; 2] {
        self.bounds.half_extents()
    }

    pub fn level(self) -> u8 {
        self.level
    }
    pub fn get_neighbours(&self) -> NodeNeighbours {
        self.neighbours
    }

    // TODO: Improve frustum culling
    pub fn check_visibility(self, a: [f32; 2], b: [f32; 2]) -> bool {
        return true;
        let C = self.origin();
        (b[0] - a[0]) * (C[1] - a[1]) - (C[0] - a[0]) * (b[1] - a[1]) >= 0.0
    }
}
// impl fmt::Debug for TerrainQuadtreeLeaf {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(
//             f,
//             "({:?}, WHITE, {:?}, {:?}, (#, #, #, #), ({:?}))",
//             self.level, self.location, self.parent, self.neighbours
//         )
//     }
// }
