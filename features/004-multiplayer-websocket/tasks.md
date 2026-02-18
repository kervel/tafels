---
feature: 004-multiplayer-websocket
generated_from: plan.md, spec.md, data-model.md, contracts/protocol.md
---

# Tasks: Multiplayer via WebSocket

<!-- Tech Stack Validation: PASSED -->
<!-- Validated against: .specswarm/tech-stack.md -->
<!-- New dependencies: ewebsock, axum, tokio, postcard, serde (justified in research.md) -->

## Phase 1: Workspace Restructure (Setup)

> **Goal**: Convert single-crate project to Cargo workspace with client/server/shared members. No behavior change.

### [X] T001 — Create workspace Cargo.toml and move client crate
- **File(s)**: `Cargo.toml` (root, new), `client/Cargo.toml` (moved)
- **Action**:
  1. Create root `Cargo.toml` with `[workspace] members = ["client", "server", "shared"]`
  2. Create `client/` directory
  3. Move `src/`, `assets/`, `index.html`, `Trunk.toml` into `client/`
  4. Move current root `Cargo.toml` to `client/Cargo.toml`
  5. Update `client/Cargo.toml`: change `name = "tafels-client"`, add `tafels-shared = { path = "../shared" }` to deps
  6. Update `[[bin]]` path in `client/Cargo.toml` to `path = "src/main.rs"`
- **Depends on**: —
- **Done when**: `cargo build -p tafels-client` compiles (native)

### [X] T002 — Create shared crate skeleton
- **File(s)**: `shared/Cargo.toml`, `shared/src/lib.rs`
- **Action**:
  1. Create `shared/Cargo.toml` with name `tafels-shared`, deps: `serde = { version = "1", features = ["derive"] }`, `postcard = { version = "1", features = ["alloc"] }`, `rand = "0.8"`, `noise = "0.9"`, `glam = { version = "0.29", features = ["serde"] }`
  2. Create `shared/src/lib.rs` with `pub mod protocol;` `pub mod difficulty;` `pub mod exercise;` `pub mod heightmap;` `pub mod character_state;`
  3. Create empty module files for each
- **Depends on**: T001
- **Done when**: `cargo build -p tafels-shared` compiles

### [X] T003 — Create server crate skeleton
- **File(s)**: `server/Cargo.toml`, `server/src/main.rs`
- **Action**:
  1. Create `server/Cargo.toml` with name `tafels-server`, deps: `axum = { version = "0.7", features = ["ws"] }`, `tokio = { version = "1", features = ["full"] }`, `tafels-shared = { path = "../shared" }`, `postcard = { version = "1", features = ["alloc"] }`, `serde = "1"`, `rand = "0.8"`, `noise = "0.9"`
  2. Create `server/src/main.rs` with a minimal `#[tokio::main] async fn main()` placeholder
- **Depends on**: T001
- **Done when**: `cargo build -p tafels-server` compiles

### [X] T004 — Extract Difficulty to shared crate [P]
- **File(s)**: `shared/src/difficulty.rs`, `client/src/game/difficulty.rs`
- **Action**:
  1. Copy `Difficulty` enum and its methods from `client/src/game/difficulty.rs` to `shared/src/difficulty.rs`
  2. Add `#[derive(serde::Serialize, serde::Deserialize)]` to `Difficulty`
  3. Remove Bevy `Component`/`Resource` derives if present (keep only Copy, Clone, Debug, PartialEq, Eq, serde derives)
  4. In `client/src/game/difficulty.rs`: replace the enum definition with `pub use tafels_shared::difficulty::Difficulty;` and keep any client-specific impls (Bevy Component derive) as a local wrapper or newtype if needed
- **Depends on**: T002
- **Done when**: Client compiles and uses shared `Difficulty`

### [X] T005 — Extract exercise types to shared crate [P]
- **File(s)**: `shared/src/exercise.rs`, `client/src/game/exercise.rs`
- **Action**:
  1. Move `Operation`, `ExerciseState` enums and `generate_exercise()` function to `shared/src/exercise.rs`
  2. Add serde derives
  3. Drop any Bevy-specific derives
  4. Client re-exports or imports from `tafels_shared::exercise`
