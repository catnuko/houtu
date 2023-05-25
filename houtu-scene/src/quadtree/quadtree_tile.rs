// use std::{cell::RefCell, rc::Rc};

// use crate::{Rectangle, TilingScheme};
// pub enum QuadtreeTileLoadState {
//     START = 0,
//     LOADING = 1,
//     DONE = 2,
//     FAILED = 3,
// }
// pub enum TileSelectionResult {
//     NONE = 0,
//     CULLED = 1,
//     RENDERED = 2,
//     REFINED = 3,
//     RENDERED_AND_KICKED = 2 | 4,
//     REFINED_AND_KICKED = 3 | 4,
//     CULLED_BUT_NEEDED = 1 | 8,
// }
// // impl TileSelectionResult{
// //     pub fn wasKicked(value:)
// // }
// pub struct QuadtreeTile<T: TilingScheme> {
//     tilingScheme: T,
//     x: u32,
//     y: u32,
//     level: u32,
//     parent: Option<Rc<RefCell<QuadtreeTile<T>>>>,
//     rectangle: Rectangle,
//     southwestChild: Option<Rc<RefCell<QuadtreeTile<T>>>>,
//     southeastChild: Option<Rc<RefCell<QuadtreeTile<T>>>>,
//     northwestChild: Option<Rc<RefCell<QuadtreeTile<T>>>>,
//     northeastChild: Option<Rc<RefCell<QuadtreeTile<T>>>>,
//     // pub replacementPrevious: Option<f64>,
//     // pub replacementNext: Option<f64>,
//     distance: f64,
//     loadPriority: f64,
//     frameUpdated: Option<bool>,
//     lastSelectionResult: TileSelectionResult,
//     pub state: QuadtreeTileLoadState,
//     renderable: bool,
//     upsampledFromParent: bool,
// }
// impl<T> QuadtreeTile<T>
// where
//     T: TilingScheme,
// {
//     pub fn new(
//         x: u32,
//         y: u32,
//         level: u32,
//         tilingScheme: T,
//         parent: Option<Rc<RefCell<QuadtreeTile<T>>>>,
//     ) -> Self {
//         if x < 0 || y < 0 {
//             panic!("x and y must be greater than or equal to zero")
//         }
//         return Self {
//             x: x,
//             y: y,
//             level: level,
//             parent: parent,
//             tilingScheme: tilingScheme,
//             rectangle: tilingScheme.tile_x_y_to_rectange(x, y, level),
//             southwestChild: None,
//             southeastChild: None,
//             northwestChild: None,
//             northeastChild: None,
//             distance: 0.,
//             loadPriority: 0.,
//             frameUpdated: None,
//             lastSelectionResult: TileSelectionResult::NONE,
//             state: QuadtreeTileLoadState::START,
//             renderable: false,
//             upsampledFromParent: false,
//         };
//     }
//     pub fn createLevelZeroTiles(tilingScheme: T) -> Vec<QuadtreeTile<T>> {
//         let numberOfLevelZeroTilesX = tilingScheme.get_number_of_tiles_at_level(0);
//         let numberOfLevelZeroTilesY = tilingScheme.get_number_of_tiles_at_level(0);
//         let mut result = vec![];
//         for y in 0..numberOfLevelZeroTilesY {
//             for x in 0..numberOfLevelZeroTilesX {
//                 result.push(QuadtreeTile::new(x, y, 0, tilingScheme, None));
//             }
//         }
//         return result;
//     }
//     pub fn get_southwestChild(&mut self) -> Rc<RefCell<QuadtreeTile<T>>> {
//         return match self.southeastChild {
//             Some(v) => v.clone(),
//             None => {
//                 let v = QuadtreeTile::new(
//                     self.x * 2,
//                     self.y * 2 + 1,
//                     self.level +1,
//                     self.tilingScheme,
//                     parent: ,
//                 );
//                 let nv = Rc::new(v);
//                 self.southwestChild = Some(nv);
//                 return nv.clone();
//             }
//         };
//     }
//     pub fn get_southeastChild(&mut self) {}
//     pub fn get_northwestChild(&mut self) {}
//     pub fn get_northeastChild(&mut self) {}
// }
