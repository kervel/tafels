# Research: Multiplayer via WebSocket

## Decision 1: WebSocket Client Library

**Decision:** ewebsock v0.8

**Rationale:** ewebsock provides a non-blocking polling API (`try_recv()`) that integrates naturally with Bevy's synchronous game loop. It works on both wasm32-unknown-unknown (browser WebSocket via web-sys) and native (tungstenite backend) with zero `#[cfg]` guards in game code. Store the `(WsSender, WsReceiver)` as a Bevy Resource and poll in a regular system. Maintained by Rerun.io as their own production dependency.

**Alternatives considered:**
- **tokio-tungstenite-wasm**: Async API requires spawning tasks + channels. More boilerplate, but viable if networking logic grows complex.
- **bevy_matchbox**: P2P WebRTC — wrong architecture for client-server model.
- **lightyear**: Full multiplayer framework with prediction/rollback. Massively over-engineered for a 20-player casual game.
- **bevy_simplenet**: No Bevy 0.18 support. Heavyweight dependency tree.
- **Raw tungstenite + gloo-net split**: Reinventing the cross-platform abstraction.

## Decision 2: WebSocket Server Framework

**Decision:** axum 0.7 with WebSocket support

**Rationale:** First-class `WebSocketUpgrade` extractor. Ergonomic `State<T>` injection for shared game state. Built on tokio/hyper — standard Rust async ecosystem. TLS via axum-server + rustls for pure-Rust single binary deployment. `tokio::sync::broadcast` channel for broadcasting to all clients.

**Alternatives considered:**
- **Raw tokio-tungstenite**: More boilerplate (manual router, TLS setup). No benefit for 20 connections.
- **warp**: Filter combinators create friction for stateful WebSocket connections. Community has moved to axum.
- **actix-web**: Actor model is overkill. Own runtime layer adds indirection.

## Decision 3: Serialization Format

**Decision:** postcard v1

**Rationale:** Stable wire format, serde-based, 30% smaller than bincode (varint encoding), no_std compatible for the shared crate, works on both WASM and native. PlayerState encodes to ~18 bytes.

**Alternatives considered:**
- **JSON**: 3-5x larger for numeric data. 400 encode/decode cycles/sec at 20Hz × 20 players.
- **bincode v2**: Fastest but unstable wire format (still rc-phase). v1 works but postcard is smaller.

## Decision 4: Tick Rate Architecture

**Decision:** Server broadcasts at 20 Hz, clients send at 10 Hz

**Rationale:** 20 Hz server→client gives 50ms between updates — smooth with lerp interpolation. 10 Hz client→server halves inbound bandwidth. Bandwidth estimate: 20 players × 18 bytes × 20 Hz = ~7 KB/s outbound per client (within 5 KB/s SC-6 budget after accounting for only sending changed state).

## Decision 5: Shared Crate Architecture

**Decision:** Convert to Cargo workspace with `client`, `server`, `shared` members

**Rationale:** The shared crate contains protocol types, exercise generation, heightmap generation, and `sample_height` — all pure Rust with no Bevy dependency. Both client and server depend on it. The heightmap is deterministic (seed 42, fixed params) so the server can regenerate the identical terrain.

**Types to extract to shared crate:**
- `Difficulty` enum + methods (zero Bevy deps)
- `Operation`, `ExerciseState` enums (drop `Component` derive)
- `generate_exercise()` function (only needs `rand`)
- `HeightmapData` struct, `generate_heightmap()`, `sample_height()` (only needs `noise`)
- `CharacterState` enum (drop `Component` derive, add `Serialize`/`Deserialize`)
- New protocol types: `PlayerState`, `ServerMessage`, `ClientMessage`, `BeaconInfo`

**Shared crate dependencies:** `serde`, `postcard`, `rand = "0.8"`, `noise = "0.9"`, `glam`

## Decision 6: Bevy Integration Pattern

**Decision:** Polling pattern with ewebsock Resource

**Rationale:** Store `(WsSender, WsReceiver)` in a Bevy `Resource`. A receive system calls `try_recv()` each frame and converts messages to Bevy events. A send system serializes local player state and calls `sender.send()` at the configured tick rate (10 Hz via a `Timer`). No async runtime needed in client code.