- **Depends on**: T002
- **Done when**: Client compiles, exercise generation works identically

### [X] T006 — Extract heightmap to shared crate [P]
- **File(s)**: `shared/src/heightmap.rs`, `client/src/terrain/heightmap.rs`
- **Action**:
  1. Move `HeightmapData`, `generate_heightmap()`, `sample_height()` to `shared/src/heightmap.rs`
  2. These should already be pure Rust (no Bevy deps) — verify
  3. Client imports from `tafels_shared::heightmap`
  4. Ensure heightmap generation is deterministic (seed 42 produces identical output)
- **Depends on**: T002
- **Done when**: Client terrain renders identically, heightmap generation is shared

### [X] T007 — Extract CharacterState to shared crate [P]
- **File(s)**: `shared/src/character_state.rs`, `client/src/character/mod.rs`
- **Action**:
  1. Move `CharacterState` enum to `shared/src/character_state.rs`
  2. Add serde derives, drop Bevy Component derive
  3. Client wraps or re-exports, adds Component derive client-side
- **Depends on**: T002
- **Done when**: Client compiles with shared CharacterState

### [X] T008 — Define protocol types in shared crate
- **File(s)**: `shared/src/protocol.rs`
- **Action**:
  1. Define `AnimationState` enum: `Idle`, `Walking`, `Running` (with serde derives)
  2. Define `PlayerState` struct: `player_id: u32`, `x: f32`, `y: f32`, `z: f32`, `yaw: f32`, `animation: AnimationState`
  3. Define `BeaconInfo` struct: `beacon_id: u32`, `x: f32`, `y: f32`, `z: f32`, `question_text: String`, `choices: [u32; 4]`, `correct_index: u8`, `lifetime: f32`
  4. Define `ServerMessage` enum: `WorldSnapshot`, `PlayerJoined`, `PlayerUpdate`, `PlayerLeft`, `BeaconSpawned`, `BeaconResolved`, `BeaconExpired`
  5. Define `ClientMessage` enum: `UpdateState`, `AnswerBeacon`
  6. Add helper functions `encode<T: Serialize>(msg: &T) -> Vec<u8>` and `decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T>`
  7. Add unit tests for round-trip serialization of each message type
- **Depends on**: T002
- **Done when**: `cargo test -p tafels-shared` passes, all message types round-trip

### [X] T009 — Update CI deploy workflow
- **File(s)**: `.github/workflows/deploy.yml`
- **Action**:
  1. Update `trunk build` path to run from `client/` subdirectory
  2. Update asset copy paths
  3. Ensure WASM build still works with workspace layout
- **Depends on**: T001
- **Done when**: CI pipeline builds WASM client from workspace

### [X] T010 — Verify full client build (native + WASM)
- **File(s)**: —
- **Action**:
  1. Run `cargo build -p tafels-client` (native)
  2. Run `cd client && trunk build` (WASM)
  3. Run `cargo build -p tafels-server`
  4. Verify game runs identically in both modes
- **Depends on**: T004, T005, T006, T007, T008, T009
- **Checkpoint**: Phase 1 complete. Workspace compiles, game behavior unchanged.

## Phase 2: Game Server — Core (US1: Joining a Game Session)

> **Goal**: Server accepts WebSocket connections, assigns player IDs, sends WorldSnapshot, broadcasts join/leave. Clients can connect and see each other exist.

### [X] T011 — Server WebSocket handler and AppState
- **File(s)**: `server/src/main.rs`, `server/src/state.rs`
- **Action**:
  1. Create `AppState` with `Arc<Mutex<GameWorld>>` and `broadcast::Sender<Vec<u8>>`
  2. `GameWorld` holds: `players: HashMap<u32, PlayerState>`, `beacons: HashMap<u32, BeaconInfo>`, `next_player_id: u32`, `next_beacon_id: u32`
  3. axum router: `GET /ws` → WebSocket upgrade, `GET /health` → 200 OK
  4. WebSocket handler: assign player_id, send `WorldSnapshot`, broadcast `PlayerJoined` to others
  5. Spawn reader + writer tasks per connection
  6. On disconnect: remove player, broadcast `PlayerLeft`
  7. Bind address from `BIND_ADDR` env var (default `0.0.0.0:3000`)
