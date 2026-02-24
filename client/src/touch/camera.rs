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
    mut touch_events: MessageReader<TouchInput>,
    joystick: Res<JoystickState>,
    mut cam_state: ResMut<TouchCameraState>,
    mut orbit_query: Query<&mut OrbitCamera>,
    touches: Res<Touches>,
    windows: Query<&Window>,
) {
    // Don't orbit while pinching (2+ fingers)
    let active_count = touches.iter().count();
    if active_count >= 2 {
        cam_state.touch_id = None;
        return;
    }

    // Compute joystick exclusion zone (same math as joystick input)
    let joystick_zone = windows.single().ok().map(|window| {
        let vmin = window.width().min(window.height());
        let joystick_size = vmin * 0.25;
        let center = Vec2::new(
            window.width() * 0.15 + joystick_size * 0.5,
            window.height() - (window.height() * 0.05 + joystick_size * 0.5),
        );
        (center, joystick_size * 1.2)
    });

    for event in touch_events.read() {
        match event.phase {
            TouchPhase::Started => {
                // Reject if this is the joystick finger OR if touch lands in joystick zone
                let in_joystick_zone = joystick_zone
                    .map(|(center, radius)| event.position.distance(center) < radius)
                    .unwrap_or(false);
                if joystick.touch_id != Some(event.id)
                    && !in_joystick_zone
                    && cam_state.touch_id.is_none()
                {
                    cam_state.touch_id = Some(event.id);
                    cam_state.last_position = event.position;

                    for mut orbit in &mut orbit_query {
                        orbit.auto_follow = false;
                        orbit.return_timer = 0.0;
                    }
                }
            }
            TouchPhase::Moved => {
                if cam_state.touch_id == Some(event.id) {
                    let delta = event.position - cam_state.last_position;
                    cam_state.last_position = event.position;

                    let sensitivity = 0.005;
                    for mut orbit in &mut orbit_query {
                        orbit.yaw -= delta.x * sensitivity;
                        orbit.pitch += delta.y * sensitivity;
                        orbit.pitch = orbit.pitch.clamp(-1.2, 1.2);
                    }
                }
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                if cam_state.touch_id == Some(event.id) {
                    cam_state.touch_id = None;
                    // Re-enable auto-follow immediately on touch release
                    for mut orbit in &mut orbit_query {
                        orbit.auto_follow = true;
                    }
                }
            }
        }
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
