use bevy::prelude::*;
use houtu_scene::GeographicTilingScheme;
use houtu_scene::{Rectangle, TilingScheme};
use std::fmt;
use std::ops::Index;
use std::ops::IndexMut;
use std::sync::Arc;

use super::globe_surface_tile::GlobeSurfaceTile;
use super::tile_replacement_queue::TileReplacementState;
use super::tile_selection_result::TileSelectionResult;
use super::{tile_datasource::TilingSchemeWrap, TileKey};
#[derive(Component, Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum TileNode {
    /// None variant.
    None,
    /// Identifier of an internal node.
    Internal(Entity),
}
#[derive(Component, Debug, PartialEq)]
pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
    Root(usize),
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
#[derive(Component, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuadtreeTileLoadState {
    START = 0,
    LOADING = 1,
    DONE = 2,
    FAILED = 3,
}
#[derive(Component, Debug)]
pub struct QuadtreeTileOtherState {
    pub _distance: f64,
    pub _loadPriority: f64,
    pub renderable: bool,
    pub upsampledFromParent: bool,
    pub _lastSelectionResult: TileSelectionResult,
    pub _lastSelectionResultFrame: Option<u32>,
}
impl Default for QuadtreeTileOtherState {
    fn default() -> Self {
        Self {
            _distance: 0.0,
            _loadPriority: 0.0,
            renderable: false,
            upsampledFromParent: false,
            _lastSelectionResultFrame: None,
            _lastSelectionResult: TileSelectionResult::NONE,
        }
    }
}
#[derive(Component)]
pub struct QuadtreeTileData(pub Option<GlobeSurfaceTile>);
#[derive(Debug, Component)]
pub struct QuadtreeTileParent(pub TileNode);
#[derive(Bundle)]
pub struct QuadtreeTile {
    pub mark: QuadtreeTileMark,
    pub key: TileKey,
    pub rectangle: Rectangle,
    pub parent: QuadtreeTileParent,
    // pub node: TileNode,
    pub location: Quadrant,
    pub children: NodeChildren,
    pub state: QuadtreeTileLoadState,
    pub other_state: QuadtreeTileOtherState,
    pub data: GlobeSurfaceTile,
}
impl QuadtreeTile {
    pub fn new(
        key: TileKey,
        rectangle: Rectangle,
        location: Quadrant,
        parent: QuadtreeTileParent,
        // node_id: TileNode,
    ) -> Self {
        let me = Self {
            mark: QuadtreeTileMark,
            key: key,
            rectangle: rectangle,
            location,
            children: Default::default(),
            state: QuadtreeTileLoadState::START,
            other_state: QuadtreeTileOtherState::default(),
            parent,
            data: GlobeSurfaceTile::new(), // node: node_id,
        };
        return me;
    }
}

#[derive(Component)]
pub struct TileToRender;
#[derive(Component)]
pub struct TileToUpdateHeight;
#[derive(Component)]
pub struct TileLoadHigh;

#[derive(Component)]
pub struct TileLoadMedium;

#[derive(Component)]
pub struct TileLoadLow;
#[derive(Component)]
pub struct TileToLoad;
