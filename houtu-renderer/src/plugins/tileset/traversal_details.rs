use bevy::prelude::{ResMut, Resource};

use super::{quadtree_tile::Quadrant, TileKey};

#[derive(Resource)]
pub struct AllTraversalQuadDetails(pub [TraversalQuadDetails; 31]);
impl AllTraversalQuadDetails {
    pub fn new() -> Self {
        AllTraversalQuadDetails([TraversalQuadDetails::new(); 31])
    }
    pub fn get(&self, level: u32) -> &TraversalQuadDetails {
        self.0.get(level as usize).unwrap()
    }
    pub fn get_mut(&mut self, level: u32) -> &mut TraversalQuadDetails {
        self.0.get_mut(level as usize).unwrap()
    }
}
#[derive(Resource)]
pub struct RootTraversalDetails(pub Vec<TraversalDetails>);
impl RootTraversalDetails {
    pub fn new() -> Self {
        RootTraversalDetails(Vec::new())
    }
    pub fn get(&self, level: u32) -> &TraversalDetails {
        self.0.get(level as usize).unwrap()
    }
    pub fn get_mut(&mut self, level: u32) -> &mut TraversalDetails {
        self.0.get_mut(level as usize).unwrap()
    }
}
#[derive(Clone, Copy)]
pub struct TraversalQuadDetails {
    pub southwest: TraversalDetails,
    pub southeast: TraversalDetails,
    pub northwest: TraversalDetails,
    pub northeast: TraversalDetails,
}
impl TraversalQuadDetails {
    pub fn new() -> Self {
        Self {
            southwest: TraversalDetails::default(),
            southeast: TraversalDetails::default(),
            northwest: TraversalDetails::default(),
            northeast: TraversalDetails::default(),
        }
    }
    pub fn combine(&self) -> TraversalDetails {
        let southwest = self.southwest;
        let southeast = self.southeast;
        let northwest = self.northwest;
        let northeast = self.northeast;
        let mut result = TraversalDetails::default();
        result.all_are_renderable = southwest.all_are_renderable
            && southeast.all_are_renderable
            && northwest.all_are_renderable
            && northeast.all_are_renderable;
        result.any_were_rendered_last_frame = southwest.any_were_rendered_last_frame
            || southeast.any_were_rendered_last_frame
            || northwest.any_were_rendered_last_frame
            || northeast.any_were_rendered_last_frame;
        result.not_yet_renderable_count = southwest.not_yet_renderable_count
            + southeast.not_yet_renderable_count
            + northwest.not_yet_renderable_count
            + northeast.not_yet_renderable_count;
        return result;
    }
}
#[derive(Clone, Copy)]
pub struct TraversalDetails {
    pub all_are_renderable: bool,
    pub any_were_rendered_last_frame: bool, //上一帧选择结果是否是已渲染，anyWereRenderedLastFrame===TileSelectionResult.RENDERED
    pub not_yet_renderable_count: u32,
}
impl Default for TraversalDetails {
    fn default() -> Self {
        Self {
            all_are_renderable: true,
            any_were_rendered_last_frame: false,
            not_yet_renderable_count: 0,
        }
    }
}
pub fn get_traversal_details<'a>(
    all_traversal_quad_details: &'a mut ResMut<AllTraversalQuadDetails>,
    root_traversal_details: &'a mut ResMut<RootTraversalDetails>,
    location: &Quadrant,
    key: &TileKey,
) -> &'a mut TraversalDetails {
    return match location {
        Quadrant::Southwest => &mut all_traversal_quad_details.get_mut(key.level).southwest,
        Quadrant::Southeast => &mut all_traversal_quad_details.get_mut(key.level).southeast,
        Quadrant::Northwest => &mut all_traversal_quad_details.get_mut(key.level).northwest,
        Quadrant::Northeast => &mut all_traversal_quad_details.get_mut(key.level).northeast,
        Quadrant::Root(i) => root_traversal_details.0.get_mut(*i).unwrap(),
    };
}
