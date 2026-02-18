# Implementation Plan: Multiplayer via WebSocket

## Technical Context

- **Language**: Rust (stable, edition 2024)
- **Engine**: Bevy 0.18
- **Client targets**: wasm32-unknown-unknown (browser), native (desktop)
- **Server target**: x86_64-linux (Kubernetes on Titan cluster)
- **WebSocket client**: ewebsock v0.8 (polling API, WASM+native)
- **WebSocket server**: axum 0.7 + tokio
- **Serialization**: postcard v1 (binary, serde-based)
- **Deployment**: Kubernetes with ingress controller (Titan cluster)

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| Idiomatic Rust | PASS | All shared types use standard Rust patterns |
| ECS-First Architecture | PASS | Client networking via Resources + Systems. Server is non-Bevy. |
| Performance-Aware Design | PASS | 20 Hz server tick, 10 Hz client send, postcard binary encoding, ~7 KB/s bandwidth |
| Modularity and Separation | PASS | Three crates: shared, client, server. Clean plugin boundaries. |
| Readable Over Clever | PASS | Simple polling pattern, no async complexity in game code |

## Architecture Overview

```
┌─────────────────┐    WebSocket (postcard binary)    ┌──────────────┐
│  Game Client     │◄────────────────────────────────►│  Game Server  │
│  (Bevy 0.18)     │  UpdateState @ 10 Hz (client→)   │  (axum+tokio) │
│  WASM or Native  │  PlayerUpdate @ 20 Hz (←server)   │  Kubernetes   │
│                  │  BeaconSpawned/Resolved (←server) │              │
└─────────────────┘                                    └──────────────┘
         │                                                     │
         └──────────── both depend on ─────────────────────────┘
                           │
                    ┌──────┴───────┐
                    │ tafels-shared │
                    │ (no Bevy dep) │
                    │ protocol.rs   │
                    │ exercise.rs   │
                    │ heightmap.rs  │
                    │ difficulty.rs │
                    └──────────────┘
```

## Implementation Phases

### Phase 1: Workspace Restructure

Convert the single-crate project to a Cargo workspace with three members.

**1.1 Create workspace layout**
- Create `shared/`, `server/` directories
- Move current `src/`, `assets/`, `index.html`, `Trunk.toml` into `client/`
- Create workspace `Cargo.toml` at root
- Update `client/Cargo.toml` (formerly the root Cargo.toml)
- Create `shared/Cargo.toml` and `server/Cargo.toml`
- Update `.github/workflows/deploy.yml` to build from `client/` subdirectory
- Verify `cargo build` and `trunk serve` still work

**1.2 Extract shared types to `tafels-shared`**
- Move `Difficulty` enum + methods from `src/game/difficulty.rs`
- Move `Operation`, `ExerciseState` enums from `src/game/exercise.rs`
- Move `generate_exercise()` function (drop Bevy `Component` derives)
- Move `HeightmapData`, `generate_heightmap()`, `sample_height()` from `src/terrain/heightmap.rs`
- Move `CharacterState` enum from `src/character/mod.rs` (drop `Component` derive)
- Add `serde::Serialize`/`Deserialize` derives to all shared types
- Client imports these from `tafels_shared` instead of local modules
- Verify client builds on both native and WASM
- Verify native game runs identically (no behavior change)

**1.3 Define protocol types in shared crate**
- Create `shared/src/protocol.rs` with `PlayerState`, `BeaconInfo`, `ServerMessage`, `ClientMessage`
- Add `postcard` encode/decode helpers
- Unit tests for round-trip serialization

### Phase 2: Game Server

**2.1 Server skeleton**
- axum app with WebSocket upgrade handler
- `AppState` with `broadcast::Sender` and `Arc<Mutex<GameWorld>>`
- Connection handler: assign player_id, send WorldSnapshot, spawn reader/writer tasks
- Disconnect cleanup: remove player, broadcast PlayerLeft

