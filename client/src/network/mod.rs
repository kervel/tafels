pub mod remote_players;

use std::sync::Mutex;

use bevy::prelude::*;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use tafels_shared::protocol::{
    AnimationState, BeaconInfo, ClientMessage, PlayerState,
    ServerMessage, decode, encode,
};

use crate::character::{CharacterMarker, CharacterState};
use crate::game::{GameSession, GameState, Leaderboard, LeaderboardEntry, MultiplayerRoundState};

/// For WASM builds, derive WebSocket URL from the page's origin at runtime.
#[cfg(target_arch = "wasm32")]
fn server_url() -> String {
    let location = web_sys::window()
        .and_then(|w| w.location().href().ok())
        .unwrap_or_default();
    if location.starts_with("https://") {
        let host = location
            .trim_start_matches("https://")
            .split('/')
            .next()
            .unwrap_or("localhost");
        format!("wss://{host}/ws")
    } else {
        let host = location
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("localhost:3000");
        format!("ws://{host}/ws")
    }
}

/// For native builds, use SERVER_URL env var or derive from page origin.
#[cfg(not(target_arch = "wasm32"))]
fn server_url() -> String {
    std::env::var("SERVER_URL").unwrap_or_else(|_| "ws://localhost:30000/ws".to_string())
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConnectionStatus>()
            .init_resource::<RoundMessageBuffer>()
            .add_message::<RemotePlayerJoined>()
            .add_message::<RemotePlayerUpdated>()
            .add_message::<RemotePlayerLeft>()
            .add_message::<BeaconSpawnedEvent>()
            .add_message::<BeaconResolvedEvent>()
            .add_message::<BeaconExpiredEvent>()
            .add_message::<BeaconActivatedEvent>()
            .add_message::<RemoteBallShotEvent>()
            .add_systems(OnEnter(GameState::Playing), connect_to_server)
            .add_systems(OnExit(GameState::Playing), disconnect_from_server)
            .add_systems(
                Update,
                receive_messages.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                send_player_state.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                send_score_updates.run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                handle_round_events.run_if(in_state(GameState::Playing)),
            );
    }
}

/// Wrapper to make ewebsock types usable as Bevy Resources.
/// On native, ewebsock types are Send but not Sync — Mutex fixes that.
/// On WASM, they use Rc internally (not Send), but WASM is single-threaded
/// so this is safe. We use an unsafe Send+Sync wrapper for WASM.
struct SendSyncWrapper<T>(T);
// SAFETY: WASM is single-threaded; on native ewebsock types are already Send.
unsafe impl<T> Send for SendSyncWrapper<T> {}
unsafe impl<T> Sync for SendSyncWrapper<T> {}

#[derive(Resource)]
pub struct WsConnection {
    sender: Mutex<SendSyncWrapper<WsSender>>,
    receiver: Mutex<SendSyncWrapper<WsReceiver>>,
}

impl WsConnection {
    fn new(sender: WsSender, receiver: WsReceiver) -> Self {
        Self {
            sender: Mutex::new(SendSyncWrapper(sender)),
            receiver: Mutex::new(SendSyncWrapper(receiver)),
        }
    }

    /// Send a binary message via the WebSocket.
    pub fn send_binary(&self, bytes: Vec<u8>) {
        let mut sender = self.sender.lock().unwrap();
        sender.0.send(WsMessage::Binary(bytes));
    }
}

/// Send+Sync resource tracking connection state.
#[derive(Resource, Default)]
pub enum ConnectionStatus {
    #[default]
    Disconnected,
    Connecting,
    Connected {
        my_player_id: u32,
        my_color_index: u8,
        send_timer: Timer,
    },
    Reconnecting {
        attempt: u32,
        next_try: Timer,
    },
}

impl ConnectionStatus {
    pub fn is_online(&self) -> bool {
        matches!(self, ConnectionStatus::Connected { .. })
    }
}

// Network messages (Bevy 0.18 uses Message instead of Event)
#[derive(Message)]
pub struct RemotePlayerJoined(pub PlayerState);

#[derive(Message)]
pub struct RemotePlayerUpdated(pub Vec<PlayerState>);

