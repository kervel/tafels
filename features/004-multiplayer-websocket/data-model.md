# Data Model: Multiplayer via WebSocket

## Wire Protocol Types (shared crate)

### AnimationState
```
enum: Idle | Walking | Running
```
Maps to existing `CharacterState` in the game. Serialized as u8 variant tag.

### PlayerState
```
player_id:       u32       (assigned by server)
x, y, z:        f32 × 3   (world position)
yaw:             f32       (character facing direction)
animation:       AnimationState
```
~18 bytes serialized with postcard.

### BeaconInfo
```
beacon_id:       u32       (assigned by server)
x, y, z:        f32 × 3   (world position)
question_text:   String    (e.g. "7 × 8 = ?")
choices:         [u32; 4]  (answer options)
correct_index:   u8        (index of correct answer)
lifetime:        f32       (seconds remaining)
```

### ServerMessage
```
enum:
  WorldSnapshot { players: Vec<PlayerState>, beacons: Vec<BeaconInfo> }
  PlayerJoined(PlayerState)
  PlayerUpdate(Vec<PlayerState>)
  PlayerLeft { player_id: u32 }
  BeaconSpawned(BeaconInfo)
  BeaconResolved { beacon_id: u32, claimed_by: u32 }
  BeaconExpired { beacon_id: u32 }
```

### ClientMessage
```
enum:
  UpdateState(PlayerState)
  AnswerBeacon { beacon_id: u32, correct: bool }
```

## State Transitions

### Beacon Lifecycle (server-managed)
```
Spawned → Dormant → [player answers correctly] → Resolved → Faded → Removed
                   → [lifetime expires]         → Expired → Removed
                   → [answering player disconnects] → Dormant (reset)
```

### Player Lifecycle
```
Connected → server assigns player_id → WorldSnapshot sent → Active
Active → sends UpdateState at 10Hz → receives PlayerUpdate at 20Hz
Active → browser closed / network loss → server detects disconnect → PlayerLeft broadcast
```

### Connection State (client-side)
```
Disconnected → Connecting → Connected → Playing
Connected → network error → Reconnecting (backoff) → Connected
Connecting → timeout/failure → SinglePlayerFallback
```

## Relationships

- One server has many players (up to 20)
- One server has many active beacons
- Each beacon has one exercise
- Each beacon resolution is claimed by exactly one player
- Players do not interact with each other's exercises directly
