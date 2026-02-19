# Data Model: Lobby Single-Player Switch

## Modified Entities

### ConnectionStatus (client, `client/src/network/mod.rs`)

```
enum ConnectionStatus {
    Disconnected,                           // No server (GitHub Pages, ForceSinglePlayer)
    Connecting { timeout: Timer },          // Attempting WS connection
    Connected {                             // Active multiplayer participant
        my_player_id: u32,
        my_color_index: u8,
        send_timer: Timer,
    },
    ConnectedSolo {                         // Connected to server, playing solo
        my_player_id: u32,
        my_color_index: u8,
    },
    Reconnecting { attempt: u32, next_try: Timer },
}
```

**Key behaviors by state:**
- `Disconnected` → client spawns beacons, no network, no notifications
- `Connected` → server spawns beacons, sends position updates, full multiplayer
- `ConnectedSolo` → client spawns beacons, no position updates, receives LobbyState for notifications

### MultiplayerRoundState (client, `client/src/game/mod.rs`)

No new variants needed. When entering solo mode:
- Set `MultiplayerRoundState::None` (no multiplayer round active)
- Gameplay proceeds as single-player (beacon spawning checks `ConnectionStatus::Disconnected` OR `ConnectedSolo`)

When returning from solo to lobby:
- Set `MultiplayerRoundState::Lobby`

### LobbyPlayer (shared, `shared/src/protocol.rs`)

```
struct LobbyPlayer {
    pub player_id: u32,
    pub name: String,
    pub ready: bool,
    pub playing_solo: bool,     // NEW: true if player is in solo mode
}
```

### ClientMessage (shared, `shared/src/protocol.rs`)

New variants:
```
enum ClientMessage {
    // ... existing variants ...
    EnterSolo,      // Client tells server: I'm switching to solo mode
    LeaveSolo,      // Client tells server: I'm returning to the lobby
}
```

### Server GameWorld (server, `server/src/state.rs`)

```
struct GameWorld {
    // ... existing fields ...
    pub player_solo: HashSet<u32>,    // NEW: players currently in solo mode
}
```

**Solo player rules:**
- Not counted in beacon spawning (`BeaconManager` skips solo players)
- Not counted toward lobby readiness (`player_ready` check excludes solo players)
- Included in `LobbyState` broadcasts with `playing_solo: true`
- Still receive all broadcast messages (for lobby notifications)

## New Entities

### LobbyNotification (client resource, `client/src/hud/`)

```
struct LobbyNotification {
    pub waiting_count: u32,     // Number of players waiting in lobby
    pub dismissed: bool,        // User dismissed the current notification
    pub last_count: u32,        // Last count when dismissed (resets on change)
}
```

**Behavior:**
- Show notification when `waiting_count > 0 && !dismissed`
- Set `dismissed = true` when user clicks "Dismiss"
- Reset `dismissed = false` when `waiting_count` changes from `last_count`
- "Join" button: end current round, transition to lobby, send `LeaveSolo`

## State Transitions

```
Menu → (select difficulty) → Playing
  → (if Disconnected) → single-player gameplay (no lobby, no notifications)
  → (if Connected) → Lobby
    → (click "Play Solo") → ConnectedSolo + single-player gameplay
      → (notification "Join") → end round → Lobby
      → (round ends naturally) → game-over screen → Lobby
    → (all ready) → Countdown → multiplayer gameplay → RoundOver → Lobby
```

## Beacon Spawning Decision Matrix

| ConnectionStatus | Beacon Source | Position Updates | Lobby Notifications |
|-----------------|--------------|-----------------|-------------------|
| Disconnected | Client local | None | None |
| Connected | Server | Yes (10 Hz) | Yes |
| ConnectedSolo | Client local | None | Yes |
