use rand::prelude::*;
use tafels_shared::exercise::generate_exercise;
use tafels_shared::heightmap::{generate_heightmap, sample_height};
use tafels_shared::protocol::{BeaconInfo, ServerMessage, encode};

use crate::state::AppState;

/// Maximum concurrent beacons in the world.
const MAX_BEACONS: usize = 5;
/// Seconds between beacon spawn attempts.
const SPAWN_INTERVAL: f32 = 8.0;
/// Beacon lifetime in seconds.
const BEACON_LIFETIME: f32 = 45.0;
/// Minimum distance between beacons.
const MIN_BEACON_SEPARATION: f32 = 20.0;

pub struct BeaconManager {
    heightmap: tafels_shared::heightmap::HeightmapData,
    spawn_cooldown: f32,
}

impl BeaconManager {
    pub fn new() -> Self {
        // Same parameters as the client terrain (see client/src/terrain/mod.rs)
        Self {
            heightmap: generate_heightmap(256, 256, 500.0, 30.0),
            spawn_cooldown: 5.0, // Initial delay before first beacon
        }
    }

    /// Called every tick (50ms). Manages beacon spawning and expiry.
    pub async fn tick(&mut self, dt: f32, state: &AppState) {
        // Expire old beacons
        let mut expired_ids = Vec::new();
        {
            let mut world = state.world.lock().await;
            world.beacons.retain(|id, beacon| {
                // Decrease lifetime stored in the beacon's lifetime field
                if beacon.lifetime <= dt {
                    expired_ids.push(*id);
                    false
                } else {
                    true
                }
            });
            // Update remaining lifetimes
            for beacon in world.beacons.values_mut() {
                beacon.lifetime -= dt;
            }
        }

        // Broadcast expired beacons
        for beacon_id in expired_ids {
            let msg = ServerMessage::BeaconExpired { beacon_id };
            let _ = state.broadcast_tx.send(encode(&msg));
        }

        // Try to spawn new beacons
        self.spawn_cooldown -= dt;
        if self.spawn_cooldown > 0.0 {
            return;
        }

        // Collect data under lock, then release before doing RNG
        let (_player_count, _beacon_count, target_x, target_z, existing_positions, next_id) = {
            let world = state.world.lock().await;
            if world.players.is_empty() {
                return;
            }
            if world.beacons.len() >= MAX_BEACONS {
                self.spawn_cooldown = 2.0;
                return;
            }

            let players: Vec<_> = world.players.values().collect();
            let mut rng = rand::thread_rng();
            let target = players[rng.gen_range(0..players.len())];
            let existing: Vec<(f32, f32)> = world.beacons.values().map(|b| (b.x, b.z)).collect();

            (
                players.len(),
                world.beacons.len(),
                target.x,
                target.z,
                existing,
                world.next_beacon_id,
            )
        };

        // Build beacon data synchronously (no await) so thread_rng is safe
        let beacon = {
            let mut rng = rand::thread_rng();

            // Try to find a valid spawn position near the target player
            let mut spawn_pos = None;
            for _ in 0..20 {
                let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                let dist = rng.gen_range(30.0..60.0_f32);
                let cx = target_x + angle.cos() * dist;
                let cz = target_z + angle.sin() * dist;

                // Check minimum distance from player
                let dx = cx - target_x;
                let dz = cz - target_z;
                if (dx * dx + dz * dz).sqrt() < 15.0 {
                    continue;
                }

                // Check separation from existing beacons
                let mut too_close = false;
                for &(bx, bz) in &existing_positions {
                    let ex = cx - bx;
                    let ez = cz - bz;
                    if (ex * ex + ez * ez).sqrt() < MIN_BEACON_SEPARATION {
                        too_close = true;
                        break;
                    }
                }
                if too_close {
                    continue;
                }

                spawn_pos = Some((cx, cz));
                break;
            }

            let Some((x, z)) = spawn_pos else {
                self.spawn_cooldown = 2.0;
                return;
            };

            let y = sample_height(&self.heightmap, x, z);

            // Generate exercise
            let difficulty = tafels_shared::difficulty::Difficulty::Easy;
            let exercise = generate_exercise(&difficulty);
            let question_text = match exercise.operation {
                tafels_shared::exercise::Operation::Multiply => {
                    format!("{} x {} = ?", exercise.operand_a, exercise.operand_b)
                }
                tafels_shared::exercise::Operation::Divide => {
                    format!("{} / {} = ?", exercise.operand_a, exercise.operand_b)
                }
            };

            BeaconInfo {
                beacon_id: next_id,
                x,
                y,
                z,
                question_text,
                choices: exercise.choices,
                correct_index: exercise
                    .choices
                    .iter()
                    .position(|&c| c == exercise.correct_answer)
                    .unwrap_or(0) as u8,
                lifetime: BEACON_LIFETIME,
            }
        }; // rng dropped here, before any await

        // Insert beacon into world state
        {
            let mut world = state.world.lock().await;
            world.beacons.insert(next_id, beacon.clone());
            world.next_beacon_id += 1;
        }

        // Broadcast to clients
        let msg = ServerMessage::BeaconSpawned(beacon);
        let _ = state.broadcast_tx.send(encode(&msg));

        self.spawn_cooldown = SPAWN_INTERVAL;
    }
}
