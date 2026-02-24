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
            max_radius: 500.0,
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
    // All sizes use Vmin (% of smaller viewport dimension) for DPI independence.
    // On a 400px-short phone: 25 Vmin = 100px, on 800px = 200px — scales naturally.

    // Hint circle — small visual indicator, border only (no fill to avoid GPU issues)
    commands.spawn((
        JoystickRoot,
        JoystickHint,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(8.0),
            bottom: Val::Percent(5.0),
            width: Val::VMin(25.0),
            height: Val::VMin(25.0),
            border: UiRect::all(Val::VMin(0.4)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(Color::NONE),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.15)),
        GlobalZIndex(50),
    ));

    // Base circle (hidden until touch) — small visual, activation area is handled by input code
    commands.spawn((
        JoystickRoot,
        JoystickBase,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            top: Val::Px(0.0),
            width: Val::VMin(25.0),
            height: Val::VMin(25.0),
            border: UiRect::all(Val::VMin(0.3)),
            border_radius: BorderRadius::all(Val::Percent(50.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.08)),
        BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.2)),
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
            width: Val::VMin(15.0),
            height: Val::VMin(15.0),
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
    let window_height = window.height();
    let vmin = window_width.min(window_height);

    // Compute fixed joystick center from the hint circle position
    // Hint visual is at left: 8%, bottom: 5%, size 25 Vmin
    let joystick_size = vmin * 0.25;
    let hint_center = Vec2::new(
        window_width * 0.08 + joystick_size * 0.5,
        window_height - (window_height * 0.05 + joystick_size * 0.5),
    );
    // Accept touches within a generous area around the joystick
    let activation_radius = joystick_size * 2.0;
    // Max drag radius scales with viewport
    joystick.max_radius = vmin * 0.6;

    for event in touch_events.read() {
        match event.phase {
            TouchPhase::Started => {
                let dist = event.position.distance(hint_center);
                if !joystick.active && dist < activation_radius {
                    joystick.active = true;
                    joystick.touch_id = Some(event.id);
                    joystick.origin = hint_center;
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
    let Ok(window) = windows.single() else { return };
    let vmin = window.width().min(window.height());
    let base_half = vmin * 0.25 * 0.5;  // 25 Vmin / 2
    let thumb_half = vmin * 0.15 * 0.5; // 15 Vmin / 2

    if joystick.active {
        // Hide hint
        for mut vis in &mut hint_query {
            *vis = Visibility::Hidden;
        }

        // Show and position base at fixed origin, centered
        for (mut vis, mut node) in &mut base_query {
            *vis = Visibility::Inherited;
            node.left = Val::Px(joystick.origin.x - base_half);
            node.top = Val::Px(joystick.origin.y - base_half);
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
            node.left = Val::Px(clamped.x - thumb_half);
            node.top = Val::Px(clamped.y - thumb_half);
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

