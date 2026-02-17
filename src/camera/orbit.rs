use bevy::prelude::*;

use crate::character::{CharacterMarker, CharacterState, MovementInput};
use crate::terrain::TerrainResource;
use crate::terrain::heightmap;

#[derive(Component)]
pub struct OrbitCamera {
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub target: Entity,
    /// When true, camera auto-follows behind the character's movement direction
    pub auto_follow: bool,
    /// Timer counting up after mouse release; triggers return to auto-follow
    pub return_timer: f32,
    /// Last known movement direction yaw (only updated when character is walking)
    pub last_move_yaw: f32,
    /// When true, snap camera behind character on next frame (no lerp)
    pub needs_snap: bool,
}

/// Reset camera to snap behind character when entering Playing state.
pub fn reset_camera_on_play(mut camera: Query<&mut OrbitCamera>) {
    for mut orbit in &mut camera {
        orbit.yaw = std::f32::consts::PI;
        orbit.last_move_yaw = 0.0;
        orbit.needs_snap = true;
        orbit.auto_follow = true;
    }
}

pub fn camera_mouse_input(
    mut mouse_motion: MessageReader<bevy::input::mouse::MouseMotion>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut query: Query<&mut OrbitCamera>,
) {
    let dragging = mouse_button.pressed(MouseButton::Left);

    if !dragging {
        mouse_motion.clear();
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion.read() {
        delta += event.delta;
    }

    let sensitivity = 0.003;

    for mut orbit in &mut query {
        if dragging {
            orbit.auto_follow = false;
            orbit.return_timer = 0.0;
            orbit.yaw -= delta.x * sensitivity;
            orbit.pitch -= delta.y * sensitivity;
            orbit.pitch = orbit.pitch.clamp(-1.2, 1.2);
        } else if !orbit.auto_follow {
            orbit.return_timer += 0.016;
            if orbit.return_timer > 1.0 {
                orbit.auto_follow = true;
            }
        }
    }
}

pub fn camera_scroll_zoom(
    mut scroll: MessageReader<bevy::input::mouse::MouseWheel>,
    mut query: Query<&mut OrbitCamera>,
) {
    let mut scroll_amount = 0.0;
    for event in scroll.read() {
        scroll_amount += event.y;
    }
    if scroll_amount.abs() < 0.001 {
        return;
    }

    for mut orbit in &mut query {
        orbit.distance -= scroll_amount * 3.0;
        orbit.distance = orbit.distance.clamp(8.0, 80.0);
    }
}

pub fn camera_follow(
    terrain: Option<Res<TerrainResource>>,
    character_query: Query<
        (Entity, &Transform, &CharacterState, &MovementInput),
        With<CharacterMarker>,
    >,
    mut camera_query: Query<(&mut Transform, &mut OrbitCamera), Without<CharacterMarker>>,
    time: Res<Time>,
) {
    let Some((char_entity, char_transform, char_state, move_input)) =
        character_query.iter().next()
    else {
        return;
    };

    for (mut cam_transform, mut orbit) in &mut camera_query {
        if orbit.target == Entity::PLACEHOLDER {
            orbit.target = char_entity;
            orbit.needs_snap = true;
        }

        // Update the last known movement yaw when character is walking FORWARD.
        // Skip backwards/strafe-only movement so the camera doesn't swing around
        // when the player backs up toward an exercise beacon.
        if *char_state == CharacterState::Walking
            && move_input.direction.length_squared() > 0.01
            && move_input.direction.y > -0.1
        {
            // Compute the world-space movement direction from input + camera yaw
            let forward = Vec3::new(-orbit.yaw.sin(), 0.0, -orbit.yaw.cos());
            let right = Vec3::new(orbit.yaw.cos(), 0.0, -orbit.yaw.sin());
            let move_dir =
                forward * move_input.direction.y + right * move_input.direction.x;
            if move_dir.length_squared() > 0.01 {
                orbit.last_move_yaw = move_dir.x.atan2(move_dir.z);
            }
        }

        // Auto-follow: smoothly lerp camera yaw to behind the character's movement
        if orbit.auto_follow && *char_state == CharacterState::Walking {
            let target_yaw = orbit.last_move_yaw + std::f32::consts::PI;

            let mut diff = target_yaw - orbit.yaw;
            while diff > std::f32::consts::PI {
                diff -= std::f32::consts::TAU;
            }
            while diff < -std::f32::consts::PI {
                diff += std::f32::consts::TAU;
            }
            orbit.yaw += diff * 2.0 * time.delta_secs();
        }

        let target_pos = char_transform.translation + Vec3::new(0.0, 1.5, 0.0);

        let x = orbit.distance * orbit.pitch.cos() * orbit.yaw.sin();
        let y = orbit.distance * orbit.pitch.sin();
        let z = orbit.distance * orbit.pitch.cos() * orbit.yaw.cos();

        let desired_pos = target_pos + Vec3::new(x, y, z);

        let mut final_pos = desired_pos;
        if let Some(ref terrain) = terrain {
            let terrain_y = heightmap::sample_height(
                &terrain.heightmap,
                final_pos.x,
                final_pos.z,
            );
            final_pos.y = final_pos.y.max(terrain_y + 2.0);
        }

        // Snap when flagged, smooth lerp otherwise
        if orbit.needs_snap {
            cam_transform.translation = final_pos;
            orbit.needs_snap = false;
        } else {
            let lerp_speed = 8.0 * time.delta_secs();
            cam_transform.translation =
                cam_transform.translation.lerp(final_pos, lerp_speed.min(1.0));
        }
        *cam_transform = cam_transform.looking_at(target_pos, Vec3::Y);
    }
}
