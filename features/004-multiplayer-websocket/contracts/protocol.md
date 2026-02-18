# WebSocket Protocol Contract

## Transport

- Protocol: WebSocket (ws:// for dev, wss:// for production)
- Message format: Binary frames, postcard-serialized Rust enums
- Client→Server tick rate: 10 Hz (100ms interval)
- Server→Client tick rate: 20 Hz (50ms interval)

## Connection Flow

1. Client opens WebSocket to server URL
2. Server assigns `player_id` (incrementing u32)
3. Server sends `WorldSnapshot` with all current players and beacons
4. Server broadcasts `PlayerJoined` to all other clients
5. Client begins sending `UpdateState` at 10 Hz
6. Server broadcasts `PlayerUpdate` (batched) at 20 Hz

## Disconnection Flow

1. Server detects WebSocket close or ping timeout
2. Server removes player from world state
3. Server broadcasts `PlayerLeft { player_id }` to all remaining clients
4. If player was interacting with a beacon, beacon returns to Dormant

## Message Encoding

All messages use `postcard::to_allocvec()` for encoding and `postcard::from_bytes()` for decoding. Both client and server share the same Rust types from the `tafels-shared` crate.

## Bandwidth Budget (SC-6: < 5 KB/s per player)

- PlayerState: ~18 bytes × 20 players × 20 Hz = ~7.2 KB/s outbound (batched PlayerUpdate)
- Optimization: only send players whose state changed since last tick
- BeaconSpawned/Resolved/Expired: rare events, negligible bandwidth
- ClientMessage::UpdateState: ~18 bytes × 10 Hz = 180 bytes/s inbound per client
