use bevy::prelude::*;
use rand::prelude::*;

use crate::projectile::Projectile;

const MAX_PARTICLES: usize = 500;

/// A temporary point light that fades out and despawns.
#[derive(Component)]
pub struct BurstLight {
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub max_intensity: f32,
}

/// Different particle movement behaviors.
#[derive(Clone, Copy, Default)]
pub enum ParticleStyle {
    /// Classic outward burst with gravity (existing behavior).
    #[default]
    Burst,
    /// Tiny sparkles floating upward with gentle wobble (beacon appear).
    RisingSparkle,
    /// Embers drifting downward, shrinking slowly (beacon vanish).
    FallingEmber,
    /// Particles expanding outward in a horizontal ring (timeout).
    Ring,
}

/// A single particle with velocity, lifetime, and fade.
#[derive(Component)]
pub struct Particle {
    pub velocity: Vec3,
    pub lifetime: f32,
    pub max_lifetime: f32,
    pub style: ParticleStyle,
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

/// Message for styled particle effects (diverse visuals).
#[derive(Message)]
pub struct StyledParticleEvent {
    pub position: Vec3,
    pub color: Color,
    pub count: u32,
    pub speed: f32,
    pub lifetime: f32,
    pub size: f32,
    pub style: ParticleStyle,
}

/// Spawn particles from burst events (legacy, default Burst style).
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

        // Spawn a temporary point light at the burst position
        let light_intensity = 2_000_000.0;
        commands.spawn((
            BurstLight {
                lifetime: event.lifetime,
                max_lifetime: event.lifetime,
                max_intensity: light_intensity,
            },
            PointLight {
                color: event.color,
                intensity: light_intensity,
                range: 30.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(event.position),
        ));

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
                    style: ParticleStyle::Burst,
                },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(event.position),
            ));
        }
    }
}

