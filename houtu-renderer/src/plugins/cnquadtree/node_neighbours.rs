use super::direction::Direction;
use super::tile_node::TileNode;
use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;
#[derive(Clone, Copy, PartialEq)]
pub struct NodeNeighbours {
    pub(super) north: TileNode,
    pub(super) east: TileNode,
    pub(super) south: TileNode,
    pub(super) west: TileNode,
}

impl Default for NodeNeighbours {
    fn default() -> Self {
        Self {
            north: TileNode::None,
            east: TileNode::None,
            south: TileNode::None,
            west: TileNode::None,
        }
    }
}

impl Index<Direction> for NodeNeighbours {
    type Output = TileNode;

    fn index(&self, dir: Direction) -> &TileNode {
        match dir {
            Direction::North => &self.north,
            Direction::East => &self.east,
            Direction::South => &self.south,
            Direction::West => &self.west,
        }
    }
}

impl IndexMut<Direction> for NodeNeighbours {
    fn index_mut(&mut self, dir: Direction) -> &mut TileNode {
        match dir {
            Direction::North => &mut self.north,
            Direction::East => &mut self.east,
            Direction::South => &mut self.south,
            Direction::West => &mut self.west,
        }
    }
}