- **Depends on**: T008
- **Done when**: Server starts, accepts WebSocket connections, logs join/leave

### [X] T012 — Server game loop (20 Hz tick)
- **File(s)**: `server/src/game_loop.rs`
- **Action**:
  1. Spawn a `tokio::time::interval(Duration::from_millis(50))` task
  2. Each tick: collect all player states, broadcast `PlayerUpdate(Vec<PlayerState>)` to all clients
  3. Only include players whose state changed since last broadcast (optimization)
  4. Wire into AppState — game loop reads `GameWorld`, broadcasts via `broadcast::Sender`
- **Depends on**: T011
- **Done when**: Server broadcasts player updates at 20 Hz

### [X] T013 — Server receives and applies UpdateState
- **File(s)**: `server/src/main.rs` (reader task)
- **Action**:
  1. In the reader task: deserialize incoming binary messages as `ClientMessage`
  2. On `UpdateState(PlayerState)`: update player's position/yaw/animation in `GameWorld`
  3. Ignore messages from unknown player_ids
- **Depends on**: T011
- **Done when**: Server updates player positions from client messages

## Phase 3: Client Networking (US1 + US6: Connection & Fallback)

> **Goal**: Client connects via WebSocket, sends position updates, receives server messages. Falls back to single-player if server is unreachable.

### [X] T014 — Add ewebsock dependency and NetworkPlugin skeleton
- **File(s)**: `client/Cargo.toml`, `client/src/network/mod.rs`, `client/src/main.rs`
- **Action**:
  1. Add `ewebsock = "0.8"` to `client/Cargo.toml`
  2. Create `client/src/network/mod.rs` with `NetworkPlugin`
  3. Define `ConnectionState` resource enum: `Disconnected`, `Connecting`, `Connected { sender, receiver, my_player_id }`, `Reconnecting { attempt, next_try }`
  4. Define server URL compile-time constant (e.g., `const SERVER_URL: &str = "ws://localhost:3000/ws";`)
  5. Register plugin in `main.rs`
- **Depends on**: T010
- **Done when**: Client compiles with NetworkPlugin registered

### [X] T015 — Connect system (on entering Playing state)
- **File(s)**: `client/src/network/mod.rs`
- **Action**:
  1. System triggered `OnEnter(GameState::Playing)` (or equivalent state transition)
  2. Call `ewebsock::connect(SERVER_URL, ewebsock::Options::default())` to get `(WsSender, WsReceiver)`
  3. If connect succeeds: set `ConnectionState::Connecting`
  4. If connect fails immediately: set `ConnectionState::Disconnected`, log warning
  5. Cleanup system on `OnExit(GameState::Playing)`: close connection, reset to Disconnected
- **Depends on**: T014
- **Done when**: Client attempts WebSocket connection when entering Playing state

### [X] T016 — Receive system (poll messages each frame)
- **File(s)**: `client/src/network/mod.rs`
- **Action**:
  1. System runs each frame when `ConnectionState::Connected` or `Connecting`
  2. Poll `receiver.try_recv()` in a loop (drain all pending messages)
  3. Deserialize binary messages as `ServerMessage`
  4. On `WorldSnapshot`: store `my_player_id`, set state to `Connected`, emit Bevy events for each player/beacon
  5. Convert to Bevy events: `RemotePlayerJoined(PlayerState)`, `RemotePlayerUpdated(Vec<PlayerState>)`, `RemotePlayerLeft { player_id }`, `BeaconSpawnedEvent(BeaconInfo)`, `BeaconResolvedEvent { beacon_id, claimed_by }`, `BeaconExpiredEvent { beacon_id }`
  6. On WsEvent::Closed/Error: set `ConnectionState::Reconnecting`
