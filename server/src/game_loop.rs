use std::time::Duration;

use tafels_shared::protocol::{ServerMessage, encode};
use tokio::time;

use crate::beacon::BeaconManager;
use crate::state::AppState;

pub async fn run_game_loop(state: AppState) {
    let mut interval = time::interval(Duration::from_millis(50)); // 20 Hz
    let mut beacon_manager = BeaconManager::new();
    let dt = 0.05_f32; // 50ms per tick

    loop {
        interval.tick().await;

        // Manage beacon spawning and expiry
        beacon_manager.tick(dt, &state).await;

        let world = state.world.lock().await;
        if world.players.is_empty() {
            continue;
        }

        let players: Vec<_> = world.players.values().copied().collect();
        drop(world);

        let msg = ServerMessage::PlayerUpdate(players);
        let bytes = encode(&msg);

        // Ignore send errors (no receivers)
        let _ = state.broadcast_tx.send(bytes);
    }
}
