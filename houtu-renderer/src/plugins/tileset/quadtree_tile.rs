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
use super::TileKey;
#[derive(Component, Clone, Copy, Hash, Debug, PartialEq, Eq, Reflect)]
pub enum TileNode {
    /// None variant.
    None,
    /// Identifier of an internal node.
    Internal(Entity),
}
#[derive(Component, Debug, PartialEq, Reflect)]
pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
    Root(usize),
}

#[derive(Clone, Copy, PartialEq, Debug, Component, Reflect)]
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
#[derive(Component, Debug, Reflect)]
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
    pub upsampled_from_parent: bool,
    pub last_selection_result: TileSelectionResult,
    pub last_selection_result_frame: Option<u32>,
}
impl Default for QuadtreeTileOtherState {
    fn default() -> Self {
        Self {
            _distance: 0.0,
            _loadPriority: 0.0,
            renderable: false,
            upsampled_from_parent: false,
            last_selection_result_frame: None,
            last_selection_result: TileSelectionResult::NONE,
        }
    }
}
#[derive(Component)]
pub struct QuadtreeTileData(pub Option<GlobeSurfaceTile>);
#[derive(Debug, Component, Reflect)]
pub struct QuadtreeTileParent(pub TileNode);
#[derive(Bundle)]
pub struct QuadtreeTile {
    pub mark: QuadtreeTileMark,
    pub key: TileKey,
    pub rectangle: Rectangle,
    pub parent: QuadtreeTileParent,
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

#[derive(Component, Reflect)]
pub struct TileToRender;
#[derive(Component, Reflect)]
pub struct TileToUpdateHeight;
#[derive(Component, Reflect)]
pub struct TileLoadHigh;

#[derive(Component, Reflect)]
pub struct TileLoadMedium;

#[derive(Component, Reflect)]
pub struct TileLoadLow;
#[derive(Component, Reflect)]
pub struct TileToLoad; //GlobeSurfaceTileProvider.loadTile
#[derive(Component, Reflect)]
pub struct TileRendered(pub Entity);
#[derive(Component, Reflect)]
pub struct TileRenderedToDestroy;
