use bevy::prelude::Component;
use houtu_scene::{GeographicTilingScheme, TilingScheme};

use crate::plugins::tileset::TileKey;

use super::{
    direction::Direction, tile_node::TileNode, tile_node_internal::TileNodeInternal, Quadrant,
};
#[derive(Component, Debug, Clone)]
pub struct TileTree {
    root: TileNode,
    pub(super) internals: Vec<TileNodeInternal>,
}
impl Default for TileTree {
    fn default() -> Self {
        Self::new()
    }
}
impl TileTree {
    pub fn new() -> Self {
        let mut tree = Self {
            root: TileNode::None,
            internals: Vec::<TileNodeInternal>::new(),
        };
        tree.internals.shrink_to_fit();
        tree
    }

    pub(crate) fn set_parent(&mut self, node: TileNode, parent: TileNode) {
        use TileNode::*;
        match node {
            Internal(index) => {
                self.internals[index].parent = parent;
            }
            _ => unreachable!(),
        }
    }
    pub fn get_root_len(&self) -> usize {
        self.internals
            .iter()
            .filter(|x| x.location == Quadrant::Root)
            .count()
    }

    // Returns Parent of node or node if node is root
    pub(crate) fn get_parent(&self, node: TileNode) -> TileNode {
        match node {
            TileNode::Internal(index) => self.internals[index].parent,
            TileNode::None => node,
        }
    }
    // Returns kth-parent of node or root node if one of the ancestors is the root
    pub(crate) fn get_kth_parent(&self, node: TileNode, k: u8) -> TileNode {
        let mut parent;

        match node {
            TileNode::Internal(index) => parent = self.internals[index].parent,
            _ => unreachable!(),
        }

        for i in 1..k {
            match parent {
                TileNode::Internal(index) => parent = self.internals[index].parent,
                TileNode::None => return parent,
                _ => unreachable!(),
            }
        }
        parent
    }
    pub(crate) fn new_node(
        &mut self,
        key: TileKey,
        location: Quadrant,
        parent: TileNode,
    ) -> TileNode {
        self.internals.push(TileNodeInternal {
            parent: parent,
            key,
            rectangle: self
                .tiling_scheme
                .tile_x_y_to_rectange(key.x, key.y, key.level),
            children: Default::default(),
            location: location,
        });
        TileNode::Internal(self.internals.len() - 1)
    }
    pub(crate) fn subdivide(&mut self, node_id: TileNode) {
        if let TileNode::Internal(index) = node_id {
            let node = &self.internals[index];
            let southwest = node.key.southwest();
            let southeast = node.key.southeast();
            let northwest = node.key.northwest();
            let northeast = node.key.northeast();
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
    pub(crate) fn shouldSubDivide(&mut self) -> bool {
        return true;
    }
}
