use std::any::type_name;

use bevy::{
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        prelude::*,
    },
    math::{prelude::*, DVec2},
    prelude::*,
    render::view::WindowSystem,
    time::Time,
    transform::components::Transform,
    ui::update,
    utils::HashMap,
    window::PrimaryWindow,
};
pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ScreenSpaceEventHandlerPlugin);
        app.add_event::<ControlEvent>();
        app.add_system(default_input_map);
        app.insert_resource(Aggregator::default());
        app.insert_resource(UpdateWrap::default());
        app.insert_resource(IsDownWrap::default());
        app.insert_resource(EventStartPositionWrap::default());
        app.insert_resource(MovementWrap::default());
        app.insert_resource(LastMovementWrap::default());
        app.insert_resource(PressTimetWrap::default());
        app.insert_resource(ReleaseTimeWrap::default());
    }
}

#[derive(Default, Debug, Clone)]
pub struct LastMovement {
    startPosition: Vec2,
    endPosition: Vec2,
    valid: bool,
}
#[derive(Default, Debug, Clone)]
pub struct Movement {
    pub startPosition: Vec2,
    pub endPosition: Vec2,
}
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct UpdateWrap(HashMap<&'static str, bool>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct IsDownWrap(HashMap<&'static str, bool>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct EventStartPositionWrap(HashMap<&'static str, Vec2>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct MovementWrap(HashMap<&'static str, Movement>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct LastMovementWrap(HashMap<&'static str, LastMovement>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct PressTimetWrap(HashMap<&'static str, f64>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct ReleaseTimeWrap(HashMap<&'static str, f64>);
#[derive(Resource, Default)]
pub struct Aggregator {
    _currentMousePosition: Vec2,
    _buttonsDown: u32,
    _eventStartPosition: Vec2,
}
const WHEEL: &'static str = "WHEEL";
const LEFT_DRAG: &'static str = "LEFT_DRAG";
const RIGHT_DRAG: &'static str = "RIGHT_DRAG";
const MIDDLE_DRAG: &'static str = "MIDDLE_DRAG";
const PINCH: &'static str = "PINCH";
const cameraEventType: [&'static str; 4] = [WHEEL, LEFT_DRAG, RIGHT_DRAG, MIDDLE_DRAG];
pub struct ControlEventData {
    pub movement: Movement,
    pub press_time: f64,
    pub release_time: f64,
}
pub enum ControlEvent {
    Tilt(pub ControlEventData),
    Spin(pub ControlEventData),
    Zoom(pub ControlEventData),
}
pub fn default_input_map(
    time: Res<Time>,
    mut control_event_writer: EventWriter<ControlEvent>,
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
    mut aggregator: ResMut<Aggregator>,
    mut mouse_event_reader: EventReader<MouseEvent>,
) {
    let cur_time = time.elapsed_seconds_f64();
    for event in mouse_event_reader.iter() {
        match event {
            MouseEvent::Wheel(delta) => {
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
                let arcLength = 7.5 * delta.to_radians();
                press_time_wrap.insert(WHEEL, cur_time);
                release_time_wrap.insert(WHEEL, cur_time);
                movement.endPosition.x = 0.0;
                movement.endPosition.y = arcLength;
                lastMovement.endPosition = movement.endPosition.clone();
                lastMovement.valid = true;
                update_wrap.insert(WHEEL, false);
            }
            MouseEvent::MouseMove(mouse_movemet_event) => {
                for typeName in cameraEventType {
                    update_wrap.insert(typeName, true);
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
                            movement.endPosition = mouse_movemet_event.endPosition.clone();
                        } else {
                            lastMovement.startPosition = movement.startPosition.clone();
                            lastMovement.endPosition = movement.endPosition.clone();
                            lastMovement.valid = true;

                            movement.startPosition = mouse_movemet_event.startPosition.clone();
                            movement.endPosition = mouse_movemet_event.endPosition.clone();
                            update_wrap.insert(typeName, false);
                        }
                    }
                }
                aggregator._currentMousePosition = mouse_movemet_event.endPosition.clone();
            }
            MouseEvent::LeftDown(p) => {
                let mut lastMovement = match last_movement_wrap.get_mut(LEFT_DRAG) {
                    None => {
                        let v = LastMovement::default();
                        last_movement_wrap.insert(LEFT_DRAG, v);
                        last_movement_wrap.get_mut(LEFT_DRAG).unwrap()
                    }
                    Some(v) => v,
                };
                aggregator._buttonsDown += 1;
                lastMovement.valid = false;
                is_down_wrap.insert(LEFT_DRAG, true);
                press_time_wrap.insert(LEFT_DRAG, cur_time);
                event_start_position_wrap.insert(LEFT_DRAG, p.clone());
            }
            MouseEvent::LeftUp(p) => {
                aggregator._buttonsDown = (aggregator._buttonsDown - 1).max(0);
                is_down_wrap.insert(LEFT_DRAG, false);
                release_time_wrap.insert(LEFT_DRAG, cur_time);
            }
            MouseEvent::RightDown(p) => {
                let mut lastMovement = match last_movement_wrap.get_mut(RIGHT_DRAG) {
                    None => {
                        let v = LastMovement::default();
                        last_movement_wrap.insert(RIGHT_DRAG, v);
                        last_movement_wrap.get_mut(RIGHT_DRAG).unwrap()
                    }
                    Some(v) => v,
                };
                aggregator._buttonsDown += 1;
                lastMovement.valid = false;
                is_down_wrap.insert(RIGHT_DRAG, true);
                press_time_wrap.insert(RIGHT_DRAG, cur_time);
                event_start_position_wrap.insert(RIGHT_DRAG, p.clone());
            }
            MouseEvent::RightUp(p) => {
                aggregator._buttonsDown = (aggregator._buttonsDown - 1).max(0);
                is_down_wrap.insert(RIGHT_DRAG, false);
                release_time_wrap.insert(RIGHT_DRAG, cur_time);
            }
            MouseEvent::MiddleDown(p) => {
                let mut lastMovement = match last_movement_wrap.get_mut(MIDDLE_DRAG) {
                    None => {
                        let v = LastMovement::default();
                        last_movement_wrap.insert(MIDDLE_DRAG, v);
                        last_movement_wrap.get_mut(MIDDLE_DRAG).unwrap()
                    }
                    Some(v) => v,
                };
                aggregator._buttonsDown += 1;
                lastMovement.valid = false;
                is_down_wrap.insert(MIDDLE_DRAG, true);
                press_time_wrap.insert(MIDDLE_DRAG, cur_time);
                event_start_position_wrap.insert(MIDDLE_DRAG, p.clone());
            }
            MouseEvent::MiddleUp(p) => {
                aggregator._buttonsDown = (aggregator._buttonsDown - 1).max(0);
                is_down_wrap.insert(MIDDLE_DRAG, false);
                release_time_wrap.insert(MIDDLE_DRAG, cur_time);
            }
            _ => {}
        }
    }

    for typeName in cameraEventType {
        //isMoving
        if !update_wrap.get(typeName).expect("没有controlType") {
            if let Some(movement) = movement_wrap.get(typeName) {
                let press_time = press_time_wrap.get(typeName).unwrap();
                let release_time = release_time_wrap.get(typeName).unwrap();
                match typeName {
                    WHEEL => control_event_writer.send(ControlEvent::Zoom(ControlEventData {
                        movement: movement.clone(),
                        press_time: press_time.clone(),
                        release_time: release_time.clone(),
                    })),
                    LEFT_DRAG => control_event_writer.send(ControlEvent::Spin(ControlEventData {
                        movement: movement.clone(),
                        press_time: press_time.clone(),
                        release_time: release_time.clone(),
                    })),
                    MIDDLE_DRAG => {
                        control_event_writer.send(ControlEvent::Tilt(ControlEventData {
                            movement: movement.clone(),
                            press_time: press_time.clone(),
                            release_time: release_time.clone(),
                        }))
                    }
                    _ => {}
                }
            }
        }
    }
}

pub struct ScreenSpaceEventHandlerPlugin;
impl bevy::prelude::Plugin for ScreenSpaceEventHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScreenSpaceEventHandler::default());
        app.insert_resource(PositionsWrap::default());
        app.insert_resource(PreviousPositionsWrap::default());
        app.insert_resource(IsButtonDownWrap::default());
        app.add_event::<MouseEvent>();
        app.add_system(screen_space_event_hanlder_system);
    }
}
const Left: &'static str = "Left";
const Right: &'static str = "Right";
const Middle: &'static str = "Middle";
const LeftDown: &'static str = "LeftDown";
const RightDown: &'static str = "RightDown";
const MiddleDown: &'static str = "MiddleDown";
const mouseEvent: [&'static str; 3] = [Left, Right, Middle];
fn getMouseButtonName(mouseButton: MouseButton) -> &'static str {
    return match mouseButton {
        MouseButton::Left => Left,
        MouseButton::Right => Right,
        MouseButton::Middle => Middle,
        _ => {
            panic!("")
        }
    };
}
fn getMyMouseButtonName(mouseButton: MouseButton) -> &'static str {
    return match mouseButton {
        MouseButton::Left => LeftDown,
        MouseButton::Right => RightDown,
        MouseButton::Middle => MiddleDown,
        _ => {
            panic!("")
        }
    };
}
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct PositionsWrap(HashMap<&'static str, Vec2>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct PreviousPositionsWrap(HashMap<&'static str, Vec2>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct IsButtonDownWrap(HashMap<&'static str, bool>);

#[derive(Resource)]
pub struct ScreenSpaceEventHandler {
    _primaryPosition: Vec2,
    _primaryStartPosition: Vec2,
    _primaryPreviousPosition: Vec2,
    _isPinching: bool,
    _pinchingPosition: Vec2,
    _clickPixelTolerance: f32,
}
impl Default for ScreenSpaceEventHandler {
    fn default() -> Self {
        Self {
            _primaryPreviousPosition: Vec2::ZERO,
            _pinchingPosition: Vec2::ZERO,
            _primaryStartPosition: Vec2::ZERO,
            _primaryPosition: Vec2::ZERO,
            _clickPixelTolerance: 5.0,
            _isPinching: false,
        }
    }
}
pub enum MouseEvent {
    MouseMove(Movement),
    PinchStart(Movement),
    PinchEnd(Movement),
    Wheel(f32),
    LeftDown(Vec2),
    RightDown(Vec2),
    MiddleDown(Vec2),
    LeftUp(Vec2),
    RightUp(Vec2),
    MiddleUp(Vec2),
    LeftClick(Vec2),
    RightClick(Vec2),
    MiddleClick(Vec2),
}
pub fn screen_space_event_hanlder_system(
    mut events: EventWriter<MouseEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mouse_buttons: Res<Input<MouseButton>>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    mut positionsWrap: ResMut<ReleaseTimeWrap>,
    mut previousPositionsWrap: ResMut<ReleaseTimeWrap>,
    mut is_button_down_wrap: ResMut<IsButtonDownWrap>,
    mut screen_space_event_hanlder: ResMut<ScreenSpaceEventHandler>,
) {
    let Ok(window) = primary_query.get_single() else {
        return;
    };
    let Some(position) = window.cursor_position() else {
        return;
    };
    //收集移动事件
    screen_space_event_hanlder._primaryPosition = position.clone();
    let mut movement = Movement::default();
    movement.startPosition = screen_space_event_hanlder._primaryPreviousPosition.clone();
    movement.endPosition = position.clone();
    events.send(MouseEvent::MouseMove(movement));
    screen_space_event_hanlder._primaryPreviousPosition = position.clone();

    //收集Down事件
    if mouse_buttons.any_just_pressed([MouseButton::Left, MouseButton::Middle, MouseButton::Right])
    {
        let mut button: &'static str;
        let mut button_my: &'static str;
        if mouse_buttons.just_pressed(MouseButton::Left) {
            button = getMouseButtonName(MouseButton::Left);
            button_my = getMyMouseButtonName(MouseButton::Left);
            events.send(MouseEvent::LeftDown(position.clone()));
        } else if mouse_buttons.just_pressed(MouseButton::Right) {
            button = getMouseButtonName(MouseButton::Right);
            button_my = getMyMouseButtonName(MouseButton::Right);
            events.send(MouseEvent::RightDown(position.clone()));
        } else if mouse_buttons.just_pressed(MouseButton::Middle) {
            button = getMouseButtonName(MouseButton::Middle);
            button_my = getMyMouseButtonName(MouseButton::Middle);
            events.send(MouseEvent::MiddleDown(position.clone()));
        } else {
            return;
        }
        is_button_down_wrap.insert(button, true);
        screen_space_event_hanlder._primaryPosition = position.clone();
        screen_space_event_hanlder._primaryStartPosition = position.clone();
        screen_space_event_hanlder._primaryPreviousPosition = position.clone();
    }
    //收集Up和Click事件
    if mouse_buttons.any_just_released([MouseButton::Left, MouseButton::Middle, MouseButton::Right])
    {
        let mut button: &'static str;
        let mut button_my: &'static str;
        if mouse_buttons.just_pressed(MouseButton::Left) {
            button = getMouseButtonName(MouseButton::Left);
            button_my = getMyMouseButtonName(MouseButton::Left);
            events.send(MouseEvent::LeftUp(position.clone()));
            if checkPixelTolerance(
                &screen_space_event_hanlder._primaryStartPosition,
                &position,
                screen_space_event_hanlder._clickPixelTolerance,
            ) {
                events.send(MouseEvent::LeftClick(position.clone()));
            }
        } else if mouse_buttons.just_pressed(MouseButton::Right) {
            button = getMouseButtonName(MouseButton::Right);
            button_my = getMyMouseButtonName(MouseButton::Right);
            events.send(MouseEvent::RightUp(position.clone()));
            if checkPixelTolerance(
                &screen_space_event_hanlder._primaryStartPosition,
                &position,
                screen_space_event_hanlder._clickPixelTolerance,
            ) {
                events.send(MouseEvent::RightClick(position.clone()));
            }
        } else if mouse_buttons.just_pressed(MouseButton::Middle) {
            button = getMouseButtonName(MouseButton::Middle);
            button_my = getMyMouseButtonName(MouseButton::Middle);
            events.send(MouseEvent::MiddleUp(position.clone()));
            if checkPixelTolerance(
                &screen_space_event_hanlder._primaryStartPosition,
                &position,
                screen_space_event_hanlder._clickPixelTolerance,
            ) {
                events.send(MouseEvent::MiddleClick(position.clone()));
            }
        } else {
            return;
        }
        is_button_down_wrap.insert(button, false);
    }

    if mouse_buttons.just_pressed(MouseButton::Middle) && !screen_space_event_hanlder._isPinching {
        screen_space_event_hanlder._isPinching = true;
        screen_space_event_hanlder._pinchingPosition = position.clone();
    }
    if mouse_buttons.just_released(MouseButton::Middle) && (screen_space_event_hanlder._isPinching)
    {
        screen_space_event_hanlder._isPinching = false;
        let mut movement = Movement::default();
        movement.startPosition = screen_space_event_hanlder._pinchingPosition.clone();
        movement.endPosition = position.clone();
        events.send(MouseEvent::PinchEnd(movement));
    }
    for ev in mouse_wheel_reader.iter() {
        let delta: f32;
        match ev.unit {
            MouseScrollUnit::Line => {
                println!(
                    "Scroll (line units): vertical: {}, horizontal: {}",
                    ev.y, ev.x
                );
                delta = -ev.y * 40.0;
            }
            MouseScrollUnit::Pixel => {
                delta = -ev.y;
            }
        };
        events.send(MouseEvent::Wheel(delta));
    }
}
fn checkPixelTolerance(startPosition: &Vec2, endPosition: &Vec2, pixelTolerance: f32) -> bool {
    let xDiff = startPosition.x - endPosition.x;
    let yDiff = startPosition.y - endPosition.y;
    let totalPixels = (xDiff * xDiff + yDiff * yDiff).sqrt();

    return totalPixels < pixelTolerance;
}
