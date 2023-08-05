use std::any::type_name;

use bevy::{
    ecs::{bundle::Bundle, prelude::*},
    input::{
        mouse::{MouseMotion, MouseScrollUnit, MouseWheel},
        prelude::*,
    },
    math::{prelude::*, DVec2},
    prelude::*,
    render::{camera::RenderTarget, view::WindowSystem},
    time::Time,
    transform::components::Transform,
    ui::update,
    utils::HashMap,
    window::{PrimaryWindow, WindowRef},
};
use bevy_egui::EguiSet;
use houtu_scene::{Cartesian2, EPSILON14};

use super::egui::{self, EguiWantsFocus};
pub struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ScreenSpaceEventHandlerPlugin);
        app.add_event::<ControlEvent>();
        app.add_system(default_input_map);
        app.add_system(maintain_inertia_system);
        app.insert_resource(Aggregator::default());

        app.insert_resource(UpdateWrap::default());
        app.insert_resource(IsDownWrap::default());
        app.insert_resource(EventStartPositionWrap::default());

        app.insert_resource(MovementWrap::default());
        app.insert_resource(LastMovementWrap::default());
        app.insert_resource(PressTimetWrap::default());
        app.insert_resource(ReleaseTimeWrap::default());

        app.insert_resource(MovementStateWrap::default());
        app.add_startup_system(setup);
    }
}

