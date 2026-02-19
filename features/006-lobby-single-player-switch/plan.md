# Implementation Plan: Lobby Single-Player Switch

## Technical Context

- **Language:** Rust (stable, edition 2024)
- **Engine:** Bevy 0.18
- **Architecture:** ECS with Bevy plugins
- **Networking:** ewebsock (client), axum + tokio (server)
- **Serialization:** postcard (binary)
- **Workspace:** client/, server/, shared/ crates

## Constitution Check

- **Principle 1 (Idiomatic Rust):** All changes use standard Rust patterns, enums, pattern matching.
- **Principle 2 (ECS-First):** New state tracked via Bevy resources and components. Systems check resources for branching.
- **Principle 3 (Performance):** No new per-frame allocations. Notification UI only rendered when needed.
- **Principle 4 (Modularity):** Changes contained within existing plugins (network, hud, game). Cross-plugin communication via resources.
- **Principle 5 (Readable):** Explicit enum variants over boolean flags. Clear state transitions.

## Tech Stack Compliance Report

All technologies used are already approved:
- Rust, Bevy 0.18, postcard, ewebsock, axum, tokio
- No new dependencies required

## Implementation Phases

### Phase 1: Protocol & Shared Types

**Files:** `shared/src/protocol.rs`

1. Add `playing_solo: bool` field to `LobbyPlayer` struct
2. Add `EnterSolo` and `LeaveSolo` variants to `ClientMessage` enum
3. Ensure backward-compatible serialization (postcard handles enum variant additions)

### Phase 2: Server-Side Solo Tracking

**Files:** `server/src/state.rs`, `server/src/main.rs`, `server/src/beacon.rs`

1. Add `player_solo: HashSet<u32>` to `GameWorld`
2. Handle `ClientMessage::EnterSolo`:
   - Add player to `player_solo` set
   - Remove from `player_ready` (no longer ready for multiplayer)
   - Broadcast updated `LobbyState` (with `playing_solo: true` for this player)
3. Handle `ClientMessage::LeaveSolo`:
   - Remove player from `player_solo` set
   - Broadcast updated `LobbyState`
4. Update `LobbyState` construction to populate `playing_solo` field from `player_solo` set
5. Update lobby readiness check: exclude solo players from "all ready" condition
6. Update `BeaconManager::tick()`: skip beacon spawning for players in `player_solo`
7. On player disconnect: clean up `player_solo` set

### Phase 3: Client Connection State

**Files:** `client/src/network/mod.rs`

1. Add `ConnectedSolo { my_player_id: u32, my_color_index: u8 }` variant to `ConnectionStatus`
2. Update `send_player_state` system: skip when `ConnectedSolo`
3. Update `receive_messages` system: handle messages in `ConnectedSolo` state (same as `Connected` for lobby messages)
4. Add helper: `ConnectionStatus::is_solo()` → bool
5. Add helper: `ConnectionStatus::is_connected_or_solo()` → bool (for checking "has server connection")

### Phase 4: Beacon Spawning Guard

**Files:** `client/src/game/beacon.rs`

1. Update `spawn_beacon` system: allow local spawning when `ConnectedSolo` (currently only when `Disconnected`)
   - Change `if !matches!(*connection_status, ConnectionStatus::Disconnected)` to also match `ConnectedSolo`

### Phase 5: "Play Solo" Button in Lobby UI

**Files:** `client/src/hud/screens.rs`

1. Add a "Play Solo" button to the lobby screen (alongside the existing "Ready!" button)
2. On click:
   - Send `ClientMessage::EnterSolo` via WebSocket
   - Set `ConnectionStatus` to `ConnectedSolo` (copy player_id and color_index from `Connected`)
   - Set `MultiplayerRoundState::None`
   - Start single-player gameplay (reset `ActiveExercises`, `GameSession`)

### Phase 6: Lobby Notification HUD

**Files:** `client/src/hud/mod.rs` (or new file `client/src/hud/notification.rs`)

1. Add `LobbyNotification` resource: `{ waiting_count: u32, dismissed: bool, last_count: u32 }`
2. Add system `update_lobby_notification`:
   - Runs when `ConnectedSolo` or during multiplayer `Playing` state
   - Reads `RoundMessageBuffer::lobby_states`
   - Counts players where `!playing_solo && !ready` (actually waiting in lobby, not in a round)
   - Updates `LobbyNotification.waiting_count`
   - Resets `dismissed` if count changed
3. Add system `render_lobby_notification`:
   - If `waiting_count > 0 && !dismissed`: spawn/show notification banner UI
   - Text: "1 player waiting in lobby" / "N players waiting in lobby"
   - "Join" button: triggers transition to lobby
   - "Dismiss" button: sets `dismissed = true`
4. Add system `handle_notification_join`:
   - End current round (despawn beacons, show brief game-over or skip)
   - Send `ClientMessage::LeaveSolo`
   - Set `ConnectionStatus` back to `Connected`
   - Set `MultiplayerRoundState::Lobby`

### Phase 7: Game-Over → Lobby Transition for Solo

**Files:** `client/src/hud/screens.rs`, `client/src/game/scoring.rs`

1. In the game-over/round-complete handler:
   - If `ConnectionStatus::ConnectedSolo`:
     - Show normal game-over stats screen
     - After stats screen (or on "Continue" click): send `LeaveSolo`, set `ConnectionStatus` to `Connected`, transition to `MultiplayerRoundState::Lobby`
   - If `ConnectionStatus::Disconnected`:
     - Existing behavior (game-over → menu)

### Phase 8: Backward Compatibility Verification

1. Verify `ForceSinglePlayer` toggle still works (stays `Disconnected`, no solo-mode UI)
2. Verify GitHub Pages deploy: connection fails → `Disconnected` → normal single-player, no notifications
3. Verify lobby with multiple players still works (Ready/Countdown/Round flow unchanged)
4. Verify solo player appears in lobby list with "playing solo" for other players

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Postcard serialization break from new enum variants | High | Test with both old and new client/server versions |
| Solo player receiving round messages they shouldn't act on | Medium | `ConnectedSolo` state ensures round messages are ignored |
| Notification UI overlapping with gameplay HUD | Low | Position at top of screen, small size, dismissible |
| Game-over → lobby transition losing WebSocket state | Medium | Carefully preserve `WsConnection` resource during transition |

## Files Changed Summary

| File | Change Type |
|------|-------------|
| `shared/src/protocol.rs` | Modified (LobbyPlayer field, ClientMessage variants) |
| `server/src/state.rs` | Modified (player_solo set) |
| `server/src/main.rs` | Modified (EnterSolo/LeaveSolo handlers, lobby state) |
| `server/src/beacon.rs` | Modified (skip solo players) |
| `client/src/network/mod.rs` | Modified (ConnectedSolo variant, helpers) |
| `client/src/game/beacon.rs` | Modified (spawn guard) |
| `client/src/hud/screens.rs` | Modified (Play Solo button, game-over flow) |
| `client/src/hud/notification.rs` | New (notification HUD) |
| `client/src/hud/mod.rs` | Modified (register notification plugin) |
| `client/src/game/mod.rs` | Modified (LobbyNotification resource) |
