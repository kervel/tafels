mod beacon;
mod game_loop;
mod state;

use axum::Router;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use futures_util::{SinkExt, StreamExt};
use tafels_shared::protocol::{ClientMessage, PlayerScore, ServerMessage, decode, encode};
use tower_http::services::ServeDir;
use tower_http::set_header::SetResponseHeaderLayer;
use http::HeaderName;

use crate::game_loop::build_lobby_players;
use crate::state::{AppState, GameWorld, RoundState};

#[tokio::main]
async fn main() {
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
    let static_dir = std::env::var("STATIC_DIR").unwrap_or_else(|_| "/app/dist".to_string());

    let state = AppState::new();

    // Spawn game loop
    let loop_state = state.clone();
    tokio::spawn(game_loop::run_game_loop(loop_state));

    // Static file serving: serve WASM build from STATIC_DIR,
    // with precompressed brotli support and Cross-Origin headers for SharedArrayBuffer
    let static_service = ServeDir::new(&static_dir).precompressed_br();

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health))
        .with_state(state)
        .fallback_service(static_service)
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("cross-origin-opener-policy"),
            http::HeaderValue::from_static("same-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("cross-origin-embedder-policy"),
            http::HeaderValue::from_static("require-corp"),
        ));

    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    println!("tafels-server listening on {bind_addr} (static: {static_dir})");
    axum::serve(listener, app).await.unwrap();
}

