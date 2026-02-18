mod beacon;
mod game_loop;
mod state;

use axum::Router;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use futures_util::{SinkExt, StreamExt};
use tafels_shared::protocol::{ClientMessage, ServerMessage, decode, encode};

use crate::state::AppState;

#[tokio::main]
async fn main() {
    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    let state = AppState::new();

    // Spawn game loop
    let loop_state = state.clone();
    tokio::spawn(game_loop::run_game_loop(loop_state));

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await.unwrap();
    println!("tafels-server listening on {bind_addr}");
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
    let (player_id, snapshot_bytes) = {
        let mut world = state.world.lock().await;
        let player_state = world.add_player();
        let player_id = player_state.player_id;

        let snapshot = ServerMessage::WorldSnapshot {
            players: world.players.values().copied().collect(),
            beacons: world.beacons.values().cloned().collect(),
        };
        let snapshot_bytes = encode(&snapshot);

        // Broadcast PlayerJoined to others
        let joined = ServerMessage::PlayerJoined(player_state);
        let _ = state.broadcast_tx.send(encode(&joined));

        (player_id, snapshot_bytes)
    };

    println!("Player {player_id} connected");

    // Send WorldSnapshot to the new player
    if ws_tx.send(Message::Binary(snapshot_bytes)).await.is_err() {
        cleanup_player(player_id, &state).await;
        return;
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
                let msg = ServerMessage::BeaconResolved {
                    beacon_id,
                    claimed_by: player_id,
                };
                let _ = state.broadcast_tx.send(encode(&msg));
            }
        }
    }
}

async fn cleanup_player(player_id: u32, state: &AppState) {
    let mut world = state.world.lock().await;
    world.remove_player(player_id);
    let msg = ServerMessage::PlayerLeft { player_id };
    let _ = state.broadcast_tx.send(encode(&msg));
}
