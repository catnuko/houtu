use bevy::{
    ecs::system::{Command, EntityCommands},
    math::DVec3,
    prelude::*,
};
use houtu_scene::{Rectangle, TileBoundingRegion};

use crate::quadtree::{globe_surface_tile::TerrainState, tile_key::TileKey};

use super::{load::Load, tile_imagery::TileImageryVec};
#[derive(Component)]
pub struct ParentIndex(usize);
impl ParentIndex {
    pub const INVALID: ParentIndex = ParentIndex(usize::MAX);
}
#[derive(Debug, PartialEq)]
pub enum TileVisibility {
    NONE = -1,
    PARTIAL = 0,
    FULL = 1,
}
#[derive(Component)]
pub struct Renderable;
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Component, Default)]
pub enum QuadtreeTileLoadState {
    #[default]
    START = 0,
    LOADING = 1,
    DONE = 2,
    FAILED = 3,
}
/// QuadtreeTile和GlobeSurafceTile是同一个Entity
#[derive(Component)]
pub struct QuadtreeTile {
    pub occludee_point_in_scaled_space: Option<DVec3>,
    pub distance: f64,
    pub visibility: TileVisibility,
    pub clipped_by_boundaries: bool,
    pub renderable: bool,
    pub sse: f64,
    pub meets_sse: bool,
    pub upsampled_from_parent: bool,
}
impl Default for QuadtreeTile {
    fn default() -> Self {
        Self {
            occludee_point_in_scaled_space: None,
            distance: f64::MAX,
            visibility: TileVisibility::NONE,
            clipped_by_boundaries: false,
            renderable: false,
            sse: f64::MAX,
            meets_sse: false,
            upsampled_from_parent:false,
        }
    }
}
#[derive(Bundle)]
pub struct QuadtreeTileBundle {
    pub key: TileKey,
    pub load: Load,
    pub imagery_vec: TileImageryVec,
    pub quadtree_tile: QuadtreeTile,
    /// Enables or disables the light
    pub visibility: Visibility,
    /// Inherited visibility of an entity.
    pub inherited_visibility: InheritedVisibility,
    /// Algorithmically-computed indication of whether an entity is visible and should be extracted for rendering
    pub view_visibility: ViewVisibility,
    pub state: QuadtreeTileLoadState,
    pub terrain_state: TerrainState,
}
impl QuadtreeTileBundle {
    pub fn new(key: TileKey) -> Self {
        Self {
            key: key,
            load: Load::new(),
            quadtree_tile: QuadtreeTile::default(),
            imagery_vec: TileImageryVec::default(),
            visibility: Visibility::default(),
            inherited_visibility: InheritedVisibility::default(),
            view_visibility: ViewVisibility::default(),
            state: QuadtreeTileLoadState::default(),
            terrain_state: TerrainState::default(),
        }
    }
}
