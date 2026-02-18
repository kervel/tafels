pub mod remote_players;

use std::sync::Mutex;

use bevy::prelude::*;
use ewebsock::{WsEvent, WsMessage, WsReceiver, WsSender};
use tafels_shared::protocol::{
    AnimationState, BeaconInfo, ClientMessage, PlayerState, ServerMessage, decode, encode,
};

use crate::character::{CharacterMarker, CharacterState};
use crate::game::GameState;

/// Server URL — set at compile time via cfg.
/// For WASM builds, uses the page's host with wss://.
/// For native dev builds, defaults to localhost.
#[cfg(target_arch = "wasm32")]
const SERVER_URL: &str = "wss://tafels.example.com/ws";
#[cfg(not(target_arch = "wasm32"))]
const SERVER_URL: &str = "ws://localhost:3000/ws";

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ConnectionStatus>()
            .add_message::<RemotePlayerJoined>()
            .add_message::<RemotePlayerUpdated>()
            .add_message::<RemotePlayerLeft>()
            .add_message::<BeaconSpawnedEvent>()
            .add_message::<BeaconResolvedEvent>()
            .add_message::<BeaconExpiredEvent>()
            .add_systems(OnEnter(GameState::Playing), connect_to_server)
            .add_systems(OnExit(GameState::Playing), disconnect_from_server)
            .add_systems(
                Update,
                (receive_messages, send_player_state).run_if(in_state(GameState::Playing)),
            );
    }
}

/// Resource holding WebSocket connection handles.
/// Wrapped in Mutex to satisfy Sync (ewebsock uses mpsc internally).
/// Only accessed from the main thread, so contention is never an issue.
#[derive(Resource)]
pub struct WsConnection {
    sender: Mutex<WsSender>,
    receiver: Mutex<WsReceiver>,
}

impl WsConnection {
    fn new(sender: WsSender, receiver: WsReceiver) -> Self {
        Self {
            sender: Mutex::new(sender),
            receiver: Mutex::new(receiver),
        }
    }

    /// Send a binary message via the WebSocket.
    pub fn send_binary(&self, bytes: Vec<u8>) {
        let mut sender = self.sender.lock().unwrap();
        sender.send(WsMessage::Binary(bytes));
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

fn connect_to_server(mut status: ResMut<ConnectionStatus>, mut commands: Commands) {
    match ewebsock::connect(SERVER_URL, ewebsock::Options::default()) {
        Ok((sender, receiver)) => {
            info!("Connecting to server at {SERVER_URL}");
            commands.insert_resource(WsConnection::new(sender, receiver));
            *status = ConnectionStatus::Connecting;
        }
        Err(e) => {
            warn!("Failed to connect to server: {e}. Running in single-player mode.");
            *status = ConnectionStatus::Disconnected;
        }
    }
}

fn disconnect_from_server(mut status: ResMut<ConnectionStatus>, mut commands: Commands) {
    commands.remove_resource::<WsConnection>();
    *status = ConnectionStatus::Disconnected;
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
            match ewebsock::connect(SERVER_URL, ewebsock::Options::default()) {
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
    while let Some(event) = receiver.try_recv() {
        match event {
            WsEvent::Message(WsMessage::Binary(bytes)) => {
                if let Ok(msg) = decode::<ServerMessage>(&bytes) {
                    match msg {
                        ServerMessage::WorldSnapshot { players, beacons } => {
                            if my_player_id.is_none()
                                && let Some(me) = players.last()
                            {
                                let pid = me.player_id;
                                info!("Connected as Player {pid}");
                                for p in &players {
                                    if p.player_id != pid {
                                        joined_events.write(RemotePlayerJoined(*p));
                                    }
                                }
                                for b in beacons {
                                    beacon_spawned.write(BeaconSpawnedEvent(b));
                                }
                                drop(receiver);
                                *status = ConnectionStatus::Connected {
                                    my_player_id: pid,
                                    send_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                                };
                                return;
                            }
                        }
                        ServerMessage::PlayerJoined(ps) => {
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

    let (_, yaw_rad, _) = transform.rotation.to_euler(EulerRot::YXZ);

    let msg = ClientMessage::UpdateState(PlayerState {
        player_id: my_player_id,
        x: transform.translation.x,
        y: transform.translation.y,
        z: transform.translation.z,
        yaw: yaw_rad,
        animation,
    });

    let mut sender = ws.sender.lock().unwrap();
    sender.send(WsMessage::Binary(encode(&msg)));
}
