use super::{quadtree::Quadtree, quadtree_node::QuadtreeNode, quadtree_value::QuadtreeValue};

#[derive(Debug)]
pub struct QuadtreeStats {
    pub num_nodes: usize,
    pub num_values: usize,
    pub average_depth: f32,
    pub average_num_values: f32,
}

impl QuadtreeStats {
    // calcuates common statistics about a quadtree
    pub fn calculate<T: QuadtreeValue>(quadtree: &Quadtree<T>) -> QuadtreeStats {
        // functions
        let count_children_fn: fn(&QuadtreeNode<T>) -> usize = |node| node.children.len();
        let count_values_fn: fn(&QuadtreeNode<T>) -> usize = |node| node.values.len();
        let total_depth_fn: fn(&QuadtreeNode<T>) -> f32 = |node| node.depth as f32;
        let num_nodes = quadtree.root.aggregate_statistic(&count_children_fn);
        let num_values = quadtree.root.aggregate_statistic(&count_values_fn);
        let average_depth = quadtree.root.aggregate_statistic(&total_depth_fn) / (num_nodes as f32).max(1.);
        let average_num_values = num_values as f32 / (num_nodes as f32).max(1.);
        QuadtreeStats {
            num_nodes,
            num_values,
            average_depth,
            average_num_values,
        }
    }

    pub fn print(&self) {
        println!("{:?}", self);
    }
}
