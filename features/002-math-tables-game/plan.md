# Implementation Plan: Math Tables Game

## Technical Context

- **Language:** Rust (edition 2024)
- **Engine:** Bevy 0.18
- **Rendering:** StandardMaterial with emissive for neon glow, Bloom post-processing
- **UI:** bevy::ui (Node, Text, TextFont, TextColor) for HUD and menus
- **3D Text:** UI overlay positioned via world-to-viewport projection (no external crate)
- **Physics:** Custom projectile system (Velocity + gravity in FixedUpdate)
- **Collision:** bevy::math::bounding (Aabb3d, BoundingSphere) for ball-panel; radius check for character-vegetation
- **State Machine:** Bevy States (Menu, Playing, GameOver)
- **Particles:** Custom particle system using small meshes with emissive materials and lifetime decay
- **New dependencies:** None — all built-in Bevy 0.18 features

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| P1: Idiomatic Rust | PASS | Standard Rust patterns, no unsafe needed |
| P2: ECS-First Architecture | PASS | All game logic as Bevy systems, components, and resources |
| P3: Performance-Aware Design | PASS | Simple geometry for panels/ball, distance culling retained, bloom is GPU-side |
| P4: Modularity and Separation | PASS | New plugins: GamePlugin, ExercisePlugin, ScoringPlugin, ProjectilePlugin |
| P5: Readable Over Clever | PASS | Clear system names, state machine for game flow |

## Tech Stack Compliance Report

### Approved Technologies (already in stack)
- Rust (stable, edition 2024)
- Bevy 0.18 (ECS, rendering, UI, bloom, states, bounding math)
- noise crate (terrain - unchanged)
- rand crate (exercise generation, particle randomization)

### New Technologies
- None — all features use built-in Bevy modules

### Prohibited Technologies
- None triggered

## Architecture Overview

### New Modules

```
src/
├── game/
│   ├── mod.rs          — GamePlugin, GameState enum, GameSession resource
│   ├── exercise.rs     — Exercise generation, ActiveExercise resource, spawn/despawn
│   ├── scoring.rs      — Coin tracking, speed bonus calculation, game over detection
│   ├── difficulty.rs   — Difficulty enum, table ranges, timer values
│   └── panels.rs       — Answer panel spawning, neon materials, layout
├── projectile/
│   ├── mod.rs          — ProjectilePlugin, ball spawning, aiming crosshair
│   ├── physics.rs      — Velocity, gravity, bouncing, lifetime
│   └── collision.rs    — Ball-panel intersection, ball-ground bounce
├── hud/
│   ├── mod.rs          — HudPlugin, HUD root setup
│   ├── coins.rs        — Coin counter display and update
│   ├── timer.rs        — Countdown timer bar
│   ├── question.rs     — Floating question text (world-to-viewport tracked)
│   └── screens.rs      — Menu screen, game over screen, round complete screen
├── effects/
│   ├── mod.rs          — EffectsPlugin
│   ├── particles.rs    — Particle burst spawning and lifetime system
│   └── neon.rs         — Bloom setup, neon material factory
├── collision/
│   └── mod.rs          — CollisionPlugin, VegetationCollider, character push-back
```

### Modified Modules

```
src/main.rs             — Add new plugins, add GameState init
src/camera/mod.rs       — Add Bloom component, enable HDR
src/camera/orbit.rs     — Auto-follow behind character, smooth return when not dragging
src/vegetation/mod.rs   — Add VegetationCollider to trees/rocks during spawn
src/character/mod.rs    — Add Run animation state for faster movement
```

## Implementation Phases

### Phase 1: Foundation (Game States + Collision)

**Goal:** Game state machine working, character blocked by trees/rocks.

1. Create `src/game/mod.rs` with GameState enum (Menu, Playing, GameOver), GameSession resource
2. Create `src/game/difficulty.rs` with Difficulty enum and config values
3. Create `src/collision/mod.rs` with VegetationCollider component and push-back system
4. Modify `src/vegetation/mod.rs` to attach VegetationCollider to trees (radius ~1.5m) and rocks (radius ~1.0m)
5. Modify `src/character/controller.rs` to check vegetation collision after movement
6. Register GameState in `src/main.rs`, wrap existing Update systems in `in_state(Playing)`
7. **Verify:** Character cannot walk through trees. Game compiles with state system.

### Phase 2: Camera Rework