#[derive(Message)]
pub struct RemotePlayerLeft {
    pub player_id: u32,
}

#[derive(Message)]
pub struct BeaconSpawnedEvent(pub BeaconInfo);

#[derive(Message)]
#[allow(dead_code)]
pub struct BeaconResolvedEvent {
    pub beacon_id: u32,
    pub claimed_by: u32,
}

#[derive(Message)]
pub struct BeaconExpiredEvent {
    pub beacon_id: u32,
}

#[derive(Message)]
pub struct BeaconActivatedEvent {
    pub beacon_id: u32,
    pub activated_by: u32,
}

#[derive(Message)]
pub struct RemoteBallShotEvent {
    pub player_id: u32,
    pub position: Vec3,
    pub velocity: Vec3,
}

/// Buffered round messages from the server, processed in handle_round_events.
#[derive(Resource, Default)]
pub struct RoundMessageBuffer {
    pub lobby_states: Vec<tafels_shared::protocol::LobbyPlayer>,
    pub lobby_dirty: bool,
    pub countdown: Option<u8>,
    pub round_start: Option<f32>,
    pub round_over: Option<Vec<tafels_shared::protocol::PlayerScore>>,
    pub score_updates: Vec<(u32, i32)>,
}

fn connect_to_server(mut status: ResMut<ConnectionStatus>, mut commands: Commands) {
    let url = server_url();
    match ewebsock::connect(&url, ewebsock::Options::default()) {
        Ok((sender, receiver)) => {
            info!("Connecting to server at {url}");
            commands.insert_resource(WsConnection::new(sender, receiver));
            *status = ConnectionStatus::Connecting;
        }
        Err(e) => {
            warn!("Failed to connect to server: {e}. Running in single-player mode.");
            *status = ConnectionStatus::Disconnected;
        }
    }
}

fn disconnect_from_server(
    mut status: ResMut<ConnectionStatus>,
    mut commands: Commands,
    mut round_state: ResMut<MultiplayerRoundState>,
) {
    commands.remove_resource::<WsConnection>();
    *status = ConnectionStatus::Disconnected;
    *round_state = MultiplayerRoundState::None;
}

