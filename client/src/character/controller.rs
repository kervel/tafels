use bevy::prelude::*;

use super::{CharacterController, CharacterState, MovementInput};
use crate::terrain::TerrainResource;
use crate::terrain::heightmap;
use crate::touch::joystick::JoystickState;

pub fn read_movement_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    joystick: Res<JoystickState>,
    mut query: Query<(&mut MovementInput, &mut CharacterController)>,
) {
    for (mut input, mut controller) in &mut query {
        // Joystick takes priority over keyboard
        if joystick.active {
            let delta = joystick.current - joystick.origin;
            let dist = delta.length();

            if dist < 5.0 {
                // Dead zone
                input.direction = Vec2::ZERO;
                controller.running = false;
            } else {
                let normalized = delta / joystick.max_radius;
                let clamped = if normalized.length() > 1.0 {
                    normalized.normalize()
                } else {
                    normalized
                };
                // Apply gentle power curve so small deflections give fine control
                let mag = clamped.length();
                let curved = if mag > 0.001 {
                    clamped * (mag.powf(1.5) / mag)
                } else {
                    Vec2::ZERO
                };
                // Touch coords: X right, Y down. Movement: X right, Y forward.
                // Reduce lateral (turning) speed to 40% — forward feels right
                input.direction = Vec2::new(curved.x * 0.4, -curved.y);
                controller.running = false;
            }
            continue;
        }

        let mut dir = Vec2::ZERO;

        if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
            dir.y += 1.0;
        }
        if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
            dir.y -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
            dir.x -= 1.0;
        }
        if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
            dir.x += 1.0;
        }

        input.direction = if dir.length_squared() > 0.0 {
            dir.normalize()
        } else {
            Vec2::ZERO
        };

        // Shift to run (1.5x base speed)
        controller.running =
            keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);
    }
}

pub fn apply_movement(
    time: Res<Time>,
    terrain: Option<Res<TerrainResource>>,
    camera_query: Query<&crate::camera::orbit::OrbitCamera>,
    mut query: Query<(&mut Transform, &MovementInput, &CharacterController)>,
) {
    let camera_yaw = camera_query.iter().next().map(|c| c.yaw).unwrap_or(0.0);

    for (mut transform, input, controller) in &mut query {
        if input.direction.length_squared() >= 0.001 {
            // Move relative to camera orientation
            let forward = Vec3::new(-camera_yaw.sin(), 0.0, -camera_yaw.cos());
            let right = Vec3::new(camera_yaw.cos(), 0.0, -camera_yaw.sin());

            let raw = forward * input.direction.y + right * input.direction.x;
            let magnitude = raw.length().min(1.0);
            let movement = raw.normalize()
                * magnitude
                * controller.effective_speed()
                * time.delta_secs();

            transform.translation += movement;

            // Rotate character to face movement direction
            let look_dir = movement.normalize();
            if look_dir.length_squared() > 0.001 {
                let target_rotation = Quat::from_rotation_y(look_dir.x.atan2(look_dir.z));
                transform.rotation = transform
                    .rotation
                    .slerp(target_rotation, 4.0 * time.delta_secs());
            }
        }

        // Always snap to terrain (even when idle)
        if let Some(ref terrain) = terrain {
            let half = terrain.world_size / 2.0 - 5.0;
            transform.translation.x = transform.translation.x.clamp(-half, half);
            transform.translation.z = transform.translation.z.clamp(-half, half);

            let y = heightmap::sample_height(
                &terrain.heightmap,
                transform.translation.x,
                transform.translation.z,
            );
            transform.translation.y = y;
        }
    }
}

/// Always snap character Y to terrain height (runs in all states).
pub fn snap_to_terrain(
    terrain: Option<Res<TerrainResource>>,
    mut query: Query<&mut Transform, With<super::CharacterMarker>>,
) {
    let Some(ref terrain) = terrain else {
        return;
    };
    for mut transform in &mut query {
        let y = heightmap::sample_height(
            &terrain.heightmap,
            transform.translation.x,
            transform.translation.z,
        );
        transform.translation.y = y;
    }
}

pub fn update_character_state(
    mut query: Query<(&MovementInput, &CharacterController, &mut CharacterState)>,
) {
    for (input, controller, mut state) in &mut query {
        let new_state = if input.direction.length_squared() > 0.001 {
            if controller.running {
                CharacterState::Running
            } else {
                CharacterState::Walking
            }
        } else {
            CharacterState::Idle
        };

        if *state != new_state {
            *state = new_state;
        }
    }
}
