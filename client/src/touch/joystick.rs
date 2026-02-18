use bevy::prelude::*;
use bevy::input::touch::{TouchInput, TouchPhase};

use crate::game::GameState;
use super::{TouchDevice, touch_playing};

pub struct JoystickPlugin;

impl Plugin for JoystickPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(JoystickState::default())
            .add_systems(
                OnEnter(GameState::Playing),
                spawn_joystick_ui.run_if(|td: Res<TouchDevice>| td.detected),
            )
            .add_systems(OnExit(GameState::Playing), despawn_joystick_ui)
            .add_systems(
                Update,
                (
                    touch_joystick_input,
                    update_joystick_ui,
                )
                    .chain()
                    .run_if(touch_playing),
            );
    }
}

#[derive(Resource)]
pub struct JoystickState {
    pub active: bool,
    pub touch_id: Option<u64>,
    pub origin: Vec2,
    pub current: Vec2,
    pub max_radius: f32,
}

impl Default for JoystickState {
    fn default() -> Self {
        Self {
            active: false,
            touch_id: None,
            origin: Vec2::ZERO,
            current: Vec2::ZERO,
            max_radius: 80.0,
        }
    }
}

#[derive(Component)]
struct JoystickHint;

#[derive(Component)]
struct JoystickBase;

#[derive(Component)]
struct JoystickThumb;

#[derive(Component)]
struct JoystickRoot;

fn spawn_joystick_ui(mut commands: Commands) {
    // Hint circle (always visible, very faint)
    commands.spawn((
        JoystickRoot,
        JoystickHint,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(10.0),
            bottom: Val::Percent(10.0),
            width: Val::Px(120.0),
            height: Val::Px(120.0),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
        GlobalZIndex(50),
    ));

    // Base circle (hidden until touch)
    commands.spawn((
        JoystickRoot,
        JoystickBase,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(120.0),
            height: Val::Px(120.0),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.25)),
        Visibility::Hidden,
        GlobalZIndex(51),
    ));

    // Thumb circle (hidden until touch)
    commands.spawn((
        JoystickRoot,
        JoystickThumb,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::Px(50.0),
            height: Val::Px(50.0),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.5)),
        Visibility::Hidden,
        GlobalZIndex(52),
    ));
}

fn despawn_joystick_ui(
    mut commands: Commands,
    query: Query<Entity, With<JoystickRoot>>,
    mut joystick: ResMut<JoystickState>,
) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
    *joystick = JoystickState::default();
}

fn touch_joystick_input(
    mut touch_events: MessageReader<TouchInput>,
    mut joystick: ResMut<JoystickState>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let window_width = window.width();

    for event in touch_events.read() {
        match event.phase {
            TouchPhase::Started => {
                // Only activate if touch is in left 40% of screen and no joystick active
                if !joystick.active && event.position.x < window_width * 0.4 {
                    joystick.active = true;
                    joystick.touch_id = Some(event.id);
                    joystick.origin = event.position;
                    joystick.current = event.position;
                }
            }
            TouchPhase::Moved => {
                if joystick.touch_id == Some(event.id) {
                    joystick.current = event.position;
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                if joystick.touch_id == Some(event.id) {
                    joystick.active = false;
                    joystick.touch_id = None;
                }
            }
        }
    }
}

fn update_joystick_ui(
    joystick: Res<JoystickState>,
    windows: Query<&Window>,
    mut hint_query: Query<&mut Visibility, (With<JoystickHint>, Without<JoystickBase>, Without<JoystickThumb>)>,
    mut base_query: Query<(&mut Visibility, &mut Node), (With<JoystickBase>, Without<JoystickHint>, Without<JoystickThumb>)>,
    mut thumb_query: Query<(&mut Visibility, &mut Node), (With<JoystickThumb>, Without<JoystickHint>, Without<JoystickBase>)>,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let _window_height = window.height();

    if joystick.active {
        // Hide hint
        for mut vis in &mut hint_query {
            *vis = Visibility::Hidden;
        }

        // Show and position base at origin
        for (mut vis, mut node) in &mut base_query {
            *vis = Visibility::Inherited;
            node.left = Val::Px(joystick.origin.x - 60.0);
            node.top = Val::Px(joystick.origin.y - 60.0);
        }

        // Show and position thumb at current (clamped)
        let delta = joystick.current - joystick.origin;
        let dist = delta.length();
        let clamped = if dist > joystick.max_radius {
            joystick.origin + delta.normalize() * joystick.max_radius
        } else {
            joystick.current
        };

        for (mut vis, mut node) in &mut thumb_query {
            *vis = Visibility::Inherited;
            node.left = Val::Px(clamped.x - 25.0);
            node.top = Val::Px(clamped.y - 25.0);
        }
    } else {
        // Show hint, hide base + thumb
        for mut vis in &mut hint_query {
            *vis = Visibility::Inherited;
        }
        for (mut vis, _) in &mut base_query {
            *vis = Visibility::Hidden;
        }
        for (mut vis, _) in &mut thumb_query {
            *vis = Visibility::Hidden;
        }
    }
}