#[allow(clippy::too_many_arguments)]
fn receive_messages(
    mut status: ResMut<ConnectionStatus>,
    ws: Option<Res<WsConnection>>,
    mut commands: Commands,
    mut joined_events: MessageWriter<RemotePlayerJoined>,
    mut updated_events: MessageWriter<RemotePlayerUpdated>,
    mut left_events: MessageWriter<RemotePlayerLeft>,
    mut beacon_spawned: MessageWriter<BeaconSpawnedEvent>,
    mut beacon_resolved: MessageWriter<BeaconResolvedEvent>,
    mut beacon_expired: MessageWriter<BeaconExpiredEvent>,
    mut beacon_activated: MessageWriter<BeaconActivatedEvent>,
    mut ball_shot: MessageWriter<RemoteBallShotEvent>,
    mut round_buf: ResMut<RoundMessageBuffer>,
    session: Res<GameSession>,
    time: Res<Time>,
) {
    // Handle reconnection timer
    if let ConnectionStatus::Reconnecting {
        attempt,
        ref mut next_try,
    } = *status
    {
        next_try.tick(time.delta());
        if next_try.just_finished() {
            let url = server_url();
            match ewebsock::connect(&url, ewebsock::Options::default()) {
                Ok((sender, receiver)) => {
                    info!("Reconnecting to server (attempt {attempt})...");
                    commands.insert_resource(WsConnection::new(sender, receiver));
                    *status = ConnectionStatus::Connecting;
                }
                Err(_) => {
                    let backoff = (2u64.pow(attempt.min(5))).min(30) as f32;
                    *status = ConnectionStatus::Reconnecting {
                        attempt: attempt + 1,
                        next_try: Timer::from_seconds(backoff, TimerMode::Once),
                    };
                }
            }
        }
        return;
    }

    let Some(ws) = ws else {
        return;
    };

    let my_player_id = match &*status {
        ConnectionStatus::Connecting => None,
        ConnectionStatus::Connected { my_player_id, .. } => Some(*my_player_id),
        _ => return,
    };

    let receiver = ws.receiver.lock().unwrap();

    // Drain all pending messages
    while let Some(event) = receiver.0.try_recv() {
        match event {
            WsEvent::Message(WsMessage::Binary(bytes)) => {
                if let Ok(msg) = decode::<ServerMessage>(&bytes) {
                    match msg {
                        ServerMessage::WorldSnapshot {
                            your_player_id: pid,
                            players,
                            beacons,
                        } => {
                            if my_player_id.is_none() {
                                let my_color = players
                                    .iter()
                                    .find(|p| p.player_id == pid)
                                    .map(|p| p.color_index)
                                    .unwrap_or(0);
                                info!("Connected as Player {pid} (color {my_color})");
                                for p in &players {
                                    if p.player_id != pid {
                                        joined_events.write(RemotePlayerJoined(*p));
                                    }
                                }
                                for b in beacons {
                                    beacon_spawned.write(BeaconSpawnedEvent(b));
                                }

                                // Send player name now that we're connected
                                let name = if session.player_name.is_empty() {
                                    "Player".to_string()
                                } else {
                                    session.player_name.clone()
                                };
                                {
                                    let mut sender = ws.sender.lock().unwrap();
                                    sender.0.send(WsMessage::Binary(encode(&ClientMessage::SetName { name })));
                                }

                                drop(receiver);
                                *status = ConnectionStatus::Connected {
                                    my_player_id: pid,
                                    my_color_index: my_color,
                                    send_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                                };
                                return;
                            }
                        }
                        ServerMessage::PlayerJoined(ps) => {
                            info!("Received PlayerJoined for player {}", ps.player_id);
                            if Some(ps.player_id) != my_player_id {
                                joined_events.write(RemotePlayerJoined(ps));
                            }
                        }
                        ServerMessage::PlayerUpdate(players) => {
                            updated_events.write(RemotePlayerUpdated(players));
                        }
                        ServerMessage::PlayerLeft { player_id } => {
                            left_events.write(RemotePlayerLeft { player_id });
                        }
                        ServerMessage::BeaconSpawned(info) => {
                            beacon_spawned.write(BeaconSpawnedEvent(info));
                        }
                        ServerMessage::BeaconResolved {
                            beacon_id,
                            claimed_by,
                        } => {
                            beacon_resolved.write(BeaconResolvedEvent {
                                beacon_id,
                                claimed_by,
                            });
                        }
                        ServerMessage::BeaconExpired { beacon_id } => {
                            beacon_expired.write(BeaconExpiredEvent { beacon_id });
                        }
                        ServerMessage::BeaconActivated {
                            beacon_id,
                            activated_by,
                        } => {
                            beacon_activated.write(BeaconActivatedEvent {
                                beacon_id,
                                activated_by,
                            });
                        }
                        ServerMessage::BallShot {
                            player_id: pid,
                            x, y, z,
                            vx, vy, vz,
                        } => {
                            if Some(pid) != my_player_id {
                                ball_shot.write(RemoteBallShotEvent {
                                    player_id: pid,
                                    position: Vec3::new(x, y, z),
                                    velocity: Vec3::new(vx, vy, vz),
                                });
                            }
                        }
                        ServerMessage::LobbyState { players } => {
                            round_buf.lobby_states = players;
                            round_buf.lobby_dirty = true;
                        }
                        ServerMessage::CountdownStart { seconds } => {
                            round_buf.countdown = Some(seconds);
                        }
                        ServerMessage::RoundStart { round_time } => {
                            round_buf.round_start = Some(round_time);
                        }
                        ServerMessage::RoundOver { scores } => {
                            round_buf.round_over = Some(scores);
                        }
                        ServerMessage::ScoreUpdate { player_id, coins } => {
                            if Some(player_id) != my_player_id {
                                round_buf.score_updates.push((player_id, coins));
                            }
                        }
                    }
                }
            }
            WsEvent::Closed | WsEvent::Error(_) => {
                warn!("WebSocket connection lost. Attempting reconnect...");
                drop(receiver);
                commands.remove_resource::<WsConnection>();
                *status = ConnectionStatus::Reconnecting {
                    attempt: 1,
                    next_try: Timer::from_seconds(1.0, TimerMode::Once),
                };
                return;
            }
            _ => {}
        }
    }
}

