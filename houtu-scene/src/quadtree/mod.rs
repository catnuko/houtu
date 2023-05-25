mod quadtree_tile;
use std::{cell::RefCell, rc::Rc};

pub use quadtree_tile::*;

use crate::TilingScheme;

pub struct QuadTree {}
impl QuadTree {
    // pub fn new_node<T: TilingScheme>(
    //     x: u32,
    //     y: u32,
    //     level: u32,
    //     tilingScheme: T,
    //     parent: Option<Rc<RefCell<QuadtreeTile<T>>>>,
    // ) -> QuadtreeTile<T> {
    //     return;
    // }
}
