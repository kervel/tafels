# Tasks: Lobby Single-Player Switch

<!-- Tech Stack Validation: PASSED -->
<!-- All technologies already approved in plan.md -->
<!-- No new dependencies required -->

## Phase 1: Setup — Protocol & Shared Types

**Goal**: Extend the shared protocol to support solo mode signaling and lobby player status.

### [X] T001: Add `playing_solo` field to `LobbyPlayer` and new `ClientMessage` variants
- **File**: `shared/src/protocol.rs`
- **Story**: US1 (Solo Play Button), US2 (Notification)
- **Details**:
  1. Add `pub playing_solo: bool` field to `LobbyPlayer` struct
  2. Add `EnterSolo` variant to `ClientMessage` enum
  3. Add `LeaveSolo` variant to `ClientMessage` enum
- **Acceptance**: Code compiles. `LobbyPlayer` has `playing_solo` field. `ClientMessage` has `EnterSolo` and `LeaveSolo` variants. Postcard serialization still works.

---

## Phase 2: Foundational — Server-Side Solo Tracking

**Goal**: Server tracks which players are in solo mode and excludes them from multiplayer mechanics.

### [X] T002: Add `player_solo` set to `GameWorld`
- **File**: `server/src/state.rs`
- **Story**: US1
- **Details**:
  1. Add `pub player_solo: HashSet<u32>` field to `GameWorld`
  2. Initialize as empty in `GameWorld::new()` (or Default)
- **Acceptance**: `GameWorld` has `player_solo` field. Server compiles.

### [X] T003: Handle `EnterSolo` and `LeaveSolo` messages on server [P]
- **File**: `server/src/main.rs`
- **Story**: US1, US2
- **Depends on**: T001, T002
- **Details**:
  1. Handle `ClientMessage::EnterSolo`:
     - Add player to `player_solo` set
     - Remove from `player_ready` (no longer ready for multiplayer)
     - Broadcast updated `LobbyState` with `playing_solo: true` for this player
  2. Handle `ClientMessage::LeaveSolo`:
     - Remove player from `player_solo` set
     - Broadcast updated `LobbyState`
  3. Update `LobbyState` construction to populate `playing_solo` field from `player_solo` set
  4. Update lobby readiness check: exclude solo players from "all ready" condition
  5. On player disconnect: clean up `player_solo` set
- **Acceptance**: Server correctly tracks solo state. `LobbyState` broadcasts include `playing_solo` status. Solo players don't trigger "all ready" countdown.

### [X] T004: Skip beacon spawning for solo players [P]
- **File**: `server/src/beacon.rs`
- **Story**: US1
- **Depends on**: T002
- **Details**:
  1. In `BeaconManager::tick()`, skip beacon spawning for players in `player_solo` set
- **Acceptance**: Server does not spawn beacons for solo players. Other players still get beacons normally.

---

## Phase 3: Client Connection State

**Goal**: Client can represent "connected to server but playing solo" as a distinct state.

### [X] T005: Add `ConnectedSolo` variant to `ConnectionStatus`
- **File**: `client/src/network/mod.rs`
- **Story**: US1
- **Depends on**: T001
- **Details**:
  1. Add `ConnectedSolo { my_player_id: u32, my_color_index: u8 }` variant to `ConnectionStatus` enum
  2. Update `send_player_state` system: skip sending position updates when `ConnectedSolo`
  3. Update `receive_messages` system: handle lobby messages in `ConnectedSolo` state (same as `Connected`)
  4. Add helper method: `ConnectionStatus::is_solo() -> bool`
  5. Add helper method: `ConnectionStatus::is_connected_or_solo() -> bool`
- **Acceptance**: `ConnectedSolo` variant exists. No position updates sent in solo mode. Lobby messages still received. Helper methods work correctly.

---

## Phase 4: Beacon Spawning Guard

**Goal**: Client spawns beacons locally when in `ConnectedSolo` mode (same as `Disconnected`).

### [X] T006: Allow local beacon spawning for `ConnectedSolo`
- **File**: `client/src/game/beacon.rs`
- **Story**: US1
- **Depends on**: T005
- **Details**:
  1. Update `spawn_beacon` system guard: allow local spawning when `ConnectedSolo` (currently only when `Disconnected`)
  2. Change condition from `if !matches!(*connection_status, ConnectionStatus::Disconnected)` to also match `ConnectedSolo`
- **Acceptance**: Beacons spawn locally in `ConnectedSolo` mode. Beacons still spawn locally in `Disconnected` mode. Beacons still come from server in `Connected` mode.

---

## Phase 5: "Play Solo" Button in Lobby UI

**Goal**: Players can click "Play Solo" in the lobby to start single-player gameplay while staying connected.

### [X] T007: Add "Play Solo" button to lobby screen
- **File**: `client/src/hud/screens.rs`
- **Story**: US1
- **Depends on**: T005, T003
- **Details**:
  1. Add a "Play Solo" button to the lobby screen (alongside the existing "Ready!" button)
  2. On click:
     - Send `ClientMessage::EnterSolo` via WebSocket
     - Set `ConnectionStatus` to `ConnectedSolo` (copy `my_player_id` and `my_color_index` from `Connected`)
     - Set `MultiplayerRoundState::None`
     - Reset `ActiveExercises` and `GameSession` to start single-player gameplay
- **Acceptance**: "Play Solo" button visible in lobby. Clicking it starts single-player gameplay. WebSocket stays connected. Server receives `EnterSolo` message.

---

## Phase 6: Lobby Notification HUD

**Goal**: Solo players and players in active multiplayer rounds see a notification when others are waiting in the lobby.

