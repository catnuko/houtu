use std::collections::LinkedList;

use bevy::prelude::*;

// pub struct Plugin;
// impl bevy::prelude::Plugin for Plugin {
//     fn build(&self, app: &mut App) {
//         app.insert_resource(TileReplacementQueue::new());
//         app.add_system(system);
//     }
// }
#[derive(Component, Clone)]
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
    query: &mut Query<(&TileReplacementState)>,
    entity: Entity,
    state_entity: Option<Entity>,
    is_next: bool,
) {
    let (mut inner_state) = query.get_mut(entity).unwrap();
    if is_next {
        inner_state.replacementNext = state_entity;
    } else {
        inner_state.replacementPrevious = state_entity;
    }
}
#[derive(Debug)]
pub struct TileReplacementQueue {
    list: LinkedList<Entity>,
    _lastBeforeStartOfFrame: Option<Entity>,
    count: usize,
}
impl TileReplacementQueue {
    pub fn new() -> Self {
        Self {
            list: LinkedList::new(),
            _lastBeforeStartOfFrame: None,
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
            self._lastBeforeStartOfFrame = Some(v.clone());
        } else {
            self._lastBeforeStartOfFrame = None;
        }
    }
    fn remove(
        &mut self,
        query: &mut Query<(&TileReplacementState)>,
        item: &mut TileReplacementState,
    ) {
        {
            if self._lastBeforeStartOfFrame.is_some()
                && self._lastBeforeStartOfFrame.unwrap() == item.entity
            {
                self._lastBeforeStartOfFrame = item.replacementNext.clone();
            }
        }
        let head_mut = self.get_head_mut();
        if *head_mut.unwrap() == item.entity {
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
            set_state_of_entity(
                query,
                item.replacementPrevious.unwrap(),
                item.replacementNext,
                true,
            );
        }
        let tail_mut = self.get_tail_mut();
        if tail_mut.is_some() && *tail_mut.unwrap() == item.entity {
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
            set_state_of_entity(
                query,
                item.replacementNext.unwrap(),
                item.replacementPrevious,
                false,
            );
        }
    }
    pub fn markTileRendered(
        &mut self,
        query: &mut Query<(&TileReplacementState)>,
        item: &mut TileReplacementState,
    ) {
        {
            let head_mut = self.get_head_mut();
            if head_mut.is_some() && *head_mut.unwrap() == item.entity {
                if self._lastBeforeStartOfFrame.is_some()
                    && self._lastBeforeStartOfFrame.unwrap() == item.entity
                {
                    if let Some(t) = item.replacementPrevious {
                        self._lastBeforeStartOfFrame = Some(t.clone());
                    } else {
                        self._lastBeforeStartOfFrame = None;
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
            self.remove(query, item);
        }
        let head_mut = self.get_head_mut();
        item.replacementPrevious = None;
        if let Some(v) = head_mut {
            item.replacementNext = Some(v.clone());
            set_state_of_entity(query, v.clone(), Some(item.entity), false);
        } else {
            item.replacementNext = None;
        }
    }
}
