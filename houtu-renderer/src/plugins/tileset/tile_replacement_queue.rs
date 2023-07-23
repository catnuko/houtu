use std::collections::LinkedList;

use bevy::prelude::*;

use super::{globe_surface_tile::GlobeSurfaceTile, tile_quad_tree::GlobeSurfaceTileQuery};
#[derive(Component, Clone, Reflect)]
pub struct TileReplacementState {
    replacementPrevious: Option<Entity>,
    replacementNext: Option<Entity>,
    entity: Entity,
}
impl TileReplacementState {
    pub fn new(entity: Entity) -> Self {
        Self {
            replacementPrevious: None,
            replacementNext: None,
            entity: entity,
        }
    }
}
impl PartialEq for TileReplacementState {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
    }
}
fn set_state_of_entity(
    query: &mut Query<(GlobeSurfaceTileQuery)>,
    entity: Entity,
    state_entity: Option<Entity>,
    is_next: bool,
) {
    let (mut inner_state) = query
        .get_component_mut::<TileReplacementState>(entity)
        .unwrap();
    if is_next {
        inner_state.replacementNext = state_entity;
    } else {
        inner_state.replacementPrevious = state_entity;
    }
}
#[derive(Debug)]
pub struct TileReplacementQueue {
    list: LinkedList<Entity>,
    last_before_start_of_frame: Option<Entity>,
    count: usize,
}
impl TileReplacementQueue {
    pub fn new() -> Self {
        Self {
            list: LinkedList::new(),
            last_before_start_of_frame: None,
            count: 0,
        }
    }
    pub fn get_head(&self) -> Option<&Entity> {
        return self.list.front();
    }
    pub fn get_head_mut(&mut self) -> Option<&mut Entity> {
        return self.list.front_mut();
    }
    pub fn get_tail(&self) -> Option<&Entity> {
        return self.list.back();
    }
    pub fn get_tail_mut(&mut self) -> Option<&mut Entity> {
        return self.list.back_mut();
    }
    pub fn get_count(&self) -> usize {
        return self.list.len();
    }
    pub fn markStartOfRenderFrame(&mut self) {
        let head = self.get_head();
        if let Some(v) = head {
            self.last_before_start_of_frame = Some(v.clone());
        } else {
            self.last_before_start_of_frame = None;
        }
    }
    pub fn trimTiles(&mut self, maximum_tiles: u32, query: &mut Query<(GlobeSurfaceTileQuery)>) {
        // let mut tile_to_trim = self.get_tail_mut();
        let mut keep_trimming = true;
        let mut count = self.count;
        while (keep_trimming
            // && self.last_before_start_of_frame.is_some()
            && count > maximum_tiles as usize)
            && self.get_tail_mut().is_some()
        {
            // Stop trimming after we process the last tile not used in the
            // current frame.
            keep_trimming = self.get_tail() != self.last_before_start_of_frame.as_ref();

            let tile_to_trim = self.get_tail_mut();
            let tile_to_trim_entity = tile_to_trim.unwrap();
            let mut tile_to_trim_state = query
                .get_component_mut::<TileReplacementState>(tile_to_trim_entity.clone())
                .unwrap();

            let previous = tile_to_trim_state.replacementPrevious;

            let globe_surface_tile = query
                .get_component_mut::<GlobeSurfaceTile>(tile_to_trim_entity.clone())
                .unwrap();

            if globe_surface_tile.eligible_for_unloading() {
                // tile_to_trim_state.freeResources();

                let entity = tile_to_trim_entity.clone();
                self.remove(query, entity);
            }
            if let Some(entity) = previous {
                if let Some(v) = self.get_tail_mut() {
                    *v = entity;
                }
            }
            count = self.count;
        }
    }
    fn remove(&mut self, query: &mut Query<(GlobeSurfaceTileQuery)>, entity: Entity) {
        let mut item = query
            .get_component_mut::<TileReplacementState>(entity)
            .unwrap();
        {
            if self.last_before_start_of_frame.is_some()
                && self.last_before_start_of_frame.unwrap() == item.entity
            {
                self.last_before_start_of_frame = item.replacementNext.clone();
            }
        }
        let head_mut = self.get_head_mut();
        if head_mut == Some(&mut item.entity) {
            if let Some(t) = item.replacementNext {
                if let Some(v) = head_mut {
                    *v = t.clone();
                }
            } else {
                if let None = head_mut {
                    self.list.clear();
                }
            }
        } else {
            let entity = item.replacementPrevious.unwrap().clone();
            let state_entity = item.replacementNext.clone();
            set_state_of_entity(query, entity, state_entity, true);
        }

        let mut item = query
            .get_component_mut::<TileReplacementState>(entity)
            .unwrap();
        let tail_mut = self.get_tail_mut();
        if tail_mut.is_some() && tail_mut == Some(&mut item.entity) {
            if let Some(t) = item.replacementPrevious {
                if let Some(v) = tail_mut {
                    *v = t.clone();
                }
            } else {
                if let None = tail_mut {
                    self.list.pop_back();
                }
            }
        } else {
            let entity = item.replacementNext.unwrap().clone();
            let state_entity = item.replacementPrevious.clone();
            set_state_of_entity(query, entity, state_entity, false);
        }
        self.count -= 1;
    }
    pub fn mark_tile_rendered(
        &mut self,
        query: &mut Query<(GlobeSurfaceTileQuery)>,
        entity: Entity,
    ) {
        let mut item = query
            .get_component_mut::<TileReplacementState>(entity)
            .unwrap();
        {
            let head_mut = self.get_head_mut();
            if head_mut.is_some() && *head_mut.unwrap() == item.entity {
                if self.last_before_start_of_frame.is_some()
                    && self.last_before_start_of_frame.unwrap() == item.entity
                {
                    if let Some(t) = item.replacementPrevious {
                        self.last_before_start_of_frame = Some(t.clone());
                    } else {
                        self.last_before_start_of_frame = None;
                    }
                }
                return;
            }
        }
        self.count += 1;
        {
            let head_mut = self.get_head_mut();
            if head_mut.is_none() {
                item.replacementNext = None;
                item.replacementPrevious = None;
                if let Some(v) = head_mut {
                    *v = item.entity;
                }
                let tail_mut = self.get_tail_mut();
                if let Some(v) = tail_mut {
                    *v = item.entity;
                }
                return;
            }
        }
        if item.replacementNext.is_some() || item.replacementPrevious.is_some() {
            self.remove(query, entity);
        }
        let mut item = query
            .get_component_mut::<TileReplacementState>(entity)
            .unwrap();
        let head_mut = self.get_head_mut();
        item.replacementPrevious = None;
        if let Some(v) = head_mut {
            item.replacementNext = Some(v.clone());
            let entity = v.clone();
            let state_entity = Some(item.entity.clone());
            set_state_of_entity(query, entity, state_entity, false);
        } else {
            item.replacementNext = None;
        }
    }
}
