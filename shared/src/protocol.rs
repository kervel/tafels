use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationState {
    Idle,
    Walking,
    Running,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayer {
    pub player_id: u32,
    pub name: String,
    pub ready: bool,
    pub playing_solo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerScore {
    pub player_id: u32,
    pub name: String,
    pub coins: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PlayerState {
    pub player_id: u32,
    pub color_index: u8,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub yaw: f32,
    pub animation: AnimationState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaconInfo {
    pub beacon_id: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub question_text: String,
    pub choices: [u32; 4],
    pub correct_index: u8,
    pub lifetime: f32,
    pub target_player_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerMessage {
    WorldSnapshot {
        your_player_id: u32,
        players: Vec<PlayerState>,
        beacons: Vec<BeaconInfo>,
    },
    PlayerJoined(PlayerState),
    PlayerUpdate(Vec<PlayerState>),
    PlayerLeft {
        player_id: u32,
    },
    BeaconSpawned(BeaconInfo),
    BeaconResolved {
        beacon_id: u32,
        claimed_by: u32,
    },
    BeaconExpired {
        beacon_id: u32,
    },
    BeaconActivated {
        beacon_id: u32,
        activated_by: u32,
    },
    BallShot {
        player_id: u32,
        x: f32,
        y: f32,
        z: f32,
        vx: f32,
        vy: f32,
        vz: f32,
    },
    LobbyState {
        players: Vec<LobbyPlayer>,
    },
    CountdownStart {
        seconds: u8,
    },
    RoundStart {
        round_time: f32,
    },
    RoundOver {
        scores: Vec<PlayerScore>,
    },
    ScoreUpdate {
        player_id: u32,
        coins: i32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientMessage {
    UpdateState(PlayerState),
    AnswerBeacon { beacon_id: u32, correct: bool },
    ActivateBeacon { beacon_id: u32 },
    ShootBall { x: f32, y: f32, z: f32, vx: f32, vy: f32, vz: f32 },
    SetName { name: String },
    SetDifficulty { difficulty: crate::difficulty::Difficulty },
    Ready,
    UpdateScore { coins: i32 },
    EnterSolo,
    LeaveSolo,
    StartRoundNow,
}

pub fn encode<T: Serialize>(msg: &T) -> Vec<u8> {
    postcard::to_allocvec(msg).expect("serialization should not fail")
}

pub fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, postcard::Error> {
    postcard::from_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_player_state() {
        let state = PlayerState {
            player_id: 42,
            color_index: 0,
            x: 1.0,
            y: 2.5,
            z: -3.0,
            yaw: 1.57,
            animation: AnimationState::Walking,
        };
        let bytes = encode(&state);
        let decoded: PlayerState = decode(&bytes).unwrap();
        assert_eq!(decoded.player_id, 42);
        assert!((decoded.x - 1.0).abs() < f32::EPSILON);
        assert_eq!(decoded.animation, AnimationState::Walking);
    }

    #[test]
    fn roundtrip_server_message() {
        let msg = ServerMessage::WorldSnapshot {
            your_player_id: 1,
            players: vec![PlayerState {
                player_id: 1,
                color_index: 0,
                x: 0.0,
                y: 0.0,
                z: 0.0,
                yaw: 0.0,
                animation: AnimationState::Idle,
            }],
            beacons: vec![BeaconInfo {
                beacon_id: 10,
                x: 5.0,
                y: 1.0,
                z: 5.0,
                question_text: "7 x 8 = ?".to_string(),
                choices: [56, 48, 63, 54],
                correct_index: 0,
                lifetime: 45.0,
                target_player_id: 1,
            }],
        };
        let bytes = encode(&msg);
        let decoded: ServerMessage = decode(&bytes).unwrap();
        match decoded {
            ServerMessage::WorldSnapshot {
                your_player_id,
                players,
                beacons,
            } => {
                assert_eq!(your_player_id, 1);
                assert_eq!(players.len(), 1);
                assert_eq!(beacons.len(), 1);
                assert_eq!(beacons[0].question_text, "7 x 8 = ?");
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_client_message() {
        let msg = ClientMessage::AnswerBeacon {
            beacon_id: 5,
            correct: true,
        };
        let bytes = encode(&msg);
        let decoded: ClientMessage = decode(&bytes).unwrap();
        match decoded {
            ClientMessage::AnswerBeacon { beacon_id, correct } => {
                assert_eq!(beacon_id, 5);
                assert!(correct);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_beacon_resolved() {
        let msg = ServerMessage::BeaconResolved {
            beacon_id: 3,
            claimed_by: 7,
        };
        let bytes = encode(&msg);
        let decoded: ServerMessage = decode(&bytes).unwrap();
        match decoded {
            ServerMessage::BeaconResolved {
                beacon_id,
                claimed_by,
            } => {
                assert_eq!(beacon_id, 3);
                assert_eq!(claimed_by, 7);
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn roundtrip_player_joined_left() {
        let joined = ServerMessage::PlayerJoined(PlayerState {
            player_id: 99,
            color_index: 2,
            x: 10.0,
            y: 5.0,
            z: -10.0,
            yaw: 3.14,
            animation: AnimationState::Running,
        });
        let bytes = encode(&joined);
        let decoded: ServerMessage = decode(&bytes).unwrap();
        match decoded {
            ServerMessage::PlayerJoined(ps) => {
                assert_eq!(ps.player_id, 99);
                assert_eq!(ps.animation, AnimationState::Running);
            }
            _ => panic!("wrong variant"),
        }

        let left = ServerMessage::PlayerLeft { player_id: 99 };
        let bytes = encode(&left);
        let decoded: ServerMessage = decode(&bytes).unwrap();
        match decoded {
            ServerMessage::PlayerLeft { player_id } => assert_eq!(player_id, 99),
            _ => panic!("wrong variant"),
        }
    }
}
