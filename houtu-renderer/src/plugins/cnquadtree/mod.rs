use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;

mod direction;
mod node_children;
mod node_id;
mod node_neighbours;
mod terrain_quadtree;
mod terrain_quadtree_internal;
mod terrain_quadtree_leaf;
mod terrain_quadtree_node;
use self::direction::Direction;
use self::node_children::NodeChildren;
use self::node_neighbours::NodeNeighbours;
use self::terrain_quadtree_leaf::TerrainQuadtreeLeaf;

#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
    Root,
}
