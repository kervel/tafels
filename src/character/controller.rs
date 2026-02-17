use bevy::prelude::*;

use super::{CharacterController, CharacterState, MovementInput};
use crate::terrain::TerrainResource;
use crate::terrain::heightmap;

pub fn read_movement_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut MovementInput, &mut CharacterController)>,
) {
    for (mut input, mut controller) in &mut query {
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

        // Shift to run (1.5x speed)
        controller.speed = if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            12.0
        } else {
            8.0
        };
    }
}

pub fn apply_movement(
    time: Res<Time>,
    terrain: Option<Res<TerrainResource>>,
    camera_query: Query<&crate::camera::orbit::OrbitCamera>,
    mut query: Query<(&mut Transform, &MovementInput, &CharacterController)>,
) {
    let camera_yaw = camera_query
        .iter()
        .next()
        .map(|c| c.yaw)
        .unwrap_or(0.0);

    for (mut transform, input, controller) in &mut query {
        if input.direction.length_squared() < 0.001 {
            continue;
        }

        // Move relative to camera orientation
        let forward = Vec3::new(-camera_yaw.sin(), 0.0, -camera_yaw.cos());
        let right = Vec3::new(camera_yaw.cos(), 0.0, -camera_yaw.sin());

        let movement = (forward * input.direction.y + right * input.direction.x).normalize()
            * controller.speed
            * time.delta_secs();

        transform.translation += movement;

        // Clamp to terrain boundaries (invisible wall)
        if let Some(ref terrain) = terrain {
            let half = terrain.world_size / 2.0 - 5.0; // 5 unit margin
            transform.translation.x = transform.translation.x.clamp(-half, half);
            transform.translation.z = transform.translation.z.clamp(-half, half);

            // Snap Y to terrain height
            let y = heightmap::sample_height(
                &terrain.heightmap,
                transform.translation.x,
                transform.translation.z,
            );
            transform.translation.y = y;
        }

        // Rotate character to face movement direction
        let look_dir = movement.normalize();
        if look_dir.length_squared() > 0.001 {
            let target_rotation = Quat::from_rotation_y(look_dir.x.atan2(look_dir.z));
            transform.rotation = transform.rotation.slerp(target_rotation, 10.0 * time.delta_secs());
        }
    }
}

pub fn update_character_state(
    mut query: Query<(&MovementInput, &CharacterController, &mut CharacterState)>,
) {
    for (input, controller, mut state) in &mut query {
        let new_state = if input.direction.length_squared() > 0.001 {
            if controller.speed > 10.0 {
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
