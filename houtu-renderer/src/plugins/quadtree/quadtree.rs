use super::{quadtree_node::QuadtreeNode, quadtree_value::QuadtreeValue, QuadtreeNodeTrait};

pub struct Quadtree<T: QuadtreeNodeTrait> {
    pub roots: Vec<T>,
}

impl<T: QuadtreeNodeTrait> Quadtree<T> {
    pub fn empty() -> Self {
        Quadtree { roots: Vec::new() }
    }

    pub fn add_node(&mut self, node: T) {
        self.roots.push(node)
    }
}
impl<T: QuadtreeNodeTrait> Default for Quadtree<T> {
    fn default() -> Self {
        Self::empty()
    }
}
