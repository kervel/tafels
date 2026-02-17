use bevy::prelude::*;

use super::{BounceCount, Projectile, ProjectileLifetime, Velocity};
use crate::terrain::TerrainResource;
use crate::terrain::heightmap;

const GRAVITY: f32 = 9.81;
const RESTITUTION: f32 = 0.7;

pub fn apply_gravity(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Projectile>>,
) {
    let dt = time.delta_secs();

    for (mut transform, mut vel) in &mut query {
        vel.0.y -= GRAVITY * dt;
        transform.translation += vel.0 * dt;
    }
}

pub fn bounce_on_terrain(
    mut commands: Commands,
    terrain: Option<Res<TerrainResource>>,
    mut query: Query<
        (Entity, &mut Transform, &mut Velocity, &mut BounceCount),
        With<Projectile>,
    >,
) {
    let Some(terrain) = terrain else {
        return;
    };

    for (entity, mut transform, mut vel, mut bounces) in &mut query {
        let pos = transform.translation;
        let terrain_y = heightmap::sample_height(&terrain.heightmap, pos.x, pos.z);

        if pos.y <= terrain_y + 0.25 {
            // Place on terrain surface
            transform.translation.y = terrain_y + 0.25;

            // Compute terrain normal and reflect velocity against it
            let normal = heightmap::sample_normal(&terrain.heightmap, pos.x, pos.z);

            // Reflect: v' = v - 2(v . n) * n
            let dot = vel.0.dot(normal);
            if dot < 0.0 {
                vel.0 = (vel.0 - 2.0 * dot * normal) * RESTITUTION;
            }

            bounces.count += 1;

            if bounces.count >= bounces.max_bounces || vel.0.length() < 0.5 {
                commands.entity(entity).despawn();
            }
        }
    }
}

pub fn tick_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ProjectileLifetime), With<Projectile>>,
) {
    for (entity, mut lifetime) in &mut query {
        lifetime.remaining -= time.delta_secs();
        if lifetime.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}
