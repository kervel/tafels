# Implementation Plan: Multi-Exercise Scene Panels

## Technical Context

- **Language:** Rust (edition 2024)
- **Engine:** Bevy 0.18
- **Rendering:** StandardMaterial with emissive for neon glow, Bloom post-processing (already enabled)
- **UI:** bevy::ui for simplified HUD (coins, combo, progress, feedback popup)
- **3D Text:** Bitmap texture rendering on mesh quads (extending existing render_number_texture to full strings)
- **Text Facing:** Fixed direction (same as panels), no billboard rotation. Player must approach from front.
- **Physics:** Existing custom projectile system (unchanged)
- **Collision:** Existing bevy::math::bounding for ball-panel; radius check for proximity trigger
- **State Machine:** Bevy States (Menu, Playing, GameOver) — unchanged
- **Particles:** Existing custom particle system — unchanged, handles concurrent bursts
- **New dependencies:** None — all built-in Bevy 0.18 features

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| P1: Idiomatic Rust | PASS | Entity-based pattern replacing Resource singleton is more idiomatic Bevy |
| P2: ECS-First Architecture | PASS | Exercise beacons as entities with components; proximity trigger as system; all game logic in ECS |
| P3: Performance-Aware Design | PASS | Dormant beacons are lightweight (1 entity + point light); activated exercises reuse existing entity budget; target >20 FPS with 3 concurrent |
| P4: Modularity and Separation | PASS | New beacon module; exercise.rs refactored; HUD simplified; clean plugin boundaries |
| P5: Readable Over Clever | PASS | Three-state lifecycle (Beacon → Activated → Resolved) is explicit and easy to follow |

## Tech Stack Compliance Report

### Approved Technologies (already in stack)
- Rust (stable, edition 2024)
- Bevy 0.18 (ECS, rendering, UI, bloom, states, bounding math)
- noise crate (terrain — unchanged)
- rand crate (beacon placement, world-lifetime randomization)

### New Technologies
- None — all features use built-in Bevy modules

### Prohibited Technologies
- None triggered

## Architecture Overview

### New Modules

```
src/game/
├── beacon.rs         — BeaconPlugin: beacon spawning, world-lifetime, vanish logic,
│                       proximity trigger detection, exercise reveal
```

### Modified Modules

```
src/game/mod.rs       — Add BeaconPlugin registration, add ActiveExercises resource,
                        refactor GameSession tracking (total_engaged vs current_index)
src/game/exercise.rs  — Refactor ActiveExercise from Resource to Component on beacon entity,
                        add ExerciseId component, multi-exercise timer ticking,
                        remove single-exercise spawn logic (moved to beacon.rs)
src/game/panels.rs    — Add ExerciseId to AnswerPanel, extend render_number_texture for
                        strings, add billboard text spawning, add 3D timer text
src/game/scoring.rs   — Link PendingAnswer to exercise via ExerciseId, despawn only the
                        answered exercise's panels, handle per-exercise resolution
src/game/coins.rs     — No changes (coin lifecycle is independent of exercise architecture)
src/projectile/
  collision.rs        — Add ExerciseId to PendingAnswer, identify which exercise's panel was hit
src/hud/mod.rs        — Remove QuestionDisplay, TimerDisplay, TimerBar; remove their update
                        systems; simplify layout to coins + combo + progress + feedback only
src/effects/
  particles.rs        — No changes (already handles multiple concurrent bursts via message queue)
```

### Entity Hierarchy (Per Exercise)

```
ExerciseBeacon entity (during Beacon phase):
  ├─ BeaconVisual mesh (emissive orb/pillar)
  └─ PointLight (glow)

ExerciseBeacon entity (after Activation):
  ├─ QuestionText mesh (billboard, "7 x 8 = ?")
  ├─ TimerText mesh (billboard, "10s")
  ├─ AnswerPanel × 4 (cuboid + point light child)
  └─ PanelPole × 4
```

### State Machine (Per Exercise Entity)

```
                ┌──────────────────────────────────────┐
                │                                      │
  Spawn ──►  Beacon  ──proximity──►  Activated  ──►  Resolved  ──►  Despawn
                │                       │                │
                │ world-lifetime        │ answer timer   │ particles +
                │ expires               │ expires        │ coin effects
                │                       │                │
                └──► Vanish (quiet)     └──► TimedOut    └──► Correct/Incorrect
                     no penalty              -coins           ±coins
```

## Implementation Phases

### Phase 1: Entity-Based Exercise Architecture

**Goal:** Refactor from singleton `ActiveExercise` Resource to entity-based exercises. Game works identically after this phase — same single-exercise flow, but using the new entity pattern.

