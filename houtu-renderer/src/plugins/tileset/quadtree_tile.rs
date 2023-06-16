use bevy::prelude::*;
use houtu_scene::GeographicTilingScheme;
use houtu_scene::{Rectangle, TilingScheme};
use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;
use std::sync::Arc;

use super::tile_replacement_queue::TileReplacementState;
use super::{tile_datasource::TilingSchemeWrap, TileKey};
#[derive(Component, Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum TileNode {
    /// None variant.
    None,
    /// Identifier of an internal node.
    Internal(usize),
}
#[derive(Component, Debug, PartialEq)]
pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
    Root,
}

#[derive(Clone, Copy, PartialEq, Debug, Component)]
pub struct NodeChildren {
    pub(super) northwest: TileNode,
    pub(super) northeast: TileNode,
    pub(super) southwest: TileNode,
    pub(super) southeast: TileNode,
}

impl Default for NodeChildren {
    fn default() -> Self {
        Self {
            northwest: TileNode::None,
            northeast: TileNode::None,
            southwest: TileNode::None,
            southeast: TileNode::None,
        }
    }
}

impl Index<Quadrant> for NodeChildren {
    type Output = TileNode;

    fn index(&self, quadrant: Quadrant) -> &TileNode {
        match quadrant {
            Quadrant::Northwest => &self.northwest,
            Quadrant::Northeast => &self.northeast,
            Quadrant::Southwest => &self.southwest,
            Quadrant::Southeast => &self.southeast,
            _ => unreachable!(),
        }
    }
}

impl IndexMut<Quadrant> for NodeChildren {
    fn index_mut(&mut self, quadrant: Quadrant) -> &mut TileNode {
        match quadrant {
            Quadrant::Northwest => &mut self.northwest,
            Quadrant::Northeast => &mut self.northeast,
            Quadrant::Southwest => &mut self.southwest,
            Quadrant::Southeast => &mut self.southeast,
            _ => unreachable!(),
        }
    }
}

impl IntoIterator for NodeChildren {
    type Item = TileNode;
    type IntoIter = ::std::vec::IntoIter<TileNode>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            self.northwest,
            self.northeast,
            self.southwest,
            self.southeast,
        ]
        .into_iter()
    }
}
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum Direction {
    North,
    East,
    South,
    West,
}
impl Direction {
    pub(crate) fn traversal(&self) -> Direction {
        match self {
            Direction::West => Direction::South,
            Direction::North => Direction::East,
            Direction::East => Direction::North,
            Direction::South => Direction::West,
        }
    }
    pub(crate) fn opposite(&self) -> Direction {
        match self {
            Direction::West => Direction::East,
            Direction::North => Direction::South,
            Direction::East => Direction::West,
            Direction::South => Direction::North,
        }
    }
}
#[derive(Component, Debug)]
pub struct QuadtreeTileMark;
#[derive(Component, Debug)]
pub enum QuadtreeTileLoadState {
    Start = 0,
    Loading = 1,
    Done = 2,
    Failed = 3,
}
#[derive(Component, Debug)]
pub struct QuadtreeTileOtherState {
    pub _distance: f64,
    pub _loadPriority: f64,
    pub renderable: bool,
    pub upsampledFromParent: bool,
}
impl Default for QuadtreeTileOtherState {
    fn default() -> Self {
        Self {
            _distance: 0.0,
            _loadPriority: 0.0,
            renderable: false,
            upsampledFromParent: false,
        }
    }
}
#[derive(Bundle, Debug)]
pub struct QuadtreeTile {
    pub mark: QuadtreeTileMark,
    pub key: TileKey,
    pub rectangle: Rectangle,
    pub parent: TileNode,
    pub location: Quadrant,
    pub children: NodeChildren,
    pub state: QuadtreeTileLoadState,
    pub other_state: QuadtreeTileOtherState,
}
impl QuadtreeTile {
    pub fn new(key: TileKey, rectangle: Rectangle, location: Quadrant, parent: TileNode) -> Self {
        let me = Self {
            mark: QuadtreeTileMark,
            key: key,
            rectangle: rectangle,
            parent,
            location,
            children: Default::default(),
            state: QuadtreeTileLoadState::Start,
            other_state: QuadtreeTileOtherState::default(),
        };
        return me;
    }
}

#[derive(Component)]
pub struct TileToRender;

#[derive(Component)]
pub struct TileLoadHigh;

#[derive(Component)]
pub struct TileLoadMedium;

#[derive(Component)]
pub struct TileLoadLow;
