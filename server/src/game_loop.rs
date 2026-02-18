use std::time::Duration;

use tafels_shared::protocol::{LobbyPlayer, PlayerScore, ServerMessage, encode};
use tokio::time;

use crate::beacon::BeaconManager;
use crate::state::{AppState, GameWorld, RoundState};

pub async fn run_game_loop(state: AppState) {
    let mut interval = time::interval(Duration::from_millis(50)); // 20 Hz
    let mut beacon_manager = BeaconManager::new();
    let dt = 0.05_f32; // 50ms per tick

    loop {
        interval.tick().await;

        // Drive round state machine
        tick_round_state(dt, &state).await;

        // Only manage beacons during Playing state
        let is_playing = {
            let world = state.world.lock().await;
            world.round_state.is_playing()
        };
        if is_playing {
            beacon_manager.tick(dt, &state).await;
        }

        // Broadcast player positions
        let world = state.world.lock().await;
        if world.players.is_empty() {
            continue;
        }

        let players: Vec<_> = world.players.values().copied().collect();
        drop(world);

        let msg = ServerMessage::PlayerUpdate(players);
        let bytes = encode(&msg);
        let _ = state.broadcast_tx.send(bytes);
    }
}

async fn tick_round_state(dt: f32, state: &AppState) {
    let mut world = state.world.lock().await;

    let transition = match world.round_state {
        RoundState::Lobby => None,
        RoundState::Countdown(ref mut remaining) => {
            *remaining -= dt;
            if *remaining <= 0.0 {
                Some(RoundState::Playing(world.round_time()))
            } else {
                None
            }
        }
        RoundState::Playing(ref mut remaining) => {
            *remaining -= dt;
            if *remaining <= 0.0 {
                // Collect final scores
                let scores: Vec<PlayerScore> = world
                    .players
                    .keys()
                    .map(|&pid| PlayerScore {
                        player_id: pid,
                        name: world
                            .player_names
                            .get(&pid)
                            .cloned()
                            .unwrap_or_else(|| format!("Player {pid}")),
                        coins: world.player_coins.get(&pid).copied().unwrap_or(0),
                    })
                    .collect();

                let msg = ServerMessage::RoundOver {
                    scores: scores.clone(),
                };
                let _ = state.broadcast_tx.send(encode(&msg));

                Some(RoundState::RoundOver(5.0))
            } else {
                None
            }
        }
        RoundState::RoundOver(ref mut remaining) => {
            *remaining -= dt;
            if *remaining <= 0.0 {
                // Reset for next round
                world.player_ready.clear();
                world.player_coins.clear();
                world.beacons.clear();
                world.activated_beacons.clear();

                let lobby_players = build_lobby_players(&world);
                let msg = ServerMessage::LobbyState {
                    players: lobby_players,
                };
                let _ = state.broadcast_tx.send(encode(&msg));

                Some(RoundState::Lobby)
            } else {
                None
            }
        }
    };

    if let Some(new_state) = transition {
        if let RoundState::Playing(round_time) = &new_state {
            let msg = ServerMessage::RoundStart {
                round_time: *round_time,
            };
            let _ = state.broadcast_tx.send(encode(&msg));
        }
        world.round_state = new_state;
    }
}

pub fn build_lobby_players(world: &GameWorld) -> Vec<LobbyPlayer> {
    world
        .players
        .keys()
        .map(|&pid| LobbyPlayer {
            player_id: pid,
            name: world
                .player_names
                .get(&pid)
                .cloned()
                .unwrap_or_else(|| format!("Player {pid}")),
            ready: world.player_ready.contains(&pid),
        })
        .collect()
}