1. Create `ExerciseId` component (simple u32 counter) in `exercise.rs`
2. Convert `ActiveExercise` from Resource to Component on an exercise entity
3. Add `ExerciseId` field to `AnswerPanel` component to link panels to their parent exercise
4. Add `ExerciseId` field to `PendingAnswer` resource (collision now reports which exercise was hit)
5. Modify `check_ball_panel_collision` in `collision.rs` to extract `ExerciseId` from the hit panel
6. Modify `process_pending_answer` in `scoring.rs` to:
   - Find the specific exercise entity by `ExerciseId`
   - Only despawn panels matching that `ExerciseId`
   - Despawn the exercise entity itself
7. Modify `tick_exercise_timer` to iterate over all exercise entities (Query instead of Res)
8. Modify `check_exercise_spawn` to query for exercise entity count instead of `Option<Res<ActiveExercise>>`
9. Keep spawning exactly 1 exercise at a time (for now) — validate that the refactored flow works identically
10. **Verify:** Game plays exactly as before. Single exercise at a time, same scoring, same HUD. Only internal architecture changed.

### Phase 2: Multi-Exercise Spawning + Placement

**Goal:** Multiple exercises exist simultaneously. Scattered placement across the landscape.

1. Create `ActiveExercises` resource in `game/mod.rs`:
   - `total_engaged: u32` (exercises activated this round, tracks toward round limit)
   - `total_spawned: u32` (total beacons created, including vanished ones)
   - `target_concurrent: u32` (default: 3)
   - `next_exercise_id: u32` (counter for unique IDs)
2. Create `src/game/beacon.rs` with `BeaconPlugin`:
   - `spawn_beacon` system: spawns new exercise entities when active count < target_concurrent
   - Placement logic: random position 30-60m from player, minimum 15m from player, minimum 10m separation from other beacons
   - Sample heightmap for ground level at spawn position
   - Avoid spawning inside vegetation clusters (check against VegetationCollider positions)
   - Spawn direction: random angle around player (not biased forward)
3. Remove old `check_exercise_spawn` from `exercise.rs` (replaced by beacon.rs)
4. Update `GameSession` to track `total_engaged` instead of `current_index`
5. Update `check_round_complete` in `scoring.rs` to use `total_engaged >= total_exercises`
6. For now, all spawned exercises immediately show panels and start timers (beacon phase comes next)
7. **Verify:** 3 exercises visible simultaneously. Answering one causes a replacement to spawn. Round ends after 20 engagements.

### Phase 3: Beacon Phase + Proximity Trigger

**Goal:** Exercises start as mystery beacons. Content only revealed when player approaches.

1. Add `BeaconState` enum to `beacon.rs`: `Dormant`, `Activated`, `Resolved`
2. Add `BeaconState` component to exercise entities (starts as `Dormant`)
3. Add `WorldLifetime` component: `remaining: f32` (random 30-60 seconds)
4. Create beacon visual: emissive orb or vertical pillar mesh with point light
   - Bright neon color, high emissive intensity (visible at 30-60m)
   - Spawn as child entity of the exercise beacon
5. Create `tick_world_lifetime` system:
   - Decrement `WorldLifetime.remaining` each frame for `Dormant` beacons only
   - When reaches zero: despawn beacon and all children (quiet dissolve), no coin penalty
   - Does NOT count toward round total
