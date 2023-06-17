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
// #[derive(Component)]
// pub struct EmitMarkTileRendered;

// #[derive(Resource)]
// pub struct TileReplacementQueue {
//     head: Option<TileReplacementState>,
//     tail: Option<TileReplacementState>,
//     count: u32,
//     _lastBeforeStartOfFrame: Option<TileReplacementState>,
// }
// impl TileReplacementQueue {
//     fn new() -> Self {
//         Self {
//             head: None,
//             tail: None,
//             count: 0,
//             _lastBeforeStartOfFrame: None,
//         }
//     }
//     pub fn markStartOfRenderFrame(&mut self) {
//         self._lastBeforeStartOfFrame = self.head;
//     }
// }
// fn system(
//     mut queue: ResMut<TileReplacementQueue>,
//     mut commands: Commands,
//     mut mark_tile_rendered_query: Query<
//         (Entity, &TileReplacementState),
//         With<EmitMarkTileRendered>,
//     >,
// ) {
//     for (entity, mut state) in &mut mark_tile_rendered_query {
//         let mut entity_mut = commands.get_entity(entity).expect("entity不存在");
//         entity_mut.remove::<EmitMarkTileRendered>();
//         if queue.head.is_some() && queue.head.unwrap() == *state {
//             queue._lastBeforeStartOfFrame = None;
//             if queue._lastBeforeStartOfFrame.is_some()
//                 && *state == queue._lastBeforeStartOfFrame.unwrap()
//             {
//                 queue._lastBeforeStartOfFrame = get_state_of_entity(
//                     &mut commands,
//                     state.replacementNext.expect("entity不存在"),
//                 );
//             }
//         }
//         queue.count += 1;
//         if queue.head.is_none() {
//             state.replacementPrevious = None;
//             state.replacementNext = None;
//             queue.head = Some(state.clone());
//             queue.tail = Some(state.clone());
//             return;
//         }
//         if state.replacementPrevious.is_some() || state.replacementNext.is_some() {
//             //remove
//             let next_entity =
//                 get_state_of_entity(&mut commands, state.replacementNext.expect("entity不存在"));
//             let pre_entity = get_state_of_entity(
//                 &mut commands,
//                 state.replacementPrevious.expect("entity不存在"),
//             );

//             if queue._lastBeforeStartOfFrame.is_some()
//                 && queue._lastBeforeStartOfFrame.unwrap() == *state
//             {
//                 queue._lastBeforeStartOfFrame = next_entity.clone();
//             }
//             if queue.head.is_some() && queue.head.unwrap() == *state {
//                 queue.head = next_entity.clone();
//             } else {
//                 set_state_of_entity(
//                     &mut commands,
//                     state.replacementPrevious.expect("entity不存在"),
//                     state.replacementNext,
//                     true,
//                 );
//             }
//             if queue.tail.is_some() && queue.tail.unwrap() == *state {
//                 queue.tail = pre_entity.clone();
//             } else {
//                 set_state_of_entity(
//                     &mut commands,
//                     state.replacementNext.expect("entity不存在"),
//                     state.replacementPrevious,
//                     false,
//                 );
//             }
//             state.replacementPrevious = None;
//             state.replacementNext = None;
//             queue.count -= 1;
//         }
//         state.replacementPrevious = None;
//         state.replacementNext = Some(queue.head.unwrap().entity);
//         set_state_of_entity(
//             &mut commands,
//             queue.head.unwrap().entity,
//             Some(entity),
//             false,
//         );
//         queue.head = Some(state.clone());
//     }
// }
fn get_state_of_entity(commands: &mut Commands, entity: Entity) -> Option<TileReplacementState> {
    let mut res: Option<TileReplacementState> = None;
    commands.add(|world: &mut World| {
        if let Some(state) = world.get::<TileReplacementState>(entity) {
            res = Some(state.clone());
        }
    });
    return res;
}
fn set_state_of_entity(
    commands: &mut Commands,
    entity: Entity,
    state_entity: Option<Entity>,
    is_next: bool,
) {
    let mut res: Option<TileReplacementState> = None;
    commands.add(|world: &mut World| {
        let mut inner_state = world
            .get_mut::<TileReplacementState>(entity)
            .expect("state不存在");
        if is_next {
            inner_state.replacementNext = state_entity;
        } else {
            inner_state.replacementPrevious = state_entity;
        }
    });
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
    pub fn get_head_mut(&self) -> Option<&mut Entity> {
        return self.list.front_mut();
    }
    pub fn get_tail(&self) -> Option<&Entity> {
        return self.list.back();
    }
    pub fn get_tail_mut(&self) -> Option<&mut Entity> {
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
    fn remove(&self, commands: &mut Commands, item: &mut TileReplacementState) {
        let head_mut = self.get_head_mut();
        let tail_mut = self.get_tail_mut();
        if self._lastBeforeStartOfFrame.is_some()
            && self._lastBeforeStartOfFrame.unwrap() == item.entity
        {
            self._lastBeforeStartOfFrame = item.replacementNext.clone();
        }
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
                commands,
                item.replacementPrevious.unwrap(),
                item.replacementNext,
                true,
            );
        }
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
                commands,
                item.replacementNext.unwrap(),
                item.replacementPrevious,
                false,
            );
        }
    }
    pub fn markTileRendered(&mut self, commands: &mut Commands, item: &mut TileReplacementState) {
        let head_mut = self.get_head_mut();
        let tail_mut = self.get_tail_mut();
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
        self.count += 1;
        if head_mut.is_none() {
            item.replacementNext = None;
            item.replacementPrevious = None;
            if let Some(v) = head_mut {
                *v = item.entity;
            }
            if let Some(v) = tail_mut {
                *v = item.entity;
            }
            return;
        }
        if item.replacementNext.is_some() || item.replacementPrevious.is_some() {
            self.remove(commands, item);
        }
        item.replacementPrevious = None;
        if let Some(v) = head_mut {
            item.replacementNext = Some(v.clone());
        } else {
            item.replacementNext = None;
        }
        set_state_of_entity(commands, *head_mut.unwrap(), Some(item.entity), false);
    }
}
