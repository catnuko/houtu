use std::any::type_name;

use bevy::{
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        prelude::*,
    },
    math::{prelude::*, DVec2},
    prelude::*,
    time::Time,
    transform::components::Transform,
    ui::update,
    utils::HashMap,
};
pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ControlEvent>();
        app.add_event::<CameraControlEvent>();
        app.add_system(default_input_map);
        app.insert_resource(UpdateWrap::default());
        app.insert_resource(IsDownWrap::default());
        app.insert_resource(EventStartPositionWrap::default());
        app.insert_resource(MovementWrap::default());
        app.insert_resource(LastMovementWrap::default());
        app.insert_resource(PressTimetWrap::default());
        app.insert_resource(ReleaseTimeWrap::default());
    }
}
pub enum ControlEvent {
    Orbit(Vec2),
    TranslateTarget(Vec2),
    Zoom(f32),
}
#[derive(Default, Debug, Clone)]
pub struct LastMovement {
    startPosition: Vec2,
    endPosition: Vec2,
    valid: bool,
}
#[derive(Default, Debug, Clone)]
pub struct Movement {
    startPosition: Vec2,
    endPosition: Vec2,
}
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct UpdateWrap(HashMap<&'static str, bool>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct IsDownWrap(HashMap<&'static str, bool>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct EventStartPositionWrap(HashMap<&'static str, bool>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct MovementWrap(HashMap<&'static str, Movement>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct LastMovementWrap(HashMap<&'static str, LastMovement>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct PressTimetWrap(HashMap<&'static str, f64>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct ReleaseTimeWrap(HashMap<&'static str, f64>);
const WHEEL: &'static str = "WHEEL";
const LEFT_DRAG: &'static str = "LEFT_DRAG";
const RIGHT_DRAG: &'static str = "RIGHT_DRAG";
const MIDDLE_DRAG: &'static str = "MIDDLE_DRAG";
const PINCH: &'static str = "PINCH";
const cameraEventType: [&'static str; 5] = [WHEEL, LEFT_DRAG, RIGHT_DRAG, MIDDLE_DRAG, PINCH];
pub fn default_input_map(
    time: Res<Time>,
    mut events: EventWriter<ControlEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut update_wrap: ResMut<UpdateWrap>,
    mut is_down_wrap: ResMut<IsDownWrap>,
    mut event_start_position_wrap: ResMut<EventStartPositionWrap>,
    mut movement_wrap: ResMut<MovementWrap>,
    mut last_movement_wrap: ResMut<LastMovementWrap>,
    mut press_time_wrap: ResMut<PressTimetWrap>,
    mut release_time_wrap: ResMut<ReleaseTimeWrap>,
) {
    let cur_time = time.elapsed_seconds_f64();

    // let mut cursor_delta = Vec2::ZERO;
    // for event in mouse_motion_events.iter() {
    //     cursor_delta += event.delta;
    // }

    // if keyboard.pressed(KeyCode::LControl) {
    //     events.send(ControlEvent::Orbit(mouse_rotate_sensitivity * cursor_delta));
    // }

    // if mouse_buttons.pressed(MouseButton::Right) {
    //     events.send(ControlEvent::TranslateTarget(
    //         mouse_translate_sensitivity * cursor_delta,
    //     ));
    // }
    //滚轮滚动
    let mut scalar = 1.0;
    for event in mouse_wheel_reader.iter() {
        update_wrap.insert(WHEEL, true);
        let mut movement = match movement_wrap.get_mut(WHEEL) {
            None => {
                let v = Movement::default();
                movement_wrap.insert(WHEEL, v);
                movement_wrap.get_mut(WHEEL).unwrap()
            }
            Some(v) => v,
        };
        let mut lastMovement = match last_movement_wrap.get_mut(WHEEL) {
            None => {
                let v = LastMovement::default();
                last_movement_wrap.insert(WHEEL, v);
                last_movement_wrap.get_mut(WHEEL).unwrap()
            }
            Some(v) => v,
        };
        let delta = event.y;
        let arcLength = 7.5 * delta.to_radians();
        press_time_wrap.insert(WHEEL, cur_time);
        release_time_wrap.insert(WHEEL, cur_time);
        movement.endPosition.x = 0.0;
        movement.endPosition.y = arcLength;
        lastMovement.endPosition = movement.endPosition.clone();
        lastMovement.valid = true;
        update_wrap.insert(WHEEL, false);
    }
    //鼠标拖动
    for event in mouse_motion_events.iter() {
        for typeName in cameraEventType {
            let mut movement = match movement_wrap.get_mut(typeName) {
                None => {
                    let v = Movement::default();
                    movement_wrap.insert(typeName, v);
                    movement_wrap.get_mut(typeName).unwrap()
                }
                Some(v) => v,
            };
            let mut lastMovement = match last_movement_wrap.get_mut(typeName) {
                None => {
                    let v = LastMovement::default();
                    last_movement_wrap.insert(typeName, v);
                    last_movement_wrap.get_mut(typeName).unwrap()
                }
                Some(v) => v,
            };

            if is_down_wrap.get(typeName).is_some() {
                if !update_wrap.get(typeName).is_none() {
                    movement.endPosition = event.delta.clone();
                } else {
                    lastMovement.startPosition = movement.startPosition.clone();
                    lastMovement.endPosition = movement.endPosition.clone();
                    lastMovement.valid = true;

                    movement.startPosition = movement.startPosition.clone();
                    movement.endPosition = event.delta.clone();
                    lastMovement.valid = false;
                }
            }
        }
    }
    events.send(ControlEvent::Zoom(scalar));
}

pub struct CameraControlEvent {}
pub fn camera_control_event_type(
    time: Res<Time>,
    mut events: EventWriter<ControlEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mouse_buttons: Res<Input<MouseButton>>,
    keyboard: Res<Input<KeyCode>>,
    mut update_wrap: ResMut<UpdateWrap>,
    mut is_down_wrap: ResMut<IsDownWrap>,
    mut event_start_position_wrap: ResMut<EventStartPositionWrap>,
    mut movement_wrap: ResMut<MovementWrap>,
    mut last_movement_wrap: ResMut<LastMovementWrap>,
    mut press_time_wrap: ResMut<PressTimetWrap>,
    mut release_time_wrap: ResMut<ReleaseTimeWrap>,
) {
}
