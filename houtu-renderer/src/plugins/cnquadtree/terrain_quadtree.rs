use bevy::prelude::Component;
use houtu_scene::{GeographicTilingScheme, TilingScheme};

use super::{
    direction::Direction, node_id::NodeId, terrain_quadtree_internal::TerrainQuadtreeInternal,
    terrain_quadtree_node::TerrainQuadtreeNode, Quadrant,
};
#[derive(Component, Debug, Clone)]
pub struct TerrainQuadtree {
    root: TerrainQuadtreeNode,
    pub(super) internals: Vec<TerrainQuadtreeInternal>,
    tiling_scheme: GeographicTilingScheme,
}
impl Default for TerrainQuadtree {
    fn default() -> Self {
        Self::new()
    }
}
impl TerrainQuadtree {
    pub fn new() -> Self {
        let mut tree = Self {
            root: TerrainQuadtreeNode::None,
            internals: Vec::<TerrainQuadtreeInternal>::new(),
            tiling_scheme: GeographicTilingScheme::default(),
        };
        tree.internals.shrink_to_fit();
        tree
    }

    pub(crate) fn set_parent(&mut self, node: TerrainQuadtreeNode, parent: TerrainQuadtreeNode) {
        use TerrainQuadtreeNode::*;
        match node {
            Internal(index) => {
                self.internals[index].parent = parent;
            }
            _ => unreachable!(),
        }
    }

    // Returns Parent of node or node if node is root
    pub(crate) fn get_parent(&self, node: TerrainQuadtreeNode) -> TerrainQuadtreeNode {
        match node {
            TerrainQuadtreeNode::Internal(index) => self.internals[index].parent,
            TerrainQuadtreeNode::None => node,
        }
    }
    // Returns kth-parent of node or root node if one of the ancestors is the root
    pub(crate) fn get_kth_parent(&self, node: TerrainQuadtreeNode, k: u8) -> TerrainQuadtreeNode {
        let mut parent;

        match node {
            TerrainQuadtreeNode::Internal(index) => parent = self.internals[index].parent,
            _ => unreachable!(),
        }

        for i in 1..k {
            match parent {
                TerrainQuadtreeNode::Internal(index) => parent = self.internals[index].parent,
                TerrainQuadtreeNode::None => return parent,
                _ => unreachable!(),
            }
        }
        parent
    }

    pub(crate) fn new_node(
        &mut self,
        node_id: NodeId,
        location: Quadrant,
        parent: TerrainQuadtreeNode,
    ) -> TerrainQuadtreeNode {
        self.internals.push(TerrainQuadtreeInternal {
            parent: parent,
            id: node_id,
            rectangle: self
                .tiling_scheme
                .tile_x_y_to_rectange(node_id.x, node_id.y, node_id.level),
            children: Default::default(),
            location: location,
        });
        TerrainQuadtreeNode::Internal(self.internals.len() - 1)
    }
    pub(crate) fn subdivide(&mut self, node_id: TerrainQuadtreeNode) {
        if let TerrainQuadtreeNode::Internal(index) = node_id {
            let node = &self.internals[index];
            let southwest = node.id.southwest();
            let southeast = node.id.southeast();
            let northwest = node.id.northwest();
            let northeast = node.id.northeast();
            let nw = self.new_node(southwest, Quadrant::Southwest, node_id);
            let ne = self.new_node(southeast, Quadrant::Southeast, node_id);
            let sw = self.new_node(northwest, Quadrant::Northwest, node_id);
            let se = self.new_node(northeast, Quadrant::Northeast, node_id);
            node.children.northwest = nw;
            node.children.northeast = ne;
            node.children.southwest = sw;
            node.children.southeast = se;
        }
    }
}
