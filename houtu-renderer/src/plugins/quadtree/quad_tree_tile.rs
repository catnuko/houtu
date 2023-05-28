use std::ops::Deref;

use super::{
    quad_tree_tile_value::QuadtreeTileValue, quadtree_node::QuadtreeNode, QuadtreeNodeTrait,
    QuadtreeValue,
};
use bevy::{
    ecs::{
        system::EntityCommands,
        world::{self, EntityMut},
    },
    prelude::*,
};
use houtu_scene::{GeographicTilingScheme, Rectangle, TilingScheme};

#[derive(Clone, Default)]
pub struct QuadtreeTile {
    pub node: QuadtreeNode<QuadtreeTileValue>,
}
impl QuadtreeTile {
    pub fn createLevelZeroTiles(tiling_scheme: &dyn TilingScheme) -> Vec<Self> {
        let numberOfLevelZeroTilesX = tiling_scheme.get_number_of_x_tiles_at_level(0);
        let numberOfLevelZeroTilesY = tiling_scheme.get_number_of_y_tiles_at_level(0);
        let mut result = Vec::new();
        for y in 0..numberOfLevelZeroTilesY {
            for x in 0..numberOfLevelZeroTilesX {
                let rectangle = tiling_scheme.tile_x_y_to_rectange(x, y, 0);
                result.push(Self::new(x, y, 0, rectangle))
            }
        }
        return result;
    }
    pub fn new(x: u32, y: u32, level: u32, rectangle: Rectangle) -> Self {
        let mut node = QuadtreeNode::<QuadtreeTileValue>::empty();
        let data = QuadtreeTileValue::new(x, y, level, rectangle);
        node.value = data;
        Self { node }
    }

    pub fn get_southwest_child(&self, tiling_scheme: &dyn TilingScheme) -> Self {
        let x = self.data.x * 2;
        let y = self.data.y * 2 + 1;
        let level = self.data.level + 1;
        let rectangle = tiling_scheme.tile_x_y_to_rectange(x, y, level);
        let me = Self::new(x, y, level, rectangle);
        me.parent = Some(self);
        self.node.southwestChild = Some(me);
        return me;
    }
    pub fn get_southeast_child(&self, tiling_scheme: &dyn TilingScheme) -> Self {
        let x = self.data.x * 2 + 1;
        let y = self.data.y * 2 + 1;
        let level = self.data.level + 1;
        let rectangle = tiling_scheme.tile_x_y_to_rectange(x, y, level);
        let me = Self::new(x, y, level, rectangle);
        me.parent = Some(self);
        self.node.southeastChild = Some(me);
        return me;
    }
    pub fn get_northwest_child(&self, tiling_scheme: &dyn TilingScheme) -> Self {
        let x = self.data.x * 2;
        let y = self.data.y * 2;
        let level = self.data.level + 1;
        let rectangle = tiling_scheme.tile_x_y_to_rectange(x, y, level);
        let me = Self::new(x, y, level, rectangle);
        me.parent = Some(self);
        self.node.northwestChild = Some(me);
        return me;
    }
    pub fn get_northeast_child(&self, tiling_scheme: &dyn TilingScheme) -> Self {
        let x = self.data.x * 2 + 1;
        let y = self.data.y * 2;
        let level = self.data.level + 1;
        let rectangle = tiling_scheme.tile_x_y_to_rectange(x, y, level);
        let me = Self::new(x, y, level, rectangle);
        me.parent = Some(self);
        self.node.northeastChild = Some(me);
        return me;
    }
    pub fn setup(&self, commands: &mut Commands) {}
}
impl QuadtreeNodeTrait for QuadtreeTile {}
impl Deref for QuadtreeTile {
    type Target = QuadtreeNode<QuadtreeTileValue>;

    fn deref(&self) -> &Self::Target {
        &self.node.value
    }
}