### [X] T008: Create `LobbyNotification` resource and update system
- **File**: `client/src/hud/notification.rs` (new), `client/src/hud/mod.rs`
- **Story**: US2, US7
- **Depends on**: T005
- **Details**:
  1. Create new file `client/src/hud/notification.rs`
  2. Define `LobbyNotification` resource: `{ waiting_count: u32, dismissed: bool, last_count: u32 }`
  3. Add system `update_lobby_notification`:
     - Runs when `ConnectedSolo` or during multiplayer `Playing` state
     - Reads `RoundMessageBuffer::lobby_states` (or equivalent lobby state source)
     - Counts players where `!playing_solo && !ready` (waiting in lobby)
     - Updates `LobbyNotification.waiting_count`
     - Resets `dismissed` if count changed from `last_count`
  4. Add system `render_lobby_notification`:
     - If `waiting_count > 0 && !dismissed`: show notification banner at top of screen
     - Text: "1 player waiting in lobby" / "N players waiting in lobby"
     - "Join" button and "Dismiss" button
  5. Register notification plugin in `client/src/hud/mod.rs`
- **Acceptance**: Notification appears when players are waiting. Notification disappears when no one is waiting. "Dismiss" hides it. Count updates live. Works for both solo players and players in multiplayer rounds.

### [X] T009: Handle "Join" button — transition from solo/playing to lobby [P]
- **File**: `client/src/hud/notification.rs`
- **Story**: US2, US5, US7
- **Depends on**: T008
- **Details**:
  1. Add system `handle_notification_join`:
     - If in `ConnectedSolo`: end current round (despawn beacons, clear exercises)
     - Send `ClientMessage::LeaveSolo`
     - Set `ConnectionStatus` back to `Connected` (preserve `my_player_id`, `my_color_index`, recreate `send_timer`)
     - Set `MultiplayerRoundState::Lobby`
     - If in multiplayer `Playing`: just show the notification, player waits for round to end naturally (join happens after round over)
- **Acceptance**: Clicking "Join" from solo mode ends round and returns to lobby. Server receives `LeaveSolo`. Player appears as normal lobby participant. During multiplayer round, notification shows but "Join" is informational only (or navigates after round ends).

---

## Phase 7: Game-Over to Lobby Transition for Solo

**Goal**: When a solo round ends naturally, show stats then return to lobby.

### [X] T010: Solo game-over transitions to lobby instead of menu
- **File**: `client/src/hud/screens.rs`, `client/src/game/scoring.rs` (or wherever game-over is handled)
- **Story**: US6
- **Depends on**: T005, T007
- **Details**:
  1. In the game-over/round-complete handler:
     - If `ConnectionStatus::ConnectedSolo`:
       - Show normal game-over stats screen
       - On "Continue" click (or after delay): send `ClientMessage::LeaveSolo`, set `ConnectionStatus` to `Connected`, transition to `MultiplayerRoundState::Lobby`
     - If `ConnectionStatus::Disconnected`:
       - Existing behavior (game-over -> menu)
- **Acceptance**: Solo game-over shows stats, then transitions to lobby. Disconnected game-over still goes to menu. WebSocket connection preserved during transition.

---

## Phase 8: Polish — Backward Compatibility & Integration

**Goal**: Verify all existing flows still work and solo players display correctly.

### [X] T011: Display "playing solo" status in lobby list for other players
- **File**: `client/src/hud/screens.rs`
- **Story**: US1
- **Depends on**: T001, T007
- **Details**:
  1. In the lobby player list rendering, check `LobbyPlayer.playing_solo`
  2. Show "(playing solo)" status indicator next to solo players' names
- **Acceptance**: Other players see solo players listed with "playing solo" indicator. Non-solo players show normal status (waiting/ready).

### [X] T012: Backward compatibility verification [P]
- **Story**: US3 (Fully Disconnected Single-Player)
- **Depends on**: T006, T007, T008, T010
- **Details**:
  1. Verify `ForceSinglePlayer` toggle still works (stays `Disconnected`, no solo-mode UI)
  2. Verify GitHub Pages deploy: connection fails -> `Disconnected` -> normal single-player, no notifications
  3. Verify lobby with multiple players still works (Ready/Countdown/Round flow unchanged)
  4. Verify solo player appears in lobby list with "playing solo" for other players
  5. Verify no notification UI elements appear when `Disconnected`
- **Acceptance**: All existing flows unchanged. No regressions in disconnected single-player. No notification UI in disconnected mode.

---

## Dependencies

```
T001 (protocol)
├── T002 (server state) ──┐
│   ├── T003 (server handlers) ──── T007 (play solo button)
│   └── T004 (server beacon skip)   │
├── T005 (client ConnectedSolo) ────┤
│   ├── T006 (beacon guard)         │
│   ├── T008 (notification) ────── T009 (join handler)
│   └── T010 (game-over flow)
└── T011 (lobby display)
                                    T012 (verification, after all)
```

## Parallel Execution Opportunities

1. **After T001**: T002 and T005 can run in parallel (server state vs client state)
2. **After T002**: T003 and T004 can run in parallel (different server files)
3. **After T005**: T006, T008, and T010 can run in parallel (different client files)
4. **After T008**: T009 can start (same file, sequential)
5. **T011** can run in parallel with T008-T010 (lobby display is independent)

## Implementation Strategy

1. **MVP (T001-T007)**: Protocol + server + client state + play solo button = core solo functionality
2. **Notifications (T008-T009)**: Lobby notification HUD for awareness
3. **Polish (T010-T012)**: Game-over flow, display, backward compat verification
