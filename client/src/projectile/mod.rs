pub mod collision;
pub mod physics;

use bevy::prelude::*;

use crate::camera::orbit::OrbitCamera;
use crate::character::CharacterMarker;
use crate::game::GameState;
use crate::network::{ConnectionStatus, RemoteBallShotEvent, WsConnection};
use tafels_shared::protocol::{ClientMessage, encode};

pub struct ProjectilePlugin;

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_ball, spawn_remote_ball).run_if(in_state(GameState::Playing)),
        )
        .add_systems(
            FixedUpdate,
            (
                physics::apply_gravity,
                physics::bounce_on_terrain,
                physics::tick_lifetime,
                collision::check_ball_panel_collision,
                collision::check_ball_beacon_collision,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component)]
pub struct Projectile;

/// Marker for balls shot by remote players (visual only, no game collision).
#[derive(Component)]
pub struct RemoteProjectile;

#[derive(Component)]
pub struct Velocity(pub Vec3);

#[derive(Component)]
pub struct BounceCount {
    pub count: u8,
    pub max_bounces: u8,
}

#[derive(Component)]
pub struct ProjectileLifetime {
    pub remaining: f32,
}

fn spawn_ball(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    existing_balls: Query<&Projectile>,
    character: Query<&Transform, With<CharacterMarker>>,
    camera: Query<(&Transform, &OrbitCamera), Without<CharacterMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ws: Option<Res<WsConnection>>,
    connection_status: Res<ConnectionStatus>,
) {
    if !keyboard.just_pressed(KeyCode::Space) {
        return;
    }

    // Allow up to 3 balls at a time
    if existing_balls.iter().count() >= 3 {
        return;
    }

    let Ok(char_tf) = character.single() else {
        return;
    };

    let Ok((cam_tf, _orbit)) = camera.single() else {
        return;
    };

    // Throw direction: from character toward where camera is looking
    let throw_dir = (cam_tf.forward().as_vec3()).normalize_or_zero();
    let throw_speed = 25.0;

    // Spawn position: slightly above character, slightly forward
    let spawn_pos = char_tf.translation + Vec3::new(0.0, 1.8, 0.0) + throw_dir * 0.5;

    // Strong upward arc so the ball bounces
    let velocity = throw_dir * throw_speed + Vec3::new(0.0, 8.0, 0.0);

    // Toy ball: bright red with slight emissive glow
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.15, 0.15),
        emissive: bevy::color::LinearRgba::new(20.0, 3.0, 3.0, 1.0),
        ..default()
    });

    commands
        .spawn((
            Mesh3d(meshes.add(Sphere::new(0.25))),
            MeshMaterial3d(material),
            Transform::from_translation(spawn_pos),
            Projectile,
            Velocity(velocity),
            BounceCount {
                count: 0,
                max_bounces: 8,
            },
            ProjectileLifetime { remaining: 10.0 },
        ))
        .with_child((
            // Point light that moves with the ball - lights up ground/trees nearby
            PointLight {
                color: Color::srgb(1.0, 0.4, 0.2),
                intensity: 800_000.0,
                range: 25.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::IDENTITY,
        ));

    // Send to server so other players can see the ball
    if connection_status.is_online() {
        if let Some(ws) = ws {
            let msg = ClientMessage::ShootBall {
                x: spawn_pos.x,
                y: spawn_pos.y,
                z: spawn_pos.z,
                vx: velocity.x,
                vy: velocity.y,
                vz: velocity.z,
            };
            ws.send_binary(encode(&msg));
        }
    }
}

/// Spawn visual-only balls for remote players.
fn spawn_remote_ball(
    mut commands: Commands,
    mut events: MessageReader<RemoteBallShotEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in events.read() {
        let material = materials.add(StandardMaterial {
            base_color: Color::srgb(0.2, 0.4, 0.9),
            emissive: bevy::color::LinearRgba::new(3.0, 5.0, 20.0, 1.0),
            ..default()
        });

        commands
            .spawn((
                Mesh3d(meshes.add(Sphere::new(0.25))),
                MeshMaterial3d(material),
                Transform::from_translation(event.position),
                Projectile,
                RemoteProjectile,
                Velocity(event.velocity),
                BounceCount {
                    count: 0,
                    max_bounces: 8,
                },
                ProjectileLifetime { remaining: 10.0 },
            ))
            .with_child((
                PointLight {
                    color: Color::srgb(0.3, 0.5, 1.0),
                    intensity: 800_000.0,
                    range: 25.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::IDENTITY,
            ));
    }
}