/// Spawn particles from styled events (diverse behaviors).
pub fn handle_styled_events(
    mut commands: Commands,
    mut events: MessageReader<StyledParticleEvent>,
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

        let c = event.color.to_linear();
        let emissive_mult = match event.style {
            ParticleStyle::RisingSparkle => 35.0, // extra bright sparkles
            ParticleStyle::FallingEmber => 12.0,  // dimmer embers
            ParticleStyle::Ring => 25.0,
            ParticleStyle::Burst => 20.0,
        };

        let mesh = meshes.add(Sphere::new(event.size));
        let material = materials.add(StandardMaterial {
            base_color: event.color,
            emissive: bevy::color::LinearRgba::new(
                c.red * emissive_mult,
                c.green * emissive_mult,
                c.blue * emissive_mult,
                1.0,
            ),
            ..default()
        });

        // Light for burst/ring styles, skip for subtle sparkle/ember
        match event.style {
            ParticleStyle::Burst | ParticleStyle::Ring => {
                commands.spawn((
                    BurstLight {
                        lifetime: event.lifetime,
                        max_lifetime: event.lifetime,
                        max_intensity: 1_500_000.0,
                    },
                    PointLight {
                        color: event.color,
                        intensity: 1_500_000.0,
                        range: 25.0,
                        shadows_enabled: false,
                        ..default()
                    },
                    Transform::from_translation(event.position),
                ));
            }
            _ => {}
        }

        let mut rng = rand::thread_rng();

        for _ in 0..count {
            let (vel, lt) = match event.style {
                ParticleStyle::Burst => {
                    let dir = Vec3::new(
                        rng.r#gen_range(-1.0..1.0_f32),
                        rng.r#gen_range(0.2..1.0_f32),
                        rng.r#gen_range(-1.0..1.0_f32),
                    )
                    .normalize_or_zero();
                    (
                        dir * event.speed * rng.r#gen_range(0.5..1.5_f32),
                        event.lifetime * rng.r#gen_range(0.6..1.0_f32),
                    )
                }
                ParticleStyle::RisingSparkle => {
                    // Gentle upward drift with horizontal wobble
                    let vel = Vec3::new(
                        rng.r#gen_range(-0.8..0.8_f32),
                        event.speed * rng.r#gen_range(0.6..1.2_f32),
                        rng.r#gen_range(-0.8..0.8_f32),
                    );
                    (vel, event.lifetime * rng.r#gen_range(0.5..1.0_f32))
                }
                ParticleStyle::FallingEmber => {
                    // Drift outward and downward like dissolving embers
                    let angle = rng.r#gen_range(0.0..std::f32::consts::TAU);
                    let outward = rng.r#gen_range(0.5..2.0_f32);
                    let vel = Vec3::new(
                        angle.cos() * outward,
                        rng.r#gen_range(-1.5..-0.3_f32),
                        angle.sin() * outward,
                    );
                    (vel, event.lifetime * rng.r#gen_range(0.7..1.0_f32))
                }
                ParticleStyle::Ring => {
                    // Expand outward in a flat horizontal ring
                    let angle = rng.r#gen_range(0.0..std::f32::consts::TAU);
                    let ring_speed = event.speed * rng.r#gen_range(0.8..1.2_f32);
                    let vel = Vec3::new(
                        angle.cos() * ring_speed,
                        rng.r#gen_range(-0.3..0.3_f32), // nearly flat
                        angle.sin() * ring_speed,
                    );
                    (vel, event.lifetime * rng.r#gen_range(0.6..1.0_f32))
                }
            };

            // Spread spawn position slightly for sparkle/ember
            let offset = match event.style {
                ParticleStyle::RisingSparkle => Vec3::new(
                    rng.r#gen_range(-1.5..1.5_f32),
                    rng.r#gen_range(-0.5..1.0_f32),
                    rng.r#gen_range(-1.5..1.5_f32),
                ),
                ParticleStyle::FallingEmber => Vec3::new(
                    rng.r#gen_range(-0.8..0.8_f32),
                    rng.r#gen_range(0.0..3.0_f32),
                    rng.r#gen_range(-0.8..0.8_f32),
                ),
                _ => Vec3::ZERO,
            };

            commands.spawn((
                Particle {
                    velocity: vel,
                    lifetime: lt,
                    max_lifetime: event.lifetime,
                    style: event.style,
                },
                Mesh3d(mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(event.position + offset),
            ));
        }
    }
}

/// Update particle positions, apply style-specific physics, fade, and despawn.
pub fn update_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Particle, &mut Transform)>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for (entity, mut particle, mut transform) in &mut query {
        match particle.style {
            ParticleStyle::Burst => {
                particle.velocity.y -= 6.0 * dt;
            }
            ParticleStyle::RisingSparkle => {
                // No gravity, gentle horizontal wobble
                let wobble = (t * 3.0 + transform.translation.x * 2.0).sin() * 0.5;
                particle.velocity.x += wobble * dt;
                particle.velocity.z += (t * 2.5 + transform.translation.z).cos() * 0.5 * dt;
            }
            ParticleStyle::FallingEmber => {
                // Very light gravity, slight drag
                particle.velocity.y -= 1.5 * dt;
                particle.velocity *= 1.0 - 0.5 * dt; // drag
            }
            ParticleStyle::Ring => {
                // Light upward drift to keep ring visible, slight drag
                particle.velocity.y += 0.3 * dt;
                particle.velocity *= 1.0 - 1.0 * dt; // drag slows expansion
            }
        }

        transform.translation += particle.velocity * dt;
        particle.lifetime -= dt;

        if particle.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }

        let frac = (particle.lifetime / particle.max_lifetime).clamp(0.0, 1.0);

        let scale = match particle.style {
            ParticleStyle::Burst => frac * frac, // quadratic shrink
            ParticleStyle::RisingSparkle => {
                // Twinkle: pulsing scale
                let twinkle = 0.6 + 0.4 * (t * 8.0 + particle.lifetime * 5.0).sin();
                frac * twinkle
            }
            ParticleStyle::FallingEmber => {
                // Slow linear fade
                frac
            }
            ParticleStyle::Ring => {
                // Stay full size, then pop out at end
                if frac > 0.2 { 1.0 } else { frac / 0.2 }
            }
        };

        transform.scale = Vec3::splat(scale.max(0.01));
    }
}

/// Fade out and despawn burst lights.
pub fn update_burst_lights(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BurstLight, &mut PointLight)>,
) {
    let dt = time.delta_secs();
    for (entity, mut burst, mut light) in &mut query {
        burst.lifetime -= dt;
        if burst.lifetime <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        let frac = (burst.lifetime / burst.max_lifetime).clamp(0.0, 1.0);
        light.intensity = burst.max_intensity * frac;
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
                style: ParticleStyle::Burst,
            },
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(ball_tf.translation + offset),
        ));
    }
}
