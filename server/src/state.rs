use std::collections::HashMap;
use std::sync::Arc;

use tafels_shared::protocol::{BeaconInfo, PlayerState};
use tokio::sync::{Mutex, broadcast};

pub struct GameWorld {
    pub players: HashMap<u32, PlayerState>,
    pub beacons: HashMap<u32, BeaconInfo>,
    pub next_player_id: u32,
    pub next_beacon_id: u32,
}

impl GameWorld {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            beacons: HashMap::new(),
            next_player_id: 1,
            next_beacon_id: 1,
        }
    }

    pub fn add_player(&mut self) -> PlayerState {
        let id = self.next_player_id;
        self.next_player_id += 1;
        let state = PlayerState {
            player_id: id,
            x: 0.0,
            y: 0.0,
            z: 0.0,
            yaw: 0.0,
            animation: tafels_shared::protocol::AnimationState::Idle,
        };
        self.players.insert(id, state);
        state
    }

    pub fn remove_player(&mut self, player_id: u32) {
        self.players.remove(&player_id);
    }
}

#[derive(Clone)]
pub struct AppState {
    pub world: Arc<Mutex<GameWorld>>,
    pub broadcast_tx: broadcast::Sender<Vec<u8>>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self {
            world: Arc::new(Mutex::new(GameWorld::new())),
            broadcast_tx: tx,
        }
    }
}
