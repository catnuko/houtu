


use bevy::prelude::Entity;
use houtu_scene::Rectangle;

use super::globe_surface_tile::GlobeSurfaceTile;

use super::tile_key::TileKey;
use super::tile_selection_result::TileSelectionResult;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum QuadtreeTileLoadState {
    START = 0,
    LOADING = 1,
    DONE = 2,
    FAILED = 3,
}
pub struct QuadtreeTile {
    pub location: Quadrant,
    pub parent: Option<TileKey>,
    pub northwest: Option<TileKey>,
    pub northeast: Option<TileKey>,
    pub southwest: Option<TileKey>,
    pub southeast: Option<TileKey>,
    pub key: TileKey,
    pub rectangle: Rectangle,
    pub distance: f64,
    pub load_priority: f64,
    pub renderable: bool,
    pub upsampled_from_parent: bool,
    pub last_selection_result: TileSelectionResult,
    pub last_selection_result_frame: Option<u32>,
    pub replacement_previous: Option<TileKey>,
    pub replacement_next: Option<TileKey>,
    pub data: GlobeSurfaceTile,
    pub state: QuadtreeTileLoadState,
    pub entity: Option<Entity>,
}
impl QuadtreeTile {
    pub fn new(
        key: TileKey,
        location: Quadrant,
        parent: Option<TileKey>,
        rectangle: Rectangle,
    ) -> Self {
        bevy::log::info!("new quadtree tile,{:?}", key);
        Self {
            location: location,
            parent: parent,
            key: key,
            rectangle: rectangle,
            distance: 0.,
            load_priority: 0.,
            renderable: false,
            upsampled_from_parent: false,
            last_selection_result: TileSelectionResult::NONE,
            last_selection_result_frame: None,
            northeast: None,
            northwest: None,
            southeast: None,
            southwest: None,
            replacement_next: None,
            replacement_previous: None,
            data: GlobeSurfaceTile::new(),
            state: QuadtreeTileLoadState::START,
            entity: None,
        }
    }
    pub fn eligible_for_unloading(&self) -> bool {
        return self.data.eligible_for_unloading();
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Quadrant {
    Northwest,
    Northeast,
    Southwest,
    Southeast,
    Root(usize),
}
#[derive(Clone, Copy, Hash, Debug, PartialEq, Eq)]
pub enum TileNode {
    /// None variant.
    None,
    /// Identifier of an internal node.
    Internal(usize),
}
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct NodeChildren {
    pub(super) northwest: Option<TileKey>,
    pub(super) northeast: Option<TileKey>,
    pub(super) southwest: Option<TileKey>,
    pub(super) southeast: Option<TileKey>,
}

impl Default for NodeChildren {
    fn default() -> Self {
        Self {
            northwest: None,
            northeast: None,
            southwest: None,
            southeast: None,
        }
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
