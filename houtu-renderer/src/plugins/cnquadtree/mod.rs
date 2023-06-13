use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;

mod direction;
mod node_children;
mod node_id;
mod node_neighbours;
mod terrain_quadtree;
mod terrain_quadtree_internal;
mod terrain_quadtree_node;
pub use node_id::NodeId;
pub use terrain_quadtree::TerrainQuadtree;
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
    Root,
}
