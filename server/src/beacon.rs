use rand::prelude::*;
use tafels_shared::exercise::generate_exercise;
use tafels_shared::heightmap::{generate_heightmap, sample_height};
use tafels_shared::protocol::{BeaconInfo, ServerMessage, encode};

use crate::state::AppState;

/// Maximum concurrent beacons in the world.
/// Maximum concurrent beacons per player in the world.
const MAX_BEACONS_PER_PLAYER: usize = 10;
/// Seconds between beacon spawn attempts.
const SPAWN_INTERVAL: f32 = 1.0;
/// Beacon lifetime in seconds.
const BEACON_LIFETIME: f32 = 45.0;
use tafels_shared::constants::MIN_BEACON_SEPARATION;

pub struct BeaconManager {
    heightmap: tafels_shared::heightmap::HeightmapData,
    spawn_cooldown: f32,
}

impl BeaconManager {
    pub fn new() -> Self {
        // Same parameters as the client terrain (see client/src/terrain/mod.rs)
        Self {
            heightmap: generate_heightmap(256, 256, 500.0, 60.0),
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
            for &id in &expired_ids {
                world.activated_beacons.remove(&id);
            }
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

        // Collect data under lock: per-player beacon counts, positions, difficulty
        let (players_needing_beacons, existing_positions, difficulty, mut next_id) = {
            let world = state.world.lock().await;
            if world.players.is_empty() {
                return;
            }

            let existing: Vec<(f32, f32)> = world.beacons.values().map(|b| (b.x, b.z)).collect();

            // Count beacons targeted at each player
            let mut beacon_counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
            for beacon in world.beacons.values() {
                *beacon_counts.entry(beacon.target_player_id).or_insert(0) += 1;
            }

            // Find players that need more beacons
            let players: Vec<(u32, f32, f32)> = world
                .players
                .values()
                .filter(|p| beacon_counts.get(&p.player_id).copied().unwrap_or(0) < MAX_BEACONS_PER_PLAYER)
                .map(|p| (p.player_id, p.x, p.z))
                .collect();

            (players, existing, world.difficulty, world.next_beacon_id)
        };

        // Build beacons synchronously (no await) so thread_rng is safe
        let mut new_beacons = Vec::new();
        {
            let mut rng = rand::thread_rng();
            let mut all_positions = existing_positions;

            for (player_id, px, pz) in &players_needing_beacons {
                // Try to find a valid spawn position near this player
                let mut spawn_pos = None;
                for _ in 0..20 {
                    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
                    let dist = rng.gen_range(30.0..60.0_f32);
                    let cx = px + angle.cos() * dist;
                    let cz = pz + angle.sin() * dist;

                    // Check minimum distance from player
                    let dx = cx - px;
                    let dz = cz - pz;
                    if (dx * dx + dz * dz).sqrt() < 15.0 {
                        continue;
                    }

                    // Check separation from existing beacons
                    let mut too_close = false;
                    for &(bx, bz) in &all_positions {
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
                    continue;
                };

                let y = sample_height(&self.heightmap, x, z);

                let exercise = generate_exercise(&difficulty);
                let question_text = match exercise.operation {
                    tafels_shared::exercise::Operation::Multiply => {
                        format!("{} x {} = ?", exercise.operand_a, exercise.operand_b)
                    }
                    tafels_shared::exercise::Operation::Divide => {
                        format!("{} / {} = ?", exercise.operand_a, exercise.operand_b)
                    }
                };

                let beacon = BeaconInfo {
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
                    target_player_id: *player_id,
                };

                all_positions.push((x, z));
                next_id += 1;
                new_beacons.push(beacon);
            }
        } // rng dropped here, before any await

        // Insert beacons into world state and broadcast
        if !new_beacons.is_empty() {
            let mut world = state.world.lock().await;
            for beacon in &new_beacons {
                world.beacons.insert(beacon.beacon_id, beacon.clone());
            }
            world.next_beacon_id = next_id;
            drop(world);

            for beacon in new_beacons {
                let msg = ServerMessage::BeaconSpawned(beacon);
                let _ = state.broadcast_tx.send(encode(&msg));
            }
        }

        self.spawn_cooldown = SPAWN_INTERVAL;
    }
}
