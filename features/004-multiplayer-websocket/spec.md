---
parent_branch: main
feature_number: 004
status: In Progress
created_at: 2026-02-18T00:00:00+01:00
---

# Feature: Multiplayer via WebSocket

## Overview

Enable multiple players to share the same 3D landscape simultaneously. Each player sees other players' characters moving in real time. Both WASM (browser, hosted on GitHub Pages) and native (desktop) clients connect to a separate game server via WebSocket. The server manages shared world state including player positions and exercise beacons. Players can race to reach beacons and answer exercises competitively. Native and WASM clients are fully interoperable.

## Clarifications

### Session 2026-02-18

- Q: When should remote players be visible? → A: Playing state only. Remote players are hidden during level selection menu.
- Q: When two players race to a beacon, what happens? → A: First player to answer correctly wins the beacon. The beacon transitions to a faded/resolved visual stub (rather than disappearing instantly) so other nearby players aren't confused.
- Q: Do remote players collide with the local player? → A: Yes, soft push collision (same as vegetation). Additionally, remote players display a floating nameplate above their character.
- Q: Should the server enforce answer correctness? → A: Trust the client. Client reports answer result to server. Simplest approach; cheating prevention is not a concern.
- Q: Should exercises be client-side or server-side? → A: Server-side. The server spawns and manages exercise beacons so all players see the same beacons at the same locations and can race to them.

## User Scenarios

### Scenario 1: Joining a Game Session

A player opens the game in their browser or launches the native build and selects a level. The client connects to the game server via WebSocket upon entering the Playing state. The player sees other currently-playing players' characters in the landscape with floating nameplates. No login or room selection is required — all connected players share one world.

### Scenario 2: Seeing Other Players Move

While playing, a player sees other players' characters walking and running across the terrain in real time. Other players' animations (idle, walk, run) match their movement. Character positions update smoothly. Remote players have soft collision push-back, preventing players from occupying the same space.

### Scenario 3: Player Joins Mid-Session

A new player connects while others are already playing. The new player immediately sees the current positions and states of all existing players, plus all active beacons in the world. Existing players see the new character appear.

### Scenario 4: Player Disconnects

When a player closes their browser or loses connection, their character disappears from all other players' screens within a few seconds. No stale ghost characters remain in the world. Any beacon the disconnecting player was actively interacting with returns to its dormant state.

### Scenario 5: Racing to Exercise Beacons

The server spawns exercise beacons at positions visible to all players. Multiple players can approach the same beacon. When a player enters the beacon's proximity, they see the exercise panel and can answer. The first player to answer correctly claims the beacon — it transitions to a faded visual stub and awards coins/score only to the winning player. Other players nearby see the beacon resolve and must find another one.

### Scenario 6: Server Unavailable

If the game server is unreachable (down, network error), the game falls back to single-player mode with client-side exercise generation. The player sees a subtle indicator that multiplayer is unavailable. The game is fully playable without the server.

## Functional Requirements

### Connection

- FR-1: The client connects to the game server via WebSocket when entering the Playing state.
- FR-2: The game server URL is configured in the client build (compile-time constant or HTML configuration). Same URL for native and WASM clients.
- FR-3: Connection failures fall back silently to single-player mode with client-side exercise generation.
- FR-4: The client automatically attempts to reconnect after disconnection (with backoff).
- FR-5: The WebSocket networking layer works identically on both WASM and native targets, using platform-appropriate WebSocket implementations.

### State Synchronization

- FR-6: Each client sends its player's position (x, y, z), rotation (yaw), and animation state (idle/walking/running) to the server at a fixed tick rate.
- FR-7: The server broadcasts each player's state to all other connected clients.
- FR-8: The server assigns each connection a unique player ID on connect.
- FR-9: On connect, the server sends the current state of all existing players and all active beacons to the new client.
- FR-10: On disconnect, the server notifies all remaining clients to remove that player.

### Exercise Beacons (Server-Managed)

