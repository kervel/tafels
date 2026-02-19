---
parent_branch: main
feature_number: 006
status: In Progress
created_at: 2026-02-19T08:00:00+01:00
---

# Feature: Lobby Single-Player Switch

## Overview

When a player connects to the multiplayer server and enters the lobby, but no other players are online, they should be able to switch to single-player mode without returning to the main menu. While playing single-player (connected to server), the player receives a notification if another player joins the lobby, allowing them to return to multiplayer. This must not break the fully disconnected single-player mode used on GitHub Pages.

## Clarifications

### Session 2026-02-19

- Q: When the solo player's round ends naturally, what should happen? → A: Show the normal game-over stats screen, then transition to the multiplayer lobby automatically.
- Q: If a multiplayer round is already in progress when the solo player taps "Join Lobby", what should happen? → A: Solo player waits in lobby for the current round to finish. Other players in the active round also see a notification that a player is waiting in the lobby.
- Q: Should solo players appear in the server's player count / lobby list? → A: Yes, solo players appear in the lobby list with a "playing solo" status indicator.

## User Scenarios

### Scenario 1: Switch to Single-Player from Empty Lobby

1. Player selects a difficulty and enters the game
2. Client connects to the server and enters the lobby
3. No other players are in the lobby
4. Player sees a "Play Solo" button/switch in the lobby UI
5. Player taps/clicks "Play Solo"
6. Lobby UI disappears and single-player gameplay begins immediately
7. Client remains connected to the server (but in a passive/observer state)
8. Server-managed beacons are NOT spawned; client spawns its own beacons locally

### Scenario 2: Notification When Another Player Joins

1. Player is in single-player mode (connected to server, playing solo)
2. Another player connects to the server and enters the lobby
3. The solo player sees a non-intrusive notification: "A player is waiting in the lobby"
4. Player can choose to:
   - **Join lobby**: Current single-player round ends, player returns to lobby
   - **Dismiss**: Continue playing solo, notification disappears
5. If the waiting player leaves before the solo player responds, the notification disappears automatically

### Scenario 3: Fully Disconnected Single-Player (GitHub Pages)

1. Player opens the game on GitHub Pages (no server available)
2. Connection attempt fails or `ForceSinglePlayer` is toggled
3. Game works exactly as it does today — no lobby, no notifications, pure offline single-player
4. No UI elements related to lobby switching or player notifications are shown

### Scenario 4: Multiple Notifications

1. Player is in solo mode (connected)
2. A player joins the lobby — notification appears
3. That player leaves — notification disappears
4. Later, another player joins — notification reappears
5. Notifications always reflect the current lobby state

### Scenario 5: Solo Player Joins Lobby Mid-Round

1. Player is in solo mode, mid-round (e.g., 15 exercises done out of 30)
2. Notification appears that a player is waiting
3. Player taps "Join Lobby"
4. Current single-player round ends (show brief stats or skip)
5. Player transitions to the lobby screen, connected and ready to start a multiplayer round

### Scenario 6: Solo Round Ends Naturally

1. Player is in solo mode (connected to server)
2. The round ends (all exercises completed or time runs out)
3. Normal game-over stats screen is shown
4. After the stats screen, player transitions automatically to the multiplayer lobby

### Scenario 7: Notification During Active Multiplayer Round

1. Players A and B are in an active multiplayer round
2. Player C (who was playing solo) taps "Join Lobby"
3. Player C enters the lobby and waits
4. Players A and B see a notification: "1 player is waiting in the lobby"
5. When the round ends, all players return to the lobby together

## Functional Requirements

### FR1: Solo Play Button in Lobby

- A clearly visible "Play Solo" button appears in the lobby UI
- Button is always available in the lobby, regardless of how many players are present
- Pressing the button transitions the player from lobby to single-player gameplay
- The WebSocket connection remains active after switching to solo

### FR2: Connected Solo State

- A new connection state distinguishes "connected but playing solo" from "fully disconnected"
- While in connected solo state:
  - Client spawns beacons locally (same as disconnected single-player)
  - Server does NOT spawn beacons for this player
  - Client still receives lobby-related messages (PlayerJoined, PlayerLeft, LobbyState)
  - Client does NOT send position updates or gameplay state to the server
- Server must be aware that this player is in solo mode (not participating in multiplayer rounds)
- Solo players appear in the lobby list for other players with a "playing solo" status indicator

### FR3: Lobby Player Notification

- When a player is in connected solo mode and another player enters the lobby, a notification appears
- When players are in an active multiplayer round and a player is waiting in the lobby, they also see this notification
- Notification is non-intrusive (e.g., a small banner or toast at the top of the screen)
- Notification shows: "1 player is waiting in the lobby" (or "N players waiting" for multiple)
- Notification includes two actions: "Join" and "Dismiss"
- Notification updates live: if the waiting player leaves, the notification disappears
- Notification is shown to both solo players and players in active multiplayer rounds

### FR4: Transition from Solo to Lobby

- When the solo player taps "Join", the current single-player round ends
- Player transitions to the lobby screen
- Player's connection state changes from solo back to normal multiplayer
- Server is notified that the player is back in the lobby
- When a solo round ends naturally (all exercises or time up), the game-over stats screen is shown, then the player transitions to the lobby automatically

### FR5: Backward Compatibility

- Fully disconnected single-player (no server) works identically to current behavior
- No solo-mode UI elements appear when disconnected
- The existing `ForceSinglePlayer` toggle continues to work as before
- The existing menu flow is unchanged

## Success Criteria

- A player alone in the lobby can start playing within 5 seconds (no forced waiting)
- Players in solo mode are notified within 2 seconds of another player joining the lobby
- Players in active multiplayer rounds are notified within 2 seconds of a player waiting in the lobby
- Notification dismissal and lobby-join actions respond within 1 second
- Fully disconnected single-player mode has zero visual or functional differences from current behavior
- No increase in network traffic during solo play compared to disconnected single-player (no position updates sent)

## Key Entities

### ConnectionStatus (modified)

Current states: `Disconnected`, `Connecting`, `Connected`, `Reconnecting`

New state needed: a way to represent "connected to server but playing solo" — either a new variant or a flag on `Connected`.

### MultiplayerRoundState (modified)

Current states: `None`, `Lobby`, `Countdown(f32)`, `Playing`, `RoundOver(Vec<PlayerScore>)`

May need: a `Solo` state or equivalent to distinguish solo play from disconnected play.

### Server-Side Player State

Server needs to track which players are in "solo" mode so it:
- Does not spawn beacons for them
- Does not count them toward lobby readiness
- Still sends them lobby state updates
- Shows them in lobby list with "playing solo" status for other players

### LobbyPlayer (modified)

The lobby player list entry needs a status field to distinguish between "waiting in lobby", "ready", and "playing solo".

### LobbyNotification (new)

A client-side resource or UI component that:
- Tracks whether a notification should be shown
- Contains the count of waiting players (those in the lobby, not playing solo or in a round)
- Manages dismiss state
- Works for both solo players and players in active multiplayer rounds

## Assumptions

- A single "Play Solo" button in the lobby is sufficient; no need for a countdown or confirmation dialog
- The notification should be a simple HUD overlay, not a modal that blocks gameplay
- When transitioning from solo to lobby, the single-player round simply ends (no save/resume)
- The server already broadcasts `LobbyState` messages that include player lists; this can be reused for notification updates
- The "Play Solo" button is available even when other players are in the lobby (player can choose solo regardless)
- After a solo round ends naturally, the transition to lobby is automatic (no choice to restart solo)
