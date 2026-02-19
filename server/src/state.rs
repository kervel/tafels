use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use tafels_shared::difficulty::Difficulty;
use tafels_shared::protocol::{BeaconInfo, PlayerState};
use tokio::sync::{Mutex, broadcast};

#[derive(Debug, Clone)]
pub enum RoundState {
    Lobby,
    Countdown(f32),
    Playing(f32),
    RoundOver(f32),
}

impl RoundState {
    pub fn is_playing(&self) -> bool {
        matches!(self, RoundState::Playing(_))
    }
}

pub struct GameWorld {
    pub players: HashMap<u32, PlayerState>,
    pub beacons: HashMap<u32, BeaconInfo>,
    pub activated_beacons: HashSet<u32>,
    pub next_player_id: u32,
    pub next_beacon_id: u32,
    pub player_names: HashMap<u32, String>,
    pub player_ready: HashSet<u32>,
    pub player_coins: HashMap<u32, i32>,
    pub player_solo: HashSet<u32>,
    pub round_state: RoundState,
    pub difficulty: Difficulty,
}

impl GameWorld {
    pub fn new() -> Self {
        Self {
            players: HashMap::new(),
            beacons: HashMap::new(),
            activated_beacons: HashSet::new(),
            next_player_id: 1,
            next_beacon_id: 1,
            player_names: HashMap::new(),
            player_ready: HashSet::new(),
            player_solo: HashSet::new(),
            player_coins: HashMap::new(),
            round_state: RoundState::Lobby,
            difficulty: Difficulty::Easy,
        }
    }

    pub fn add_player(&mut self) -> PlayerState {
        let id = self.next_player_id;
        self.next_player_id += 1;
        let state = PlayerState {
            player_id: id,
            color_index: ((id - 1) % 6) as u8,
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
        self.player_names.remove(&player_id);
        self.player_ready.remove(&player_id);
        self.player_solo.remove(&player_id);
        self.player_coins.remove(&player_id);
    }

    pub fn round_time(&self) -> f32 {
        self.difficulty.round_time()
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
