use super::{quadtree_node::QuadtreeNode, quadtree_value::QuadtreeValue};
use houtu_scene::Rectangle;
// use super::command::{QuadTreeNode, QuandTreeNodeRelationType};
#[derive(Clone)]
pub struct QuadtreeTileValue {
    pub x: u32,
    pub y: u32,
    pub level: u32,
    pub rectangle: Rectangle,
    pub renderable: bool,
    pub _distance: f64,
    pub _loadPriority: f64, //加载优先级
    // pub _loadQueueType: TileLoadQueueType,
    pub _lastSelectionResultFrame: u32,
    pub _lastSelectionResult: TileSelectionResult,
    pub needsLoading: bool,
}
impl QuadtreeTileValue {
    pub fn new(x: u32, y: u32, level: u32, rectangle: Rectangle) -> Self {
        Self {
            x: x,
            y: y,
            level: level,
            rectangle: rectangle,
            _lastSelectionResultFrame: 0,
            ..Default::default()
        }
    }
}
impl QuadtreeValue for QuadtreeTileValue {}
