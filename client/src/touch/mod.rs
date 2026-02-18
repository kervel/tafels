pub mod camera;
pub mod joystick;
pub mod keyboard;
pub mod shoot;

use bevy::prelude::*;
use bevy::input::touch::TouchInput;
use crate::game::GameState;

pub struct TouchPlugin;

impl Plugin for TouchPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TouchDevice { detected: false })
            .add_systems(Startup, detect_touch_device)
            .add_systems(Update, runtime_touch_detection)
            .add_plugins(joystick::JoystickPlugin)
            .add_plugins(camera::TouchCameraPlugin)
            .add_plugins(shoot::ShootPlugin)
            .add_plugins(keyboard::SoftKeyboardPlugin);
    }
}

#[derive(Resource)]
pub struct TouchDevice {
    pub detected: bool,
}

#[cfg(target_arch = "wasm32")]
fn detect_touch_device(mut touch: ResMut<TouchDevice>) {
    if let Some(window) = web_sys::window() {
        let navigator = window.navigator();
        if navigator.max_touch_points() > 0 {
            touch.detected = true;
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn detect_touch_device() {}

fn runtime_touch_detection(
    mut touch_events: MessageReader<TouchInput>,
    mut touch_device: ResMut<TouchDevice>,
) {
    if !touch_device.detected && touch_events.read().next().is_some() {
        touch_device.detected = true;
    }
    touch_events.clear();
}

/// Run condition: only run when touch device detected and playing.
pub fn touch_playing(touch: Res<TouchDevice>, state: Res<State<GameState>>) -> bool {
    touch.detected && *state.get() == GameState::Playing
}
