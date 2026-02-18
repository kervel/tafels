use bevy::prelude::*;
use bevy::input::touch::{TouchInput, TouchPhase, Touches};

use crate::camera::orbit::OrbitCamera;
use super::{touch_playing, joystick::JoystickState};

pub struct TouchCameraPlugin;

impl Plugin for TouchCameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TouchCameraState::default())
            .insert_resource(PinchState::default())
            .add_systems(
                Update,
                (touch_camera_input, touch_pinch_zoom)
                    .run_if(touch_playing),
            );
    }
}

#[derive(Resource, Default)]
struct TouchCameraState {
    touch_id: Option<u64>,
    last_position: Vec2,
}

#[derive(Resource, Default)]
struct PinchState {
    last_distance: Option<f32>,
}

fn touch_camera_input(
    mut _touch_events: MessageReader<TouchInput>,
    _joystick: Res<JoystickState>,
    mut _cam_state: ResMut<TouchCameraState>,
    mut orbit_query: Query<&mut OrbitCamera>,
    _touches: Res<Touches>,
) {
    // On touch devices, camera always auto-follows behind the player.
    // No manual orbit — only pinch-zoom is allowed.
    for mut orbit in &mut orbit_query {
        orbit.auto_follow = true;
    }
}

fn touch_pinch_zoom(
    touches: Res<Touches>,
    mut pinch: ResMut<PinchState>,
    mut orbit_query: Query<&mut OrbitCamera>,
) {
    let pressed: Vec<_> = touches.iter().collect();

    if pressed.len() == 2 {
        let a = pressed[0].position();
        let b = pressed[1].position();
        let current_distance = a.distance(b);

        if let Some(prev) = pinch.last_distance {
            let delta = prev - current_distance;
            // Positive delta = fingers moved closer = zoom in
            for mut orbit in &mut orbit_query {
                orbit.distance += delta * 0.15;
                orbit.distance = orbit.distance.clamp(8.0, 80.0);
            }
        }
        pinch.last_distance = Some(current_distance);
    } else {
        pinch.last_distance = None;
    }
}