6. Create `check_proximity_trigger` system:
   - For each `Dormant` beacon, check XZ distance to player AND check if player is in front (within a forward-facing arc of the beacon's orientation)
   - If player within proximity radius (18m) AND in the forward arc: transition to `Activated`
   - On activation:
     - Remove beacon visual (orb/pillar)
     - Call `spawn_answer_panels` to create the 4 panels at the beacon's position
     - Spawn 3D question text and timer text (Phase 4 placeholder — use debug cubes for now)
     - Start the exercise countdown timer
     - Increment `total_engaged`
   - Once activated, cannot deactivate (timer runs regardless of player distance)
7. Modify `tick_exercise_timer` to only tick `Activated` exercises
8. Modify spawn logic: only spawn replacement beacons when total active (Dormant + Activated) < target
9. **Verify:** Beacons appear as glowing orbs. Walking toward one reveals the exercise. Beacons vanish after their world-lifetime with no penalty. Timer starts on proximity.

### Phase 4: 3D Question Text + Timer Text

**Goal:** Question and countdown timer rendered as 3D billboard text above each exercise.

1. Extend `render_number_texture` in `panels.rs` to a new `render_text_texture` function:
   - Accepts a string (e.g. "7 x 8 = ?", "10s")
   - Renders each character using the existing 5×7 bitmap font
   - Add bitmaps for characters: `x`, `/`, `=`, `?`, `s`, space, `-` (extend DIGIT_BITMAPS)
   - Returns RGBA Image texture
2. Create `QuestionText` marker component
3. Create `TimerText` marker component
4. On exercise activation (in `check_proximity_trigger`), spawn:
   - QuestionText entity: mesh quad with rendered question string texture, positioned 4-5m above ground (above panels), facing same direction as panels (fixed rotation)
   - TimerText entity: mesh quad with rendered timer string texture, positioned below question text, facing same direction as panels
5. No billboard system needed — text faces a fixed direction determined at spawn time (same rotation as panels)
6. Create `update_timer_text` system:
   - Each frame, regenerate the timer texture with current remaining seconds
   - Change texture color: green (>50% time), yellow (25-50%), red (<25%)
7. On exercise despawn, despawn question and timer text entities (children of exercise entity, auto-removed)
8. **Verify:** Each activated exercise shows "7 x 8 = ?" and "10s" floating above panels. Timer counts down with color changes. Text always faces camera.

### Phase 5: HUD Simplification

**Goal:** Remove exercise-specific info from HUD. Keep only game-level information.

1. Remove from `hud/mod.rs`:
   - `QuestionDisplay` component and `update_question_display` system
   - `TimerDisplay` component and `update_timer_display` system
   - `TimerBar` component and timer bar container from layout
2. Adjust HUD layout:
   - Top bar: CoinDisplay (left), ComboDisplay (center-right), ProgressDisplay (right)
   - No center content (timer bar and question removed)
   - FeedbackPopup remains centered
3. Update `ProgressDisplay` text to show `total_engaged / total_exercises` from `ActiveExercises` resource
4. **Verify:** HUD shows only coins, combo, and progress. No question text or timer on screen. Feedback popups still appear on correct/wrong answers.

### Phase 6: Polish + Beacon Visual Design

**Goal:** Beacon urgency cues, reveal animation, visual polish, performance validation.

1. Beacon urgency visual in `tick_world_lifetime`:
   - When <30% world-lifetime remaining: beacon starts pulsing (sine-wave scale oscillation)
   - When <15% world-lifetime remaining: beacon flickers (rapid visibility toggling)
   - Provides visual cue to player that this beacon will vanish soon
2. Exercise reveal animation in `check_proximity_trigger`:
   - Brief light burst when beacon transforms (particle effect at beacon position)
   - Panels could unfold/scale-up from zero over 0.3 seconds (optional, if performant)
3. Beacon visual variety:
   - Randomize beacon color from the neon palette
   - Slight vertical bob animation (sine wave, like collectible coins)
4. Performance testing:
   - Verify >20 FPS with 3 concurrent exercise groups on Intel Iris Xe
   - If under budget: reduce beacon point light range/intensity, simplify beacon mesh
   - Monitor entity count: beacons + panels + particles should stay under budget
5. Fine-tune parameters:
   - Spawn radius: 30-60m
   - Proximity trigger: 18m
   - World-lifetime: 30-60s random
   - Minimum beacon separation: 10m
   - Cooldown between spawn: 5s
6. **Verify:** Full gameplay loop works end-to-end. Beacons pulse/flicker before vanishing. Reveal animation plays on proximity. 20-exercise round completes without issues. Performance target met.

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| 3× entity count drops FPS below 20 on Iris Xe | Medium | High | Dormant beacons are cheap (1 mesh + 1 light). Profile after Phase 2 before adding more visual complexity. Reduce to 2 concurrent if needed. |
| Bitmap string texture regeneration each frame for timer is expensive | Medium | Medium | Cache timer textures (prerender "12s", "11s"... "1s" at exercise creation). Only swap texture handle, don't regenerate. |
| Proximity trigger feels inconsistent on hilly terrain | Low | Medium | Use XZ distance only (ignore Y) for trigger radius check, consistent with existing collision patterns. |
| Beacons obscured by vegetation despite placement checks | Medium | Low | Beacon visual should be tall (3-4m pillar) and bright enough to peek above most vegetation. Place beacons at positions with terrain height check. |
| Player confusion about hidden exercises (what is a beacon?) | Low | Medium | Beacon visual design should be clearly "game object" (neon glow, bob animation). First-time visual feedback on approach makes the mechanic self-teaching. |
| Panel collision ambiguity with multiple activated exercises nearby | Low | Medium | Existing collision checks first-hit-wins. Exercises 10m+ apart means ball trajectory naturally targets one group. |