#[derive(Default, Debug, Clone)]
pub struct LastMovement {
    startPosition: DVec2,
    endPosition: DVec2,
    valid: bool,
}
#[derive(Default, Debug, Clone)]
pub struct Movement {
    pub startPosition: DVec2,
    pub endPosition: DVec2,
}
impl Movement {
    fn into_state(&self, v: bool) -> MovementState {
        MovementState {
            startPosition: self.startPosition.clone(),
            endPosition: self.endPosition.clone(),
            inertiaEnabled: v,
        }
    }
}
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct UpdateWrap(HashMap<&'static str, bool>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct IsDownWrap(HashMap<&'static str, bool>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct EventStartPositionWrap(HashMap<&'static str, DVec2>);
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
    _currentMousePosition: DVec2,
    _buttonsDown: u32,
    _eventStartPosition: DVec2,
}
impl Aggregator {
    pub fn getStartMousePosition(
        &self,
        typeName: &'static str,
        event_start_position_wrap: &EventStartPositionWrap,
    ) -> DVec2 {
        if typeName == WHEEL {
            return self._currentMousePosition.clone();
        }
        return event_start_position_wrap.get(typeName).unwrap().clone();
    }
}
const WHEEL: &'static str = "WHEEL";
const LEFT_DRAG: &'static str = "LEFT_DRAG";
const RIGHT_DRAG: &'static str = "RIGHT_DRAG";
const MIDDLE_DRAG: &'static str = "MIDDLE_DRAG";
const PINCH: &'static str = "PINCH";
const cameraEventType: [&'static str; 4] = [WHEEL, LEFT_DRAG, RIGHT_DRAG, MIDDLE_DRAG];
#[derive(Debug)]
pub struct ControlEventData {
    pub movement: MovementState,
    pub startPosition: DVec2, // pub press_time: f64,
                              // pub release_time: f64,
}
pub enum ControlEvent {
    Tilt(ControlEventData),
    Spin(ControlEventData),
    Zoom(ControlEventData),
}
fn setup(
    mut update_wrap: ResMut<UpdateWrap>,
    // mut movement_wrap: ResMut<MovementWrap>,
    // mut last_movement_wrap: ResMut<LastMovementWrap>,
) {
    for typeName in cameraEventType {
        update_wrap.insert(typeName, true);
    }
}
pub fn default_input_map(
    time: Res<Time>,
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
                    if let Some(v) = is_down_wrap.get(typeName) {
                        if !v {
                            continue;
                        }
                        if !update_wrap.get(typeName).unwrap() {
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
                // println!("left down");
            }
            MouseEvent::LeftUp(p) => {
                aggregator._buttonsDown = (aggregator._buttonsDown - 1).max(0);
                is_down_wrap.insert(LEFT_DRAG, false);
                release_time_wrap.insert(LEFT_DRAG, cur_time);
                // println!("left up");
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
}

#[derive(Debug, Default, Clone)]
pub struct MovementState {
    pub startPosition: DVec2,
    pub endPosition: DVec2,
    pub inertiaEnabled: bool,
}
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct MovementStateWrap(HashMap<&'static str, MovementState>);

pub fn maintain_inertia_system(
    mut control_event_writer: EventWriter<ControlEvent>,
    mut update_wrap: ResMut<UpdateWrap>,
    movement_wrap: ResMut<MovementWrap>,
    event_start_position_wrap: ResMut<EventStartPositionWrap>,
    last_movement_wrap: ResMut<LastMovementWrap>,
    press_time_wrap: ResMut<PressTimetWrap>,
    release_time_wrap: ResMut<ReleaseTimeWrap>,
    aggregator: ResMut<Aggregator>,
    mouse_event_reader: EventReader<MouseEvent>,
    mut movement_state_wrap: ResMut<MovementStateWrap>,
    mut is_down_wrap: ResMut<IsDownWrap>,
    time: Res<Time>,
) {
    for typeName in cameraEventType {
        //isMoving
        if !update_wrap.get(typeName).unwrap() {
            let startPosition =
                aggregator.getStartMousePosition(typeName, &event_start_position_wrap);
            if let Some(movement) = movement_wrap.get(typeName) {
                match typeName {
                    WHEEL => {
                        control_event_writer.send(ControlEvent::Zoom(ControlEventData {
                            movement: movement.into_state(false),
                            startPosition: startPosition,
                        }));
                        activate_inertia(&mut movement_state_wrap, "_lastInertiaZoomMovement");
                    }
                    LEFT_DRAG => {
                        control_event_writer.send(ControlEvent::Spin(ControlEventData {
                            movement: movement.into_state(false),
                            startPosition: startPosition,
                        }));
                        activate_inertia(&mut movement_state_wrap, "_lastInertiaSpinMovement");
                    }
                    MIDDLE_DRAG => {
                        control_event_writer.send(ControlEvent::Tilt(ControlEventData {
                            movement: movement.into_state(false),
                            startPosition: startPosition,
                        }));
                        activate_inertia(&mut movement_state_wrap, "_lastInertiaTiltMovement");
                    }
                    _ => {}
                }
            }
        } else {
            match typeName {
                WHEEL => {
                    if maintain_inertia(
                        &mut movement_state_wrap,
                        typeName,
                        "_lastInertiaZoomMovement",
                        0.9,
                        &press_time_wrap,
                        &release_time_wrap,
                        &last_movement_wrap,
                        &is_down_wrap,
                        &time,
                    ) {
                        let startPosition =
                            aggregator.getStartMousePosition(typeName, &event_start_position_wrap);
                        let movement = movement_state_wrap.get("_lastInertiaZoomMovement").unwrap();
                        control_event_writer.send(ControlEvent::Zoom(ControlEventData {
                            movement: movement.clone(),
                            startPosition: startPosition,
                        }))
                    }
                }
                LEFT_DRAG => {
                    if maintain_inertia(
                        &mut movement_state_wrap,
                        typeName,
                        "_lastInertiaSpinMovement",
                        0.9,
                        &press_time_wrap,
                        &release_time_wrap,
                        &last_movement_wrap,
                        &is_down_wrap,
                        &time,
                    ) {
                        let startPosition =
                            aggregator.getStartMousePosition(typeName, &event_start_position_wrap);
                        let movement = movement_state_wrap.get("_lastInertiaSpinMovement").unwrap();
                        control_event_writer.send(ControlEvent::Zoom(ControlEventData {
                            movement: movement.clone(),
                            startPosition: startPosition,
                        }))
                    }
                }
                MIDDLE_DRAG => {
                    if maintain_inertia(
                        &mut movement_state_wrap,
                        typeName,
                        "_lastInertiaTiltMovement",
                        0.9,
                        &press_time_wrap,
                        &release_time_wrap,
                        &last_movement_wrap,
                        &is_down_wrap,
                        &time,
                    ) {
                        let startPosition =
                            aggregator.getStartMousePosition(typeName, &event_start_position_wrap);
                        let movement = movement_state_wrap.get("_lastInertiaTiltMovement").unwrap();
                        control_event_writer.send(ControlEvent::Zoom(ControlEventData {
                            movement: movement.clone(),
                            startPosition: startPosition,
                        }))
                    }
                }
                _ => {}
            }
        }
        //重置状态
        update_wrap.insert(typeName, true);
    }
}
fn activate_inertia(
    movement_state_wrap: &mut ResMut<MovementStateWrap>,
    lastMovementName: &'static str,
) {
    if let Some(movement_state) = movement_state_wrap.get_mut(lastMovementName) {
        movement_state.inertiaEnabled = true;
    }
    let mut last_movement_name_list: Vec<&'static str>;
    if lastMovementName == "_lastInertiaZoomMovement" {
        last_movement_name_list = [
            "_lastInertiaSpinMovement",
            "_lastInertiaTranslateMovement",
            "_lastInertiaTiltMovement",
        ]
        .into();
    } else if lastMovementName == "_lastInertiaTiltMovement" {
        last_movement_name_list =
            ["_lastInertiaSpinMovement", "_lastInertiaTranslateMovement"].into();
    } else {
        last_movement_name_list = Vec::new();
    }
    for last_movement_name in last_movement_name_list {
        if let Some(movement_state) = movement_state_wrap.get_mut(last_movement_name) {
            movement_state.inertiaEnabled = false;
        }
    }
}
const inertiaMaxClickTimeThreshold: f64 = 0.4;
fn maintain_inertia(
    movement_state_wrap: &mut ResMut<MovementStateWrap>,
    typeName: &'static str,
    lastMovementName: &'static str,
    inertiaConstant: f64,
    press_time_wrap: &ResMut<PressTimetWrap>,
    release_time_wrap: &ResMut<ReleaseTimeWrap>,
    last_movement_wrap: &ResMut<LastMovementWrap>,
    is_down_wrap: &ResMut<IsDownWrap>,
    time: &Res<Time>,
) -> bool {
    let mut movement_state = match movement_state_wrap.get_mut(lastMovementName) {
        None => {
            let v = MovementState::default();
            movement_state_wrap.insert(lastMovementName, v);
            movement_state_wrap.get_mut(lastMovementName).unwrap()
        }
        Some(v) => v,
    };

    let ts = press_time_wrap.get(typeName);
    let tr = release_time_wrap.get(typeName);
    if ts.is_none() || tr.is_none() {
        return false;
    }
    let ts = ts.unwrap();
    let tr = tr.unwrap();

    let threshold = (tr - ts);
    let now = time.elapsed_seconds_f64();
    let fromNow = (now - tr);
    //如果按键释放事件和点击事件之间的时间差在0.4秒内才会保持惯性，滚轮缩放时，阈值=0，所以会保持惯性，而spin和tilt大于0.4，一般不会保持惯性，除非很快的拉动地球才会。
    if threshold < inertiaMaxClickTimeThreshold {
        //随时间增加，从1无限接近于0
        let d = decay(fromNow, inertiaConstant);

        let lastMovement = last_movement_wrap.get(typeName);
        if lastMovement.is_none() || !movement_state.inertiaEnabled {
            return false;
        }
        let lastMovement = lastMovement.unwrap();
        if lastMovement.startPosition.equals_epsilon(
            lastMovement.endPosition,
            Some(EPSILON14),
            None,
        ) {
            return false;
        }
        //不清楚为什么乘以0.5,可能想减小动作幅度
        let mut motion = DVec2::ZERO;
        motion.x = (lastMovement.endPosition.x - lastMovement.startPosition.x) * 0.5;
        motion.y = (lastMovement.endPosition.y - lastMovement.startPosition.y) * 0.5;

        movement_state.startPosition = lastMovement.startPosition.clone();
        // println!(
        //     "startPositin={:?},endPosition={:?},motion={:?}",
        //     lastMovement.startPosition.clone(),
        //     lastMovement.endPosition.clone(),
        //     motion.clone() * d
        // );
        movement_state.endPosition = motion * (d as f64);
        movement_state.endPosition = movement_state.startPosition + movement_state.endPosition;

        // If value from the decreasing exponential function is close to zero,
        // the end coordinates may be NaN.
        if (movement_state.endPosition.x.is_nan()
            || movement_state.endPosition.y.is_nan()
            || movement_state
                .startPosition
                .distance(movement_state.endPosition)
                < 0.5)
        {
            return false;
        }

        if is_down_wrap.get(typeName).is_none() {
            //可以保持惯性，更新相机
            return true;
        }
    }
    //不可更新相机
    return false;
}
/// Base system set to allow ordering of `PanOrbitCamera`
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[system_set(base)]
pub struct PanOrbitCameraSystemSet;
pub struct ScreenSpaceEventHandlerPlugin;
impl bevy::prelude::Plugin for ScreenSpaceEventHandlerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ScreenSpaceEventHandler::default());
        app.insert_resource(PositionsWrap::default());
        app.insert_resource(PreviousPositionsWrap::default());
        app.insert_resource(IsButtonDownWrap::default());
        app.add_event::<MouseEvent>();
        app.add_system(screen_space_event_hanlder_system.in_base_set(PanOrbitCameraSystemSet));
        {
            app.init_resource::<EguiWantsFocus>()
                .add_system(
                    egui::check_egui_wants_focus
                        .after(EguiSet::InitContexts)
                        .before(PanOrbitCameraSystemSet),
                )
                .configure_set(
                    PanOrbitCameraSystemSet.run_if(resource_equals(EguiWantsFocus {
                        prev: false,
                        curr: false,
                    })),
                );
        }
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
pub struct PositionsWrap(HashMap<&'static str, DVec2>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct PreviousPositionsWrap(HashMap<&'static str, DVec2>);
#[derive(Default, Debug, Resource, Deref, DerefMut)]
pub struct IsButtonDownWrap(HashMap<&'static str, bool>);

#[derive(Resource)]
pub struct ScreenSpaceEventHandler {
    _primaryPosition: DVec2,
    _primaryStartPosition: DVec2,
    _primaryPreviousPosition: DVec2,
    _isPinching: bool,
    _pinchingPosition: DVec2,
    _clickPixelTolerance: f64,
}
impl Default for ScreenSpaceEventHandler {
    fn default() -> Self {
        Self {
            _primaryPreviousPosition: DVec2::ZERO,
            _pinchingPosition: DVec2::ZERO,
            _primaryStartPosition: DVec2::ZERO,
            _primaryPosition: DVec2::ZERO,
            _clickPixelTolerance: 5.0,
            _isPinching: false,
        }
    }
}

fn decay(time: f64, coefficient: f64) -> f64 {
    if time < 0. {
        return 0.0;
    }

    let tau = (1.0 - coefficient) * 25.0;
    return (-tau * time).exp();
}
pub enum MouseEvent {
    MouseMove(Movement),
    PinchStart(Movement),
    PinchEnd(Movement),
    Wheel(f64),
    LeftDown(DVec2),
    RightDown(DVec2),
    MiddleDown(DVec2),
    LeftUp(DVec2),
    RightUp(DVec2),
    MiddleUp(DVec2),
    LeftClick(DVec2),
    RightClick(DVec2),
    MiddleClick(DVec2),
}
pub fn screen_space_event_hanlder_system(
    mut events: EventWriter<MouseEvent>,
    mut mouse_wheel_reader: EventReader<MouseWheel>,
    mouse_buttons: Res<Input<MouseButton>>,
    primary_query: Query<&Window, With<PrimaryWindow>>,
    // mut positionsWrap: ResMut<ReleaseTimeWrap>,
    // mut previousPositionsWrap: ResMut<ReleaseTimeWrap>,
    mut is_button_down_wrap: ResMut<IsButtonDownWrap>,
    mut screen_space_event_hanlder: ResMut<ScreenSpaceEventHandler>,
    camera_query: Query<(&Camera)>,
) {
    for (camera) in camera_query.iter() {
        let Ok(window) = primary_query.get_single() else {
            return;
        };
        let Some(raw_position) = window.cursor_position() else {
            return;
        };
        let Some((left_top,_)) = camera.physical_viewport_rect() else {
            return;
        };
        let position = DVec2::new(
            raw_position.x as f64,
            window.height() as f64 - raw_position.y as f64,
        );

        //收集移动事件
        screen_space_event_hanlder._primaryPosition = position.clone();
        let mut movement = Movement::default();
        movement.startPosition = screen_space_event_hanlder._primaryPreviousPosition.clone();
        movement.endPosition = position.clone();
        events.send(MouseEvent::MouseMove(movement));
        screen_space_event_hanlder._primaryPreviousPosition = position.clone();

        //收集Down事件
        if mouse_buttons.any_just_pressed([
            MouseButton::Left,
            MouseButton::Middle,
            MouseButton::Right,
        ]) {
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
        if mouse_buttons.any_just_released([
            MouseButton::Left,
            MouseButton::Middle,
            MouseButton::Right,
        ]) {
            let mut button: &'static str;
            let mut button_my: &'static str;
            if mouse_buttons.just_released(MouseButton::Left) {
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
            } else if mouse_buttons.just_released(MouseButton::Right) {
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
            } else if mouse_buttons.just_released(MouseButton::Middle) {
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

        if mouse_buttons.just_pressed(MouseButton::Middle)
            && !screen_space_event_hanlder._isPinching
        {
            screen_space_event_hanlder._isPinching = true;
            screen_space_event_hanlder._pinchingPosition = position.clone();
        }
        if mouse_buttons.just_released(MouseButton::Middle)
            && (screen_space_event_hanlder._isPinching)
        {
            screen_space_event_hanlder._isPinching = false;
            let mut movement = Movement::default();
            movement.startPosition = screen_space_event_hanlder._pinchingPosition.clone();
            movement.endPosition = position.clone();
            events.send(MouseEvent::PinchEnd(movement));
        }
        for ev in mouse_wheel_reader.iter() {
            let delta: f64;
            match ev.unit {
                MouseScrollUnit::Line => {
                    //[-1,1]=>[-100,100]
                    delta = (ev.y as f64) * 100.;
                }
                MouseScrollUnit::Pixel => {
                    delta = -(ev.y as f64);
                }
            };
            events.send(MouseEvent::Wheel(delta));
        }
    }
}
fn checkPixelTolerance(startPosition: &DVec2, endPosition: &DVec2, pixelTolerance: f64) -> bool {
    let xDiff = startPosition.x - endPosition.x;
    let yDiff = startPosition.y - endPosition.y;
    let totalPixels = (xDiff * xDiff + yDiff * yDiff).sqrt();

    return totalPixels < pixelTolerance;
}
