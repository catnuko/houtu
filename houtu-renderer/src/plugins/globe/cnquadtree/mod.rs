use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;

mod direction;
mod node_children;
mod node_neighbours;
mod tile_node;
mod tile_node_internal;
mod tile_tree;
pub use tile_tree::TileTree;
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
    Root,
}
