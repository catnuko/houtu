use bevy::prelude::*;

pub struct Plugin;
impl bevy::prelude::Plugin for Plugin {
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
#[derive(Component)]
pub struct EmitMarkTileRendered;

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
    pub fn markStartOfRenderFrame(&mut self) {
        self._lastBeforeStartOfFrame = self.head;
    }
}
fn system(
    mut queue: ResMut<TileReplacementQueue>,
    mut commands: Commands,
    mut mark_tile_rendered_query: Query<
        (Entity, &TileReplacementState),
        With<EmitMarkTileRendered>,
    >,
) {
    for (entity, mut state) in &mut mark_tile_rendered_query {
        let mut entity_mut = commands.get_entity(entity).expect("entity不存在");
        entity_mut.remove::<EmitMarkTileRendered>();
        if queue.head.is_some() && queue.head.unwrap() == *state {
            queue._lastBeforeStartOfFrame = None;
            if queue._lastBeforeStartOfFrame.is_some()
                && *state == queue._lastBeforeStartOfFrame.unwrap()
            {
                queue._lastBeforeStartOfFrame = get_state_of_entity(
                    &mut commands,
                    state.replacementNext.expect("entity不存在"),
                );
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
            let next_entity =
                get_state_of_entity(&mut commands, state.replacementNext.expect("entity不存在"));
            let pre_entity = get_state_of_entity(
                &mut commands,
                state.replacementPrevious.expect("entity不存在"),
            );

            if queue._lastBeforeStartOfFrame.is_some()
                && queue._lastBeforeStartOfFrame.unwrap() == *state
            {
                queue._lastBeforeStartOfFrame = next_entity.clone();
            }
            if queue.head.is_some() && queue.head.unwrap() == *state {
                queue.head = next_entity.clone();
            } else {
                set_state_of_entity(
                    &mut commands,
                    state.replacementPrevious.expect("entity不存在"),
                    state.replacementNext,
                    true,
                );
            }
            if queue.tail.is_some() && queue.tail.unwrap() == *state {
                queue.tail = pre_entity.clone();
            } else {
                set_state_of_entity(
                    &mut commands,
                    state.replacementNext.expect("entity不存在"),
                    state.replacementPrevious,
                    false,
                );
            }
            state.replacementPrevious = None;
            state.replacementNext = None;
            queue.count -= 1;
        }
        state.replacementPrevious = None;
        state.replacementNext = Some(queue.head.unwrap().entity);
        set_state_of_entity(
            &mut commands,
            queue.head.unwrap().entity,
            Some(entity),
            false,
        );
        queue.head = Some(state.clone());
    }
}
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