**2.2 Server game loop**
- 20 Hz tick loop via `tokio::time::interval`
- Regenerate heightmap (seed 42, same params as client)
- Generate vegetation positions (same seed as client) for beacon placement exclusion
- Beacon spawning logic (port placement algorithm from client `spawn_beacon`)
- Beacon lifetime tick and expiry
- Broadcast batched PlayerUpdate + beacon events each tick

**2.3 Beacon interaction**
- Receive `AnswerBeacon { beacon_id, correct: true }` from client
- Mark beacon as Resolved, broadcast `BeaconResolved` to all clients
- First-correct-answer-wins: ignore late answers for already-resolved beacons

**2.4 Kubernetes deployment**
- Dockerfile for server binary
- Kubernetes Deployment + Service + Ingress manifests
- Health check endpoint (GET /health)
- Environment variable for bind address/port

### Phase 3: Client Networking

**3.1 WebSocket connection**
- Add `ewebsock` dependency to client
- `NetworkPlugin` Bevy plugin
- `WsConnection` Resource holding `(WsSender, WsReceiver)`
- Startup system: connect to server URL on entering Playing state
- Cleanup system: disconnect on exiting Playing state

**3.2 Receive system**
- Poll `receiver.try_recv()` each frame
- Convert `ServerMessage` variants to Bevy events:
  - `RemotePlayerJoined`, `RemotePlayerUpdated`, `RemotePlayerLeft`
  - `BeaconSpawned`, `BeaconResolved`, `BeaconExpired`

**3.3 Send system**
- Timer-based (10 Hz) send of local player's `PlayerState`
- Serialize with postcard, send as binary WebSocket message

**3.4 Single-player fallback**
- If WebSocket connection fails, keep existing client-side beacon/exercise logic
- Subtle HUD indicator showing "offline" / "online" status
- Reconnect with exponential backoff

### Phase 4: Remote Player Rendering

**4.1 Spawn/despawn remote characters**
- On `RemotePlayerJoined`: spawn character model (same GLTF as local player) with `RemotePlayer { player_id }` component
- On `RemotePlayerLeft`: despawn the entity

**4.2 Interpolation**
- Store last two received positions per remote player
- Lerp between them based on time since last update
- Apply yaw rotation and animation state

**4.3 Nameplate**
- Floating text above remote characters showing "Player N"
- Billboard-style always-faces-camera

**4.4 Collision**
- Add `VegetationCollider`-equivalent for remote players
- Soft push-back (same logic as tree/rock collision)

### Phase 5: Server-Managed Beacons (Client Integration)

**5.1 Receive beacon events**
- On `BeaconSpawned`: spawn beacon visual at server-specified position with server-provided exercise data
- On `BeaconResolved`: transition beacon to faded visual stub, then despawn after delay
- On `BeaconExpired`: play dissolve effect, despawn

**5.2 Beacon interaction**
- When local player enters beacon proximity: show exercise panel (using server-provided question/choices)
- On answer: send `AnswerBeacon { beacon_id, correct }` to server
- Wait for `BeaconResolved` confirmation before awarding coins locally

**5.3 Remove client-side beacon spawning in multiplayer mode**
- Gate the current `spawn_beacon` system behind `!connected_to_server`
- In multiplayer mode, beacons come only from server events

## Deployment

- **Client WASM**: GitHub Pages (existing CI pipeline, updated for workspace)
- **Client native**: `cargo run -p tafels-client`
- **Server**: Docker image deployed to Titan Kubernetes cluster
- **Server URL**: Configured via compile-time constant in client (different for dev/prod)
- **Ingress**: TLS termination at ingress controller, WebSocket upgrade supported

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| Workspace migration breaks CI | Phase 1 includes CI verification before proceeding |
| ewebsock doesn't work on WASM | Fallback: tokio-tungstenite-wasm with async channel pattern |
| Server heightmap diverges from client | Shared crate ensures identical code; integration test compares outputs |
| Bandwidth exceeds 5 KB/s | Only send changed player states; postcard's varint keeps payloads small |
| Beacon race condition (two players answer simultaneously) | Server processes answers sequentially; first correct answer wins, late answers ignored |