- **Depends on**: T015
- **Done when**: Client receives and dispatches server messages as Bevy events

### [X] T017 — Send system (10 Hz local state)
- **File(s)**: `client/src/network/mod.rs`
- **Action**:
  1. System runs with a `Timer` at 100ms interval
  2. Read local player's `Transform` and `CharacterState`
  3. Build `ClientMessage::UpdateState(PlayerState { ... })`
  4. Serialize with postcard, send as binary via `sender.send(WsMessage::Binary(bytes))`
  5. Only send when `ConnectionState::Connected`
- **Depends on**: T015
- **Done when**: Client sends position updates to server at 10 Hz

### [X] T018 — Single-player fallback and reconnection
- **File(s)**: `client/src/network/mod.rs`
- **Action**:
  1. If connection fails or times out (5 sec): keep existing client-side beacon/exercise logic active
  2. Exponential backoff reconnection: 1s, 2s, 4s, 8s, max 30s
  3. Define `is_online() -> bool` helper method on `ConnectionState`
  4. Existing beacon spawning system gated behind `!connection_state.is_online()`
- **Depends on**: T016
- **Done when**: Game is fully playable without server, reconnects automatically

### [X] T019 — HUD online/offline indicator
- **File(s)**: `client/src/hud/mod.rs` (or wherever HUD is)
- **Action**:
  1. Small indicator in corner showing connection status
  2. Green dot + "Online" when connected, red dot + "Offline" when disconnected
  3. Updates reactively from `ConnectionState` resource
- **Depends on**: T018
- **Done when**: HUD shows connection status

## Phase 4: Remote Player Rendering (US2: Seeing Other Players)

> **Goal**: Remote players appear as characters with smooth movement, nameplates, and collision.

### [X] T020 — Spawn/despawn remote player entities
- **File(s)**: `client/src/network/remote_players.rs`
- **Action**:
  1. Create `RemotePlayer { player_id: u32 }` component
  2. On `RemotePlayerJoined` event: spawn entity with same GLTF character model, `RemotePlayer` component, initial Transform from PlayerState
  3. On `RemotePlayerLeft` event: query for entity with matching player_id, despawn recursively
  4. On `WorldSnapshot` (initial connect): spawn all existing remote players
  5. Only spawn/despawn during `GameState::Playing`
- **Depends on**: T016
- **Done when**: Remote player entities appear and disappear correctly

### [X] T021 — Position interpolation for remote players
- **File(s)**: `client/src/network/remote_players.rs`
- **Action**:
  1. Add `InterpolationState` component: `prev_pos: Vec3`, `target_pos: Vec3`, `prev_yaw: f32`, `target_yaw: f32`, `timer: f32`
  2. On `RemotePlayerUpdated`: update `prev_pos = current`, `target_pos = new`, reset timer
  3. Each frame: lerp between prev and target based on elapsed time / expected interval (50ms)
  4. Apply yaw rotation smoothly
  5. Apply matching animation state from `AnimationState`
- **Depends on**: T020
- **Done when**: Remote players move smoothly without teleportation

