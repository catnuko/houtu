use std::{rc::Rc, sync::Arc};

use bevy::{
    ecs::entity::{EntityMap, MapEntities, MapEntitiesError},
    prelude::*,
};
pub struct Plugin;

impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {}
}
mod quad_tree_tile;
mod quad_tree_tile_value;
mod quadtree;
mod quadtree_node;
mod quadtree_stats;
mod quadtree_value;
pub use quad_tree_tile::*;
mod quad_tree_tile_load_state;
pub use quad_tree_tile_load_state::*;
pub use quad_tree_tile_value::*;
pub use quadtree::*;
pub use quadtree_node::*;
pub use quadtree_stats::*;
pub use quadtree_value::*;

pub const THRESHOLD: usize = 256;