**Goal:** Camera auto-follows behind character, with manual override.

1. Modify `src/camera/orbit.rs`:
   - Track character's facing direction
   - When not dragging, smoothly lerp camera yaw toward behind-character direction
   - When dragging, allow manual orbit as before
   - After releasing mouse, gradually return to auto-follow
2. Adjust camera pitch/distance for over-shoulder view
3. **Verify:** Camera stays behind character during walking, manual orbit works, smooth return.

### Phase 3: Exercise System + Answer Panels

**Goal:** Math exercises spawn with neon-lit panels, timer counts down.

1. Create `src/game/exercise.rs` with exercise generation (multiplication + division, distractor logic)
2. Create `src/game/panels.rs` with panel spawning (4 cuboid meshes in semicircle, emissive materials)
3. Create `src/effects/neon.rs` with neon material factory and bloom setup
4. Modify `src/camera/mod.rs` to add `Bloom::NATURAL` with HDR enabled
5. Implement exercise spawn trigger (cooldown timer + walk distance hybrid)
6. Implement countdown timer logic in exercise.rs
7. **Verify:** Walking triggers exercise spawn, 4 glowing panels appear ahead, timer counts down, panels despawn on timeout.

### Phase 4: Ball Throwing + Collision

**Goal:** Player can throw a ball that hits panels and bounces.

1. Create `src/projectile/mod.rs` with ball spawning on spacebar press
2. Create `src/projectile/physics.rs` with Velocity, gravity, terrain bounce, energy loss
3. Create `src/projectile/collision.rs` with ball-panel intersection (BoundingSphere vs Aabb3d)
4. Implement crosshair UI (centered dot)
5. Ball spawns from character position toward crosshair direction (camera forward)
6. On panel hit: trigger answer event. On ground: bounce with restitution 0.5, despawn after 3 bounces
7. **Verify:** Spacebar throws ball, ball arcs with gravity, bounces on ground, registers hit on panel.

### Phase 5: Scoring + HUD

**Goal:** Coins tracked, HUD shows all game info, game over works.

1. Create `src/game/scoring.rs` with coin add/subtract, speed bonus, game over check
2. Create `src/hud/mod.rs` with HUD root layout
3. Create `src/hud/coins.rs` with coin counter display
4. Create `src/hud/timer.rs` with countdown bar
5. Create `src/hud/question.rs` with floating question text (world-to-viewport)
6. Wire scoring events: correct answer → +coins, wrong → -coins, timeout → -coins
7. Implement game over trigger when coins <= 0
8. **Verify:** HUD shows coins/timer/progress, coins change on answers, game over triggers at 0.

### Phase 6: Menus + Game Flow

**Goal:** Full game loop from menu to completion/game over and back.

1. Create `src/hud/screens.rs` with menu screen (difficulty selection) and game over screen
2. Implement Menu state: show difficulty buttons, transition to Playing on selection
3. Implement GameOver state: show stats, restart button
4. Implement round completion: after N exercises, show summary, option to replay
5. Use `DespawnOnExit` for automatic cleanup on state transitions
6. **Verify:** Full loop: Menu → select difficulty → play → game over/complete → back to menu.

### Phase 7: Particle Effects + Polish

**Goal:** Visual feedback particles, ball trail, answer effects.

1. Create `src/effects/particles.rs` with particle burst system (small emissive spheres with velocity + lifetime)
2. Correct answer: green/gold burst at panel location
3. Wrong answer: red burst at panel
4. Timeout: panels fade out (scale down + alpha)
5. Ball trail: spawn small emissive particles along ball path
6. Ball disappear: small burst when ball despawns after bouncing
7. Coin gain/loss: small particles near HUD counter
8. **Verify:** All particle effects trigger correctly, performance stays above 25 FPS.

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Bloom + many emissive objects hurts FPS | Medium | High | Tune bloom intensity, limit emissive object count, reduce bloom resolution if needed |
| World-to-viewport text positioning feels floaty | Low | Medium | Add slight lag/smoothing, or switch to actual 3D mesh quads if needed |
| Ball-panel collision feels imprecise | Medium | Medium | Use generous collision volumes, add visual hit feedback, allow re-throw on miss |
| Too many entities (panels + particles + vegetation) | Low | High | Use DespawnOnExit, particle lifetime limits, entity budget monitoring |
| Timer too short for young children on Hard | Low | Low | Timer values are configurable, can adjust post-testing |
