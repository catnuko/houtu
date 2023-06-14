use bevy::prelude::*;

use super::tile_quad_tree::{system, TileQuadTree};

pub struct Plugin;

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileQuadTree::new());
        app.add_system(system);
    }
}
