use bevy::prelude::*;
use rand::prelude::*;

use super::{GameSession, GameState};
use crate::character::CharacterMarker;
use crate::effects::particles::ParticleBurstEvent;
use crate::terrain::TerrainResource;
use crate::terrain::heightmap::sample_height;

pub struct CoinPlugin;

impl Plugin for CoinPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                tick_pending_spawns,
                animate_coins,
                check_coin_pickup,
                despawn_expired,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), cleanup_coins);
    }
}

/// Spawned when a correct answer is given; counts down before spawning a collectible coin.
#[derive(Component)]
pub struct PendingCoinSpawn {
    pub position: Vec3,
    pub value: i32,
    pub timer: f32,
}

/// A physical coin in the world that the player can walk to and collect.
#[derive(Component)]
pub struct CollectibleCoin {
    pub value: i32,
    pub elapsed: f32,
    pub despawn_after: f32,
}

/// Animation state for coin bob and spin.
#[derive(Component)]
pub struct CoinAnimation {
    pub base_y: f32,
    pub phase: f32,
}

fn tick_pending_spawns(
    mut commands: Commands,
    time: Res<Time>,
    mut pending: Query<(Entity, &mut PendingCoinSpawn)>,
    terrain: Option<Res<TerrainResource>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, mut spawn) in &mut pending {
        spawn.timer -= dt;
        if spawn.timer <= 0.0 {
            commands.entity(entity).despawn();

            // Determine ground height at coin position
            let ground_y = if let Some(ref terrain) = terrain {
                sample_height(&terrain.heightmap, spawn.position.x, spawn.position.z)
            } else {
                0.0
            };
            let coin_y = ground_y + 1.0;

            let mut rng = rand::thread_rng();
            let phase = rng.r#gen_range(0.0..std::f32::consts::TAU);

            let mesh = meshes.add(Torus::new(0.25, 0.4));
            let material = materials.add(StandardMaterial {
                base_color: Color::srgb(1.0, 0.85, 0.1),
                emissive: bevy::color::LinearRgba::new(30.0, 22.0, 0.0, 1.0),
                metallic: 1.0,
                perceptual_roughness: 0.3,
                ..default()
            });

            // Spawn coin entity with point light child
            commands
                .spawn((
                    CollectibleCoin {
                        value: spawn.value,
                        elapsed: 0.0,
                        despawn_after: 15.0,
                    },
                    CoinAnimation {
                        base_y: coin_y,
                        phase,
                    },
                    Mesh3d(mesh),
                    MeshMaterial3d(material),
                    Transform::from_translation(Vec3::new(
                        spawn.position.x,
                        coin_y,
                        spawn.position.z,
                    ))
                    // Rotate torus upright (it spawns flat by default)
                    .with_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        PointLight {
                            color: Color::srgb(1.0, 0.85, 0.2),
                            intensity: 400_000.0,
                            range: 8.0,
                            shadows_enabled: false,
                            ..default()
                        },
                        Transform::IDENTITY,
                    ));
                });
        }
    }
}

fn animate_coins(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &CoinAnimation, &CollectibleCoin)>,
) {
    let t = time.elapsed_secs();

    for (mut transform, anim, coin) in &mut query {
        // Y-axis rotation (spin)
        let spin = Quat::from_rotation_y(t * 2.0 + anim.phase);
        // Keep the torus upright while spinning
        transform.rotation = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2) * spin;

        // Sine bob
        let bob = (t * 2.0 + anim.phase).sin() * 0.3;
        transform.translation.y = anim.base_y + bob;

        // Shrink in last 3 seconds
        let remaining = coin.despawn_after - coin.elapsed;
        if remaining < 3.0 {
            let scale = (remaining / 3.0).clamp(0.0, 1.0);
            transform.scale = Vec3::splat(scale);
        }
    }
}

fn check_coin_pickup(
    mut commands: Commands,
    mut session: ResMut<GameSession>,
    coins: Query<(Entity, &Transform, &CollectibleCoin)>,
    character: Query<&Transform, (With<CharacterMarker>, Without<CollectibleCoin>)>,
    mut burst_events: MessageWriter<ParticleBurstEvent>,
) {
    let Ok(char_tf) = character.single() else {
        return;
    };

    for (entity, coin_tf, coin) in &coins {
        let dx = char_tf.translation.x - coin_tf.translation.x;
        let dz = char_tf.translation.z - coin_tf.translation.z;
        let dist_xz = (dx * dx + dz * dz).sqrt();

        if dist_xz < 2.0 {
            session.coins += coin.value;

            // Gold pickup burst
            burst_events.write(ParticleBurstEvent {
                position: coin_tf.translation,
                color: Color::srgb(1.0, 0.85, 0.0),
                count: 30,
                speed: 6.0,
                lifetime: 1.0,
                size: 0.15,
            });

            commands.entity(entity).despawn();
        }
    }
}

fn despawn_expired(
    mut commands: Commands,
    time: Res<Time>,
    mut coins: Query<(Entity, &mut CollectibleCoin)>,
) {
    let dt = time.delta_secs();

    for (entity, mut coin) in &mut coins {
        coin.elapsed += dt;
        if coin.elapsed >= coin.despawn_after {
            commands.entity(entity).despawn();
        }
    }
}

fn cleanup_coins(
    mut commands: Commands,
    coins: Query<Entity, With<CollectibleCoin>>,
    pending: Query<Entity, With<PendingCoinSpawn>>,
) {
    for entity in &coins {
        commands.entity(entity).despawn();
    }
    for entity in &pending {
        commands.entity(entity).despawn();
    }
}