### [X] T022 — Remote player animation
- **File(s)**: `client/src/network/remote_players.rs`
- **Action**:
  1. When spawning remote player, set up animation player (same as local player's animation setup)
  2. On receiving AnimationState change: transition to matching animation clip (Idle, Walking, Running)
  3. Reuse existing animation transition logic from `client/src/character/animation.rs`
- **Depends on**: T021
- **Done when**: Remote players play correct walk/run/idle animations

### [X] T023 — Floating nameplate above remote players
- **File(s)**: `client/src/network/remote_players.rs`
- **Action**:
  1. Spawn a `Text2d` (or billboard text) as child of each remote player entity
  2. Display "Player {player_id}" text
  3. Position above character head (y offset ~2.0)
  4. Billboard behavior: always face camera (use Bevy's billboard or manual camera-facing transform)
- **Depends on**: T020
- **Done when**: Nameplates visible above remote players, always face camera

### [X] T024 — Remote player collision (soft push-back) [P]
- **File(s)**: `client/src/collision/mod.rs`
- **Action**:
  1. Add `RemotePlayer` entities to the collision system (same as vegetation colliders)
  2. Soft push-back when local player overlaps remote player (same radius as tree/rock collision)
  3. Reuse existing `VegetationCollider` logic or generalize it
- **Depends on**: T020
- **Done when**: Local player is pushed back when walking into remote players

## Phase 5: Server-Managed Beacons (US5: Racing to Exercise Beacons)

> **Goal**: Server spawns beacons, clients display them, first correct answer wins.

### [X] T025 — Server beacon spawning logic
- **File(s)**: `server/src/game_loop.rs`, `server/src/beacon.rs`
- **Action**:
  1. Port beacon placement algorithm from client's `spawn_beacon` system
  2. Server generates heightmap (seed 42, same params) and vegetation positions for exclusion zones
  3. Spawn beacons at intervals (same timing as current client logic)
  4. Use shared `generate_exercise()` to create question/choices
  5. Broadcast `BeaconSpawned(BeaconInfo)` to all clients
  6. Track beacon lifetime, broadcast `BeaconExpired` when time runs out
- **Depends on**: T012, T008
- **Done when**: Server spawns beacons with exercises, broadcasts to clients

### [X] T026 — Server beacon answer handling
- **File(s)**: `server/src/beacon.rs`
- **Action**:
  1. On `ClientMessage::AnswerBeacon { beacon_id, correct: true }`: check beacon exists and not already resolved
  2. First correct answer wins: mark beacon as Resolved, broadcast `BeaconResolved { beacon_id, claimed_by }`
  3. Ignore late answers for already-resolved beacons
  4. On player disconnect while near beacon: beacon returns to Dormant (no special handling needed — beacons don't track "interacting player")
- **Depends on**: T025, T013
- **Done when**: First-correct-answer-wins logic works, broadcasts to all clients

### [X] T027 — Client: display server-spawned beacons
- **File(s)**: `client/src/game/beacon.rs`
- **Action**:
  1. On `BeaconSpawnedEvent`: spawn beacon visual at server-specified position with server-provided exercise data
  2. Store `beacon_id` on the beacon entity (new component or extend existing)
  3. On `BeaconResolvedEvent`: transition beacon to faded visual stub (reduce opacity/scale), despawn after 3s delay
  4. On `BeaconExpiredEvent`: play dissolve/fade effect, despawn
  5. Gate existing client-side `spawn_beacon` system behind `!connection_state.is_online()`
- **Depends on**: T016, T018
- **Done when**: Beacons from server appear/resolve/expire correctly on client

### [X] T028 — Client: beacon interaction sends answer to server
- **File(s)**: `client/src/game/beacon.rs`, `client/src/game/exercise.rs`
- **Action**:
  1. When local player enters beacon proximity in multiplayer mode: show exercise panel using server-provided question/choices
  2. On answer: send `ClientMessage::AnswerBeacon { beacon_id, correct }` to server
  3. Wait for `BeaconResolved` confirmation before awarding coins locally
  4. If in single-player mode (offline): use existing client-side exercise logic unchanged
- **Depends on**: T027, T017
- **Done when**: Player can answer beacons, server confirms, coins awarded on confirmation

## Phase 6: Deployment

> **Goal**: Server runs on Titan Kubernetes cluster with health checks and TLS.

### [X] T029 — Dockerfile for server [P]
- **File(s)**: `server/Dockerfile`
- **Action**:
  1. Multi-stage build: `rust:latest` for build, `debian:bookworm-slim` for runtime
  2. Build `tafels-server` binary with release profile
  3. Copy binary to runtime image
  4. Expose port 3000
  5. `CMD ["./tafels-server"]`
- **Depends on**: T011
- **Done when**: `docker build -t tafels-server server/` succeeds

### [X] T030 — Kubernetes manifests [P]
- **File(s)**: `server/k8s/deployment.yaml`, `server/k8s/service.yaml`, `server/k8s/ingress.yaml`
- **Action**:
  1. Deployment: 1 replica, container from tafels-server image, env `BIND_ADDR=0.0.0.0:3000`, liveness probe on `/health`, resource limits
  2. Service: ClusterIP, port 3000
  3. Ingress: TLS termination, WebSocket upgrade support (`nginx.ingress.kubernetes.io/proxy-read-timeout: "3600"`, `proxy-send-timeout: "3600"`)
- **Depends on**: T029
- **Done when**: Manifests are valid YAML, deployable to Titan cluster

## Phase 7: Polish & Integration

### [X] T031 — Client server URL configuration
- **File(s)**: `client/src/network/mod.rs`
- **Action**:
  1. Define `SERVER_URL` as compile-time constant
  2. For dev: `ws://localhost:3000/ws`
  3. For prod: `wss://<titan-ingress-domain>/ws`
  4. Consider reading from `index.html` meta tag or env var for flexibility
- **Depends on**: T014
- **Done when**: Client connects to correct server URL for dev and prod

### [X] T032 — Remote player distance culling [P]
- **File(s)**: `client/src/network/remote_players.rs`
- **Action**:
  1. Apply same fog-distance culling as vegetation to remote player entities
  2. Hide remote players beyond render distance
- **Depends on**: T020
- **Done when**: Remote players fade/hide at fog distance

### T033 — End-to-end integration test
- **File(s)**: —
- **Action**:
  1. Start server locally
  2. Launch two client instances (native or browser)
  3. Verify: players see each other (US1), movement is smooth (US2), beacons appear for both (US5), first answer wins (US5), disconnect removes player (US4), offline mode works (US6)
- **Depends on**: T028, T024, T023
- **Checkpoint**: Feature complete. All user scenarios verified.

---

## Dependencies

```
T001 ──┬── T002 ──┬── T004 [P] ─┐
       │          ├── T005 [P] ──┤
       │          ├── T006 [P] ──┤
       │          ├── T007 [P] ──┤
       │          └── T008 ──────┤
       ├── T003                  │
       └── T009                  │
                                 ▼
                    T010 (verify all) ── CHECKPOINT: Phase 1
                         │
              ┌──────────┴──────────┐
              ▼                     ▼
         T011 (server)         T014 (client net)
              │                     │
         T012, T013            T015 → T016 → T017 → T018 → T019
              │                              │
              │                     ┌────────┴────────┐
              │                     ▼                  ▼
              │                T020 (spawn)       T027 (beacons)
              │                  │                     │
              │          T021, T022, T023, T024   T028 (interaction)
              │                                        │
         T025 (server beacons)                         │
              │                                        │
         T026 ─────────────────────────────────────────┘
              │
         T029, T030 [P] (deployment)
              │
         T031, T032 [P] (polish)
              │
         T033 (integration test) ── CHECKPOINT: Feature complete
```

## Parallel Execution Opportunities

**Phase 1 (after T002)**: T004, T005, T006, T007 can all run in parallel (different files, independent extractions)

**Phase 2 + Phase 3**: T011 (server) and T014 (client networking) can start in parallel after T010

**Phase 4**: T023 (nameplates) and T024 (collision) can run in parallel with T021/T022 (interpolation/animation)

**Phase 6**: T029 (Dockerfile) and T030 (K8s manifests) can run in parallel

**Phase 7**: T031 (URL config) and T032 (culling) can run in parallel

## Implementation Strategy

**MVP (Phases 1-3)**: Workspace restructured, server accepts connections, client connects and sends/receives. Players can join/leave. Single-player fallback works. This is the minimum to validate the architecture.

**Multiplayer Visual (Phase 4)**: Remote players visible with smooth movement. This is the first "wow" moment — two browser windows showing each other's characters.

**Competitive Gameplay (Phase 5)**: Server-managed beacons with racing. This completes the core multiplayer experience.

**Production (Phase 6-7)**: Deployment and polish. Ship to Titan cluster.