fn send_player_state(
    mut status: ResMut<ConnectionStatus>,
    ws: Option<Res<WsConnection>>,
    character: Query<(&Transform, &CharacterState), With<CharacterMarker>>,
    time: Res<Time>,
) {
    let ConnectionStatus::Connected {
        my_player_id,
        my_color_index,
        ref mut send_timer,
    } = *status
    else {
        return;
    };
    send_timer.tick(time.delta());
    if !send_timer.just_finished() {
        return;
    }

    let Some(ws) = ws else {
        return;
    };

    let Ok((transform, char_state)) = character.single() else {
        return;
    };

    let animation = match char_state {
        CharacterState::Idle => AnimationState::Idle,
        CharacterState::Walking => AnimationState::Walking,
        CharacterState::Running => AnimationState::Running,
    };

    let (yaw_rad, _, _) = transform.rotation.to_euler(EulerRot::YXZ);

    let msg = ClientMessage::UpdateState(PlayerState {
        player_id: my_player_id,
        color_index: my_color_index,
        x: transform.translation.x,
        y: transform.translation.y,
        z: transform.translation.z,
        yaw: yaw_rad,
        animation,
    });

    let mut sender = ws.sender.lock().unwrap();
    sender.0.send(WsMessage::Binary(encode(&msg)));
}

fn send_score_updates(
    ws: Option<Res<WsConnection>>,
    status: Res<ConnectionStatus>,
    mut session: ResMut<GameSession>,
) {
    if !status.is_online() {
        return;
    }
    let Some(ws) = ws else {
        return;
    };

    if session.coins != session.prev_coins {
        session.prev_coins = session.coins;
        let msg = ClientMessage::UpdateScore {
            coins: session.coins,
        };
        ws.send_binary(encode(&msg));
    }
}

fn handle_round_events(
    mut round_state: ResMut<MultiplayerRoundState>,
    mut leaderboard: ResMut<Leaderboard>,
    mut round_buf: ResMut<RoundMessageBuffer>,
    mut session: ResMut<GameSession>,
    status: Res<ConnectionStatus>,
) {
    if round_buf.lobby_dirty {
        round_buf.lobby_dirty = false;
        *round_state = MultiplayerRoundState::Lobby;
        leaderboard.entries = round_buf
            .lobby_states
            .iter()
            .map(|p| LeaderboardEntry {
                player_id: p.player_id,
                name: p.name.clone(),
                coins: 0,
            })
            .collect();
    }

    if let Some(secs) = round_buf.countdown.take() {
        *round_state = MultiplayerRoundState::Countdown(secs as f32);
    }

    if let Some(_round_time) = round_buf.round_start.take() {
        *round_state = MultiplayerRoundState::Playing;
        // Force send initial score so other players see our starting coins
        session.prev_coins = i32::MIN;
    }

    if let Some(scores) = round_buf.round_over.take() {
        *round_state = MultiplayerRoundState::RoundOver;
        leaderboard.entries = scores
            .iter()
            .map(|s| LeaderboardEntry {
                player_id: s.player_id,
                name: s.name.clone(),
                coins: s.coins,
            })
            .collect();
        leaderboard.entries.sort_by(|a, b| b.coins.cmp(&a.coins));
    }

    // Update leaderboard from score updates
    for &(player_id, coins) in &round_buf.score_updates {
        if let Some(entry) = leaderboard
            .entries
            .iter_mut()
            .find(|e| e.player_id == player_id)
        {
            entry.coins = coins;
        }
    }
    round_buf.score_updates.clear();

    // Update own score in leaderboard
    if let ConnectionStatus::Connected { my_player_id, .. } = &*status {
        if let Some(entry) = leaderboard
            .entries
            .iter_mut()
            .find(|e| e.player_id == *my_player_id)
        {
            entry.coins = session.coins;
        }
    }
}
