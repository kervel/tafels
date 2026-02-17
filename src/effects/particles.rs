use bevy::prelude::*;
use rand::prelude::*;

use crate::projectile::Projectile;

const MAX_PARTICLES: usize = 500;

/// A single particle with velocity, lifetime, and fade.
#[derive(Component)]
pub struct Particle {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
}

/// Message to request a burst of particles at a position.
#[derive(Message)]
pub struct ParticleBurstEvent {
    pub position: Vec3,
    pub color: Color,
    pub count: u32,
    pub speed: f32,
    pub lifetime: f32,
    pub size: f32,
}

/// Spawn particles from burst events.
pub fn handle_burst_events(
    mut commands: Commands,
    mut events: MessageReader<ParticleBurstEvent>,
    existing: Query<&Particle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let current_count = existing.iter().count();
    if current_count >= MAX_PARTICLES {
        events.clear();
        return;
    }

    let mut budget = MAX_PARTICLES - current_count;

    for event in events.read() {
        let count = (event.count as usize).min(budget);
        if count == 0 {
            break;
        }
        budget -= count;

        let mesh = meshes.add(Sphere::new(event.size));
        let c = event.color.to_linear();
        let material = materials.add(StandardMaterial {
            base_color: event.color,
            emissive: bevy::color::LinearRgba::new(
                c.red * 20.0,
                c.green * 20.0,
                c.blue * 20.0,
                1.0,
            ),
            ..default()
        });

        let mut rng = rand::thread_rng();

        for _ in 0..count {
            let dir = Vec3::new(
                rng.r#gen_range(-1.0..1.0_f32),
                rng.r#gen_range(0.2..1.0_f32),
                rng.r#gen_range(-1.0..1.0_f32),
            )
            .normalize_or_zero();

            let speed = event.speed * rng.r#gen_range(0.5..1.5_f32);

            commands.spawn((
                Particle {
                    velocity: dir * speed,
                    lifetime: event.lifetime * rng.r#gen_range(0.6..1.0_f32),
                    max_lifetime: event.lifetime,
                },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(event.position),
            ));
        }
    }
}

/// Update particle positions, apply gravity, fade, and despawn.
pub fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Particle, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut particle, mut transform) in &mut query {
        particle.velocity.y -= 6.0 * dt; // lighter gravity than ball
        transform.translation += particle.velocity * dt;

        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Shrink as lifetime runs out
        let frac = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);
        let scale = frac * frac; // quadratic fade
        transform.scale = Vec3::splat(scale.max(0.01));
    }
}

/// Spawn trail particles behind all balls (flying and rolling).
pub fn ball_trail_particles(
    mut commands: Commands,
    balls: Query<(&Transform, &crate::projectile::Velocity), With<Projectile>>,
    existing: Query<&Particle>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if existing.iter().count() >= MAX_PARTICLES {
        return;
    }

    let mesh = meshes.add(Sphere::new(0.12));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.6, 0.2),
        emissive: bevy::color::LinearRgba::new(30.0, 15.0, 3.0, 1.0),
        ..default()
    });

    let mut rng = rand::thread_rng();

    for (ball_tf, vel) in &balls {
        let speed = vel.0.length();
        // Higher speed = more particles. Always at least 50% chance.
        let spawn_chance = (speed / 20.0).clamp(0.5, 1.0);
        if rng.r#gen::<f32>() > spawn_chance {
            continue;
        }

        let offset = Vec3::new(
            rng.r#gen_range(-0.15..0.15_f32),
            rng.r#gen_range(-0.05..0.2_f32),
            rng.r#gen_range(-0.15..0.15_f32),
        );

        commands.spawn((
            Particle {
                velocity: Vec3::new(
                    rng.r#gen_range(-0.5..0.5_f32),
                    rng.r#gen_range(0.5..2.0_f32),
                    rng.r#gen_range(-0.5..0.5_f32),
                ),
                lifetime: 0.8,
                max_lifetime: 0.8,
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(ball_tf.translation + offset),
        ));
    }
}
