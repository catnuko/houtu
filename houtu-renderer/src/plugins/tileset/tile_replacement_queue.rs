use bevy::prelude::*;

pub struct Plugin;
impl bevy::prelude::Plugin for TileReplacementState {
    fn build(&self, app: &mut App) {
        app.insert_resource(TileReplacementQueue::new());
        app.add_system(system);
    }
}
#[derive(Component, Clone)]
pub struct TileReplacementState {
    replacementPrevious: Option<Entity>,
    replacementNext: Option<Entity>,
    entity: Entity,
}
impl PartialEq for TileReplacementState {
    fn eq(&self, other: &Self) -> bool {
        self.entity == other.entity
    }
}
#[derive(Component)]
pub struct MarkTileRendered;
#[derive(Component)]
pub struct SetReplacement {
    entity: Entity,
    location: u8,
    is_state: bool,
}

#[derive(Resource)]
pub struct TileReplacementQueue {
    head: Option<TileReplacementState>,
    tail: Option<TileReplacementState>,
    count: u32,
    _lastBeforeStartOfFrame: Option<TileReplacementState>,
}
impl TileReplacementQueue {
    fn new() -> Self {
        Self {
            head: None,
            tail: None,
            count: 0,
            _lastBeforeStartOfFrame: None,
        }
    }
    fn markStartOfRenderFrame(&mut self) {
        self._lastBeforeStartOfFrame = self.head;
    }
}
fn system(
    mut queue: ResMut<TileReplacementQueue>,
    mut commands: Commands,
    mut query: Query<(Entity, &TileReplacementState), With<MarkTileRendered>>,
    mut set_query: Query<(Entity, &TileReplacementState, &SetReplacement)>,
) {
    for (entity, mut state) in &mut query {
        if let Some(mut entity_mut) = commands.get_entity(entity) {
            entity_mut.remove::<MarkTileRendered>();
            let mut next_entity = if let Some(v) = state.replacementNext {
                commands.get_entity(v)
            } else {
                None
            };
            let mut pre_entity = if let Some(v) = state.replacementPrevious {
                commands.get_entity(v)
            } else {
                None
            };

            if queue.head.is_some() && queue.head.unwrap() == *state {
                queue._lastBeforeStartOfFrame = None;
                if queue._lastBeforeStartOfFrame.is_some()
                    && *state == queue._lastBeforeStartOfFrame.unwrap()
                {
                    if let Some(mut e) = next_entity {
                        e.insert(SetReplacement {
                            entity: state.replacementNext.unwrap(),
                            location: 3,
                            is_state: false,
                        });
                    }
                }
            }
            queue.count += 1;
            if queue.head.is_none() {
                state.replacementPrevious = None;
                state.replacementNext = None;
                queue.head = Some(state.clone());
                queue.tail = Some(state.clone());
                return;
            }
            if state.replacementPrevious.is_some() || state.replacementNext.is_some() {
                //remove
                if queue._lastBeforeStartOfFrame.is_some()
                    && queue._lastBeforeStartOfFrame.unwrap() == *state
                {
                    queue._lastBeforeStartOfFrame = None;
                    if let Some(mut e) = next_entity {
                        e.insert(SetReplacement {
                            entity: state.replacementNext.unwrap(),
                            location: 3,
                            is_state: false,
                        });
                    }
                }
                if queue.head.is_some() && queue.head.unwrap() == *state {
                    queue.head = None;
                    if let Some(mut e) = next_entity {
                        e.insert(SetReplacement {
                            entity: state.replacementNext.unwrap(),
                            location: 1,
                            is_state: false,
                        });
                    }
                } else {
                    if let Some(mut e) = pre_entity {
                        e.insert(SetReplacement {
                            entity: state.replacementNext.unwrap(),
                            location: 2,
                            is_state: true,
                        });
                    }
                }
                if queue.tail.is_some() && queue.tail.unwrap() == *state {
                    queue.head = None;
                    if let Some(mut e) = pre_entity {
                        e.insert(SetReplacement {
                            entity: state.replacementNext.unwrap(),
                            location: 2,
                            is_state: false,
                        });
                    }
                } else {
                    if let Some(mut e) = pre_entity {
                        e.insert(SetReplacement {
                            entity: state.replacementNext.unwrap(),
                            location: 2,
                            is_state: true,
                        });
                    }
                }
                state.replacementPrevious = None;
                state.replacementNext = None;
                queue.count -= 1;
            }
            state.replacementPrevious = None;
            state.replacementNext = Some(queue.head.unwrap().entity);
            //head的Entity不是当前的Entity，所以要在下一次遍历时设置，或者使用command.add命令
            if let Some(mut head_entity_mut) = commands.get_entity(queue.head.unwrap().entity) {
                head_entity_mut.insert(SetReplacement {
                    entity: state.entity,
                    location: 1,
                    is_state: true,
                });
            }
            queue.head = Some(state.clone());
        }
    }
    for (entity, mut state, set_info) in &mut set_query {
        if let Some(mut entity_mut) = commands.get_entity(entity) {
            entity_mut.remove::<SetReplacement>();
            if !set_info.is_state {
                match set_info.location {
                    1 => queue.head = Some(state.clone()),
                    2 => queue.tail = Some(state.clone()),
                    3 => queue._lastBeforeStartOfFrame = Some(state.clone()),
                }
            } else {
                match set_info.location {
                    1 => state.replacementPrevious = Some(set_info.entity),
                    2 => state.replacementNext = Some(set_info.entity),
                }
            }
        }
    }
}