async fn health() -> &'static str {
    "OK"
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut ws_tx, mut ws_rx) = socket.split();

    // Assign player ID and send WorldSnapshot
    let (player_id, snapshot_bytes, round_state_msgs) = {
        let mut world = state.world.lock().await;
        let player_state = world.add_player();
        let player_id = player_state.player_id;

        let snapshot = ServerMessage::WorldSnapshot {
            your_player_id: player_id,
            players: world.players.values().copied().collect(),
            beacons: world.beacons.values().cloned().collect(),
        };
        let snapshot_bytes = encode(&snapshot);

        // Broadcast PlayerJoined to others
        let joined = ServerMessage::PlayerJoined(player_state);
        let _ = state.broadcast_tx.send(encode(&joined));

        // Build round-state messages for late joiners
        let round_msgs = build_round_state_messages(&world);

        (player_id, snapshot_bytes, round_msgs)
    };

    println!("Player {player_id} connected");

    // Send WorldSnapshot to the new player
    if ws_tx.send(Message::Binary(snapshot_bytes)).await.is_err() {
        cleanup_player(player_id, &state).await;
        return;
    }

    // Send current round state to late joiners
    for msg_bytes in round_state_msgs {
        if ws_tx.send(Message::Binary(msg_bytes)).await.is_err() {
            cleanup_player(player_id, &state).await;
            return;
        }
    }

    // Subscribe to broadcasts
    let mut broadcast_rx = state.broadcast_tx.subscribe();

    // Writer task: forward broadcasts to this client
    let _write_state = state.clone();
    let mut write_task = tokio::spawn(async move {
        while let Ok(bytes) = broadcast_rx.recv().await {
            if ws_tx.send(Message::Binary(bytes)).await.is_err() {
                break;
            }
        }
    });

    // Reader task: process incoming messages from this client
    let read_state = state.clone();
    let mut read_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Binary(bytes) => {
                    if let Ok(client_msg) = decode::<ClientMessage>(&bytes) {
                        handle_client_message(player_id, client_msg, &read_state).await;
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // Wait for either task to finish
    tokio::select! {
        _ = &mut write_task => { read_task.abort(); }
        _ = &mut read_task => { write_task.abort(); }
    }

    cleanup_player(player_id, &state).await;
    println!("Player {player_id} disconnected");
}

fn build_round_state_messages(world: &GameWorld) -> Vec<Vec<u8>> {
    let mut msgs = Vec::new();
    match &world.round_state {
        RoundState::Lobby => {
            let lobby_players = build_lobby_players(world);
            msgs.push(encode(&ServerMessage::LobbyState {
                players: lobby_players,
            }));
        }
        RoundState::Countdown(remaining) => {
            let lobby_players = build_lobby_players(world);
            msgs.push(encode(&ServerMessage::LobbyState {
                players: lobby_players,
            }));
            msgs.push(encode(&ServerMessage::CountdownStart {
                seconds: remaining.ceil() as u8,
            }));
        }
        RoundState::Playing(remaining) => {
            msgs.push(encode(&ServerMessage::RoundStart {
                round_time: *remaining,
            }));
        }
        RoundState::RoundOver(_) => {
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
            msgs.push(encode(&ServerMessage::RoundOver { scores }));
        }
    }
    msgs
}

async fn handle_client_message(player_id: u32, msg: ClientMessage, state: &AppState) {
    match msg {
        ClientMessage::UpdateState(mut player_state) => {
            // Ensure player_id matches the connection's assigned ID
            player_state.player_id = player_id;
            let mut world = state.world.lock().await;
            if world.players.contains_key(&player_id) {
                world.players.insert(player_id, player_state);
            }
        }
        ClientMessage::AnswerBeacon { beacon_id, correct } => {
            if !correct {
                return;
            }
            let mut world = state.world.lock().await;
            if world.beacons.remove(&beacon_id).is_some() {
                world.activated_beacons.remove(&beacon_id);
                let msg = ServerMessage::BeaconResolved {
                    beacon_id,
                    claimed_by: player_id,
                };
                let _ = state.broadcast_tx.send(encode(&msg));
            }
        }
        ClientMessage::ShootBall { x, y, z, vx, vy, vz } => {
            let msg = ServerMessage::BallShot {
                player_id,
                x, y, z,
                vx, vy, vz,
            };
            let _ = state.broadcast_tx.send(encode(&msg));
        }
        ClientMessage::ActivateBeacon { beacon_id } => {
            let mut world = state.world.lock().await;
            if world.beacons.contains_key(&beacon_id)
                && !world.activated_beacons.contains(&beacon_id)
            {
                world.activated_beacons.insert(beacon_id);
                let msg = ServerMessage::BeaconActivated {
                    beacon_id,
                    activated_by: player_id,
                };
                let _ = state.broadcast_tx.send(encode(&msg));
            }
        }
        ClientMessage::SetName { name } => {
            let mut world = state.world.lock().await;
            // Sanitize: limit length, trim
            let name = name.trim().chars().take(20).collect::<String>();
            let name = if name.is_empty() {
                format!("Player {player_id}")
            } else {
                name
            };
            world.player_names.insert(player_id, name);

            let lobby_players = build_lobby_players(&world);
            let msg = ServerMessage::LobbyState {
                players: lobby_players,
            };
            let _ = state.broadcast_tx.send(encode(&msg));
        }
        ClientMessage::SetDifficulty { difficulty } => {
            let mut world = state.world.lock().await;
            // Only allow setting difficulty in lobby, first player decides
            if matches!(world.round_state, RoundState::Lobby) {
                world.difficulty = difficulty;
            }
        }
        ClientMessage::Ready => {
            let mut world = state.world.lock().await;
            if !matches!(world.round_state, RoundState::Lobby) {
                return;
            }

            world.player_ready.insert(player_id);

            let lobby_players = build_lobby_players(&world);
            let msg = ServerMessage::LobbyState {
                players: lobby_players.clone(),
            };
            let _ = state.broadcast_tx.send(encode(&msg));

            // Check if all non-solo players are ready (need at least 2)
            let total_active = world.player_names.keys()
                .filter(|pid| !world.player_solo.contains(pid))
                .count();
            let total_ready = world.player_ready.len();
            if total_active >= 2 && total_ready == total_active {
                world.round_state = RoundState::Countdown(3.0);
                let msg = ServerMessage::CountdownStart { seconds: 3 };
                let _ = state.broadcast_tx.send(encode(&msg));
            }
        }
        ClientMessage::UpdateScore { coins } => {
            let mut world = state.world.lock().await;
            world.player_coins.insert(player_id, coins);
            let msg = ServerMessage::ScoreUpdate {
                player_id,
                coins,
            };
            let _ = state.broadcast_tx.send(encode(&msg));
        }
        ClientMessage::EnterSolo => {
            let mut world = state.world.lock().await;
            world.player_solo.insert(player_id);
            world.player_ready.remove(&player_id);

            let lobby_players = build_lobby_players(&world);
            let msg = ServerMessage::LobbyState {
                players: lobby_players,
            };
            let _ = state.broadcast_tx.send(encode(&msg));
        }
        ClientMessage::LeaveSolo => {
            let mut world = state.world.lock().await;
            world.player_solo.remove(&player_id);

            let lobby_players = build_lobby_players(&world);
            let msg = ServerMessage::LobbyState {
                players: lobby_players,
            };
            let _ = state.broadcast_tx.send(encode(&msg));
        }
    }
}

async fn cleanup_player(player_id: u32, state: &AppState) {
    let mut world = state.world.lock().await;
    world.remove_player(player_id);
    let msg = ServerMessage::PlayerLeft { player_id };
    let _ = state.broadcast_tx.send(encode(&msg));

    // If we're in Lobby, broadcast updated lobby state
    if matches!(world.round_state, RoundState::Lobby) {
        let lobby_players = build_lobby_players(&world);
        let msg = ServerMessage::LobbyState {
            players: lobby_players,
        };
        let _ = state.broadcast_tx.send(encode(&msg));
    }
}