- FR-11: The server generates and spawns exercise beacons at world positions, broadcasting their location, exercise content, and answer choices to all clients.
- FR-12: All connected clients see the same beacons at the same positions.
- FR-13: When a player enters a beacon's proximity and answers, the client sends the answer result (correct/incorrect) to the server.
- FR-14: On a correct answer, the server marks the beacon as resolved, notifies all clients, and credits the answering player.
- FR-15: Resolved beacons transition to a faded visual stub on all clients before being removed, so nearby players aren't confused by sudden disappearance.
- FR-16: If a player disconnects while interacting with a beacon, the beacon returns to dormant state.

### Rendering Remote Players

- FR-17: Remote players are rendered using the same character model and animations as the local player.
- FR-18: Remote player positions are interpolated between updates to produce smooth movement.
- FR-19: Remote players are subject to the same distance culling as vegetation (hidden beyond fog distance).
- FR-20: Remote players display a floating nameplate above their character.
- FR-21: Remote players have soft collision push-back against the local player (same behavior as vegetation collision).
- FR-22: Remote players are only visible during the Playing game state.

### Server

- FR-23: The server is a standalone Rust binary that accepts WebSocket connections.
- FR-24: The server is stateless beyond current connections and active beacons — no database, no persistence.
- FR-25: The server handles up to 20 concurrent players per world instance.
- FR-26: The server generates exercises and spawns beacons at intervals, similar to the current client-side beacon spawning logic.
- FR-27: The server trusts client-reported answer results (no server-side validation of correctness).

## Success Criteria

- SC-1: Two players in separate browser windows see each other's characters moving in real time with nameplates.
- SC-2: Character positions appear smooth with no visible teleportation when players walk at normal speed.
- SC-3: A player joining sees all existing players and beacons within 2 seconds of connecting.
- SC-4: A disconnected player's character disappears from all screens within 5 seconds.
- SC-5: The game remains fully playable in single-player mode when the server is unreachable.
- SC-6: Network traffic per player stays under 5 KB/s during normal gameplay.
- SC-7: Two players approaching the same beacon both see it; the first correct answer claims it and the beacon fades on all screens.
- SC-8: Remote players push back the local player on collision (no overlap).

## Key Entities

### PlayerState (transmitted over WebSocket)

- **player_id**: Unique identifier assigned by server
- **position**: Vec3 (x, y, z) world position
- **yaw**: f32 character facing direction
- **animation_state**: Idle, Walking, or Running

### BeaconState (server-managed)

- **beacon_id**: Unique identifier assigned by server
- **position**: Vec3 (x, y, z) world position
- **exercise**: Question text and answer choices
- **correct_answer**: Index of the correct choice
- **status**: Dormant, Activated, Resolved
- **lifetime**: Remaining time before beacon expires

### ServerMessage (server to client)

- **PlayerJoined**: New player with initial state
- **PlayerUpdate**: Updated state for one or more players
- **PlayerLeft**: Player ID that disconnected
- **BeaconSpawned**: New beacon with position, exercise, and choices
- **BeaconResolved**: Beacon ID that was claimed, by which player
- **BeaconExpired**: Beacon ID that timed out
- **WorldSnapshot**: Full state of all players and beacons (sent on connect)

### ClientMessage (client to server)

- **UpdateState**: This player's current PlayerState
- **AnswerBeacon**: Beacon ID and whether the answer was correct

## Assumptions

- All connected players share a single world instance (no rooms/lobbies for initial version).
- The character model is the same for all players (no player customization).
- The game server will be hosted on a cheap VPS or free-tier cloud service — hosting choice is outside feature scope.
- A tick rate of 10-20 updates per second provides sufficient smoothness with client-side interpolation.
- The terrain heightmap is identical across all clients (deterministic generation from the same seed), so the server does not need to transmit terrain data.
- The game server URL will be different for development (localhost) and production; this is handled via compile-time configuration.
- WebSocket (wss://) is used for both native and WASM clients for simplicity and interoperability.
- Player names/identities are not needed for the initial version — nameplates may show a generated name or player number.
- The WebSocket client library must support both WASM and native targets.
- The server reuses the same exercise generation logic as the current client-side code (shared Rust crate).
- Beacon spawn positions must avoid vegetation and terrain obstacles, same as current client-side logic — this logic moves to the server but the heightmap/vegetation data must be available server-side.
- Scoring and coins remain client-side per player; the server only notifies who claimed a beacon.
- Difficulty selection happens client-side; the server spawns exercises at a single shared difficulty level or adapts to connected players.
