# Research: Lobby Single-Player Switch

## Decision 1: How to represent "connected solo" state

**Decision:** Add a `ConnectedSolo` variant to `ConnectionStatus` enum with the same fields as `Connected` (minus `send_timer` since no position updates are sent).

**Rationale:** A separate enum variant is cleaner than a boolean flag on `Connected` because:
- The beacon spawning system already checks `ConnectionStatus::Disconnected` to decide whether to spawn locally
- `ConnectedSolo` can be pattern-matched identically to `Disconnected` for beacon spawning
- No risk of forgetting to check the flag in various systems
- The `send_player_state` system can simply not run for `ConnectedSolo`

**Alternatives considered:**
- Boolean flag on `Connected`: Requires checking the flag in every system that branches on connection state. Easy to miss.
- Separate `Bevy` resource `SoloMode(bool)`: Adds another resource to check alongside `ConnectionStatus`. Increases coupling.

## Decision 2: Protocol changes for solo mode

**Decision:** Add two new `ClientMessage` variants:
- `EnterSolo` — client tells server it's switching to solo mode
- `LeaveSolo` — client tells server it's returning to lobby

Add a `playing_solo: bool` field to the `LobbyPlayer` struct.

**Rationale:** The server needs to know the player's solo status to:
- Exclude them from beacon spawning
- Exclude them from lobby readiness checks
- Include them in `LobbyState` with "playing solo" indicator

**Alternatives considered:**
- Disconnect and reconnect: Loses the persistent connection needed for lobby notifications.
- Reuse `Ready` message semantics: Overloading existing messages makes the protocol harder to reason about.

## Decision 3: Notification delivery mechanism

**Decision:** Reuse the existing `LobbyState` broadcast. The server already sends `LobbyState` messages when players join/leave. The client can derive "N players waiting in lobby" by counting `LobbyPlayer` entries where `playing_solo == false && ready == false` (i.e., players actually sitting in the lobby). Solo players and players in active rounds receive these broadcasts and show a notification HUD element.

**Rationale:** No new server message type needed. The existing `LobbyState` already contains the player list. Adding the `playing_solo` field is sufficient.

**Alternatives considered:**
- New `LobbyWaiting { count: u32 }` message: Adds protocol complexity for no benefit since `LobbyState` already has the data.
- Polling: Wasteful and adds latency.

## Decision 4: Game-over flow for connected solo

**Decision:** After the game-over stats screen, if `ConnectionStatus::ConnectedSolo`, transition to `MultiplayerRoundState::Lobby` and send `LeaveSolo` to server (which puts the player back in the lobby). The existing game-over → menu flow is only used for `Disconnected` mode.

**Rationale:** The clarification explicitly states: "Show game-over stats, then transition to lobby automatically." This requires differentiating the game-over handler based on connection status.

**Alternatives considered:**
- Always go to menu, let player re-enter lobby: Extra steps, poor UX.
- Skip game-over screen entirely: Loses the stats feedback players expect.

## Decision 5: Notification UI approach

**Decision:** A simple Bevy UI node (banner) at the top of the screen, rendered as an overlay during gameplay. Contains text + two buttons ("Join" / "Dismiss"). Managed by a `LobbyNotification` resource that tracks `waiting_count: u32` and `dismissed: bool`.

**Rationale:** Consistent with the existing HUD approach in `client/src/hud/`. Non-intrusive, doesn't block gameplay. The `dismissed` flag prevents the notification from reappearing for the same set of waiting players, but resets if the count changes.

**Alternatives considered:**
- Toast/popup with auto-dismiss timer: Player might miss it.
- Persistent indicator (always visible): Too intrusive during gameplay.
