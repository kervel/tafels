# Tasks: Math Tables Game

<!-- Tech Stack Validation: PASSED -->
<!-- Validated against: .specswarm/tech-stack.md v1.0.0 -->
<!-- No prohibited technologies found -->
<!-- 0 unapproved technologies require runtime validation -->

**Feature**: 002-math-tables-game
**Generated**: 2026-02-17
**Total Tasks**: 42
**Phases**: 7 (matching plan.md)

## Phase 1: Foundation — Game States + Vegetation Collision

**Goal**: Game state machine working, character blocked by trees/rocks.
**User Stories**: US8 (Vegetation Collision), US10 (Game Flow — partial)
**Checkpoint**: Character cannot walk through trees. Game compiles with state system.

### [X] T001: Create game state module with GameState enum and GameSession resource
- **File**: `src/game/mod.rs` (new)
- **Story**: US10
- **Depends**: —
- **Details**:
  - Create `src/game/` directory
  - Define `GameState` enum deriving `States`: `Menu`, `Playing`, `GameOver`
  - Define `GameSession` resource with fields from data-model.md: difficulty, total_exercises (20), current_index, coins (i32, default 10), correct_count, wrong_count, timeout_count, start_time
  - Define `GamePlugin` that adds `GameState` as init state (`Menu`), registers sub-plugins
  - Export all public types in mod.rs

### [X] T002: Create difficulty configuration module
- **File**: `src/game/difficulty.rs` (new)
- **Story**: US1, US10
- **Depends**: T001
- **Details**:
  - Define `Difficulty` enum: `Easy`, `Medium`, `Hard`
  - Implement config method returning `(table_range: RangeInclusive<u32>, timer_secs: f32)`:
    - Easy: 2..=5, 12.0
    - Medium: 2..=9, 10.0
    - Hard: 2..=12, 7.0
  - Implement `Default` for Difficulty (Easy)

### [X] T003: Create vegetation collision module [P]
- **File**: `src/collision/mod.rs` (new)
- **Story**: US8
- **Depends**: —
- **Details**:
  - Define `VegetationCollider` component with `radius: f32`
  - Define `CollisionPlugin`
  - Implement `check_vegetation_collision` system in `Update`:
    - Query all `(Transform, VegetationCollider)` entities
    - Query character `Transform` (mutable)
    - After character movement, check XZ distance to each collider
    - If overlapping (distance < collider.radius + character_radius ~0.3), push character position out along the overlap vector
  - Register system in CollisionPlugin, run in `PostUpdate` (after character movement)

### [X] T004: Attach VegetationCollider to trees and rocks during spawn
- **File**: `src/vegetation/mod.rs` (modify)
- **Story**: US8
- **Depends**: T003
- **Details**:
  - Import `VegetationCollider` from collision module
  - In `spawn_vegetation_entity`, after spawning:
    - Trees (all types): add `VegetationCollider { radius: 1.5 }`
    - Rocks: add `VegetationCollider { radius: 1.0 }`
    - Shrubs and Grass: no collider (do not block movement)

### [X] T005: Register new plugins in main.rs and wrap systems in state
- **File**: `src/main.rs` (modify)
- **Story**: US10
- **Depends**: T001, T003
- **Details**:
  - Add `mod game;` and `mod collision;`
  - Add `GamePlugin` and `CollisionPlugin` to app
  - Init `GameState` with `Menu` as default
  - Wrap existing terrain/vegetation/grass Update systems in `in_state(GameState::Playing)` or let them run always (terrain generation is fine always)
  - Wrap character controller in `in_state(GameState::Playing)`
  - For now, immediately set state to `Playing` on startup (menu comes in Phase 6)

### [X] T006: Verify Phase 1 — build and test collision
- **Depends**: T001-T005
- **Details**:
  - `cargo build` succeeds without warnings
  - Run game: character is blocked by trees and rocks
  - Character can still walk through grass and shrubs
  - FPS stays above 25

---

## Phase 2: Camera Rework

**Goal**: Camera auto-follows behind character, with manual override.
**User Story**: US9 (Camera Behavior)
**Checkpoint**: Camera stays behind character during walking, manual orbit works, smooth return.

### [X] T007: Track character facing direction in orbit camera
- **File**: `src/camera/orbit.rs` (modify)
- **Story**: US9
- **Depends**: T005
- **Details**:
  - Add `target_yaw: f32` field to `OrbitCamera` (computed from character facing)
  - Add `auto_follow: bool` field (default true, false while dragging)
  - Add `return_timer: f32` (counts up after mouse release, triggers auto-follow return)
  - In orbit update system:
    - Read character's Transform rotation to get facing direction yaw
    - Compute desired camera yaw as facing_yaw + PI (behind character)
    - When `auto_follow` is true and not dragging: lerp current yaw toward target_yaw (smooth factor ~2.0 * dt)
    - When dragging: allow manual orbit as before, set `auto_follow = false`
    - After mouse release: increment return_timer, when > 1.0s start lerping back, reset auto_follow=true when close

### [X] T008: Adjust camera pitch and distance for over-shoulder view
- **File**: `src/camera/orbit.rs` (modify)
- **Story**: US9
- **Depends**: T007
- **Details**:
  - Set default pitch slightly higher than current (0.35-0.40 range) for over-shoulder feel
  - Ensure camera offset looks slightly forward (toward where exercises will spawn)
  - Distance ~6-8m from character

### [X] T009: Verify Phase 2 — camera behavior
- **Depends**: T007, T008
- **Details**:
  - Camera stays behind character while walking forward
  - Turning the character causes camera to smoothly follow
  - Left-click drag overrides to manual orbit
  - Releasing mouse: camera smoothly returns to behind-character position
  - No jittering or snapping

---

## Phase 3: Exercise System + Answer Panels

**Goal**: Math exercises spawn with neon-lit panels, timer counts down.
**User Stories**: US1 (Starting a Game — partial), US2 (Encountering an Exercise)
**Checkpoint**: Walking triggers exercise spawn, 4 glowing panels appear ahead, timer counts down, panels despawn on timeout.

### [X] T010: Create exercise generation module
- **File**: `src/game/exercise.rs` (new)
- **Story**: US2
- **Depends**: T002
- **Details**:
  - Define `Operation` enum: `Multiply`, `Divide`
  - Define `ExerciseState` enum: `Active`, `Answered`, `TimedOut`
  - Define `ActiveExercise` resource with fields from data-model.md
  - Implement `generate_exercise(difficulty: &Difficulty, rng: &mut impl Rng) -> ActiveExercise`:
    - Randomly choose Multiply or Divide
    - For Multiply: pick a, b from table range, answer = a * b
    - For Divide: pick a, b, compute product = a * b, exercise is product / a = b
    - Generate 3 distractors: `answer ± 1`, `a * (b±1)`, `answer ± a`; filter duplicates and answer itself; ensure all positive
    - Shuffle 4 choices (1 correct + 3 distractors)
  - Implement spawn cooldown logic: `spawn_cooldown: Option<f32>` field, countdown in system

### [X] T011: Create neon material factory and bloom setup
- **File**: `src/effects/neon.rs` (new), `src/effects/mod.rs` (new)
- **Story**: US6
- **Depends**: —
- **[P]**
- **Details**:
  - Create `src/effects/` directory
  - Define `EffectsPlugin`
  - Define `NeonColor` enum with 4 colors: ElectricBlue, HotPink, NeonGreen, BrightYellow
  - Implement `create_neon_material(color: NeonColor, materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial>`:
    - Set `base_color` to darker version of neon color
    - Set `emissive` to LinearRgba with values 50-100 range
    - Set `unlit: false` (rely on bloom for glow)

### [X] T012: Add Bloom component to camera with HDR
- **File**: `src/camera/mod.rs` (modify)
- **Story**: US6
- **Depends**: T011
- **Details**:
  - Import `Bloom` from `bevy::core_pipeline::bloom`
  - Add `Bloom::NATURAL` to camera entity (with intensity tweaked to ~0.3)
  - Ensure camera has HDR enabled (`Camera { hdr: true, .. }`)
  - Verify existing fog/skybox still work with HDR

### [X] T013: Create answer panel spawning module
- **File**: `src/game/panels.rs` (new)
- **Story**: US2
- **Depends**: T010, T011
- **Details**:
  - Define `AnswerPanel` component with fields: value (u32), is_correct (bool), panel_index (u8)
  - Define `PanelSet` marker component for the parent entity grouping panels
  - Implement `spawn_answer_panels(exercise, character_transform, character_facing)`:
    - Position panels in semicircular arc 5-8m ahead of character, facing toward character
    - Each panel: Cuboid mesh (1.0m wide, 0.8m tall, 0.1m deep), floating at ~1.2m height
    - Assign random neon color per panel from palette
    - Use `DespawnOnExit(GameState::Playing)` component
    - Add text showing the number on the panel face (initially just the panel mesh, text in Phase 5)

### [X] T014: Implement exercise spawn trigger system
- **File**: `src/game/exercise.rs` (modify)
- **Story**: US2
- **Depends**: T010, T013
- **Details**:
  - Create `ExerciseSpawnState` resource tracking: cooldown timer, last spawn position
  - System `check_exercise_spawn`:
    - If no active exercise and cooldown expired (5s):
      - If character moved > 6m from last spawn → spawn ahead
      - If cooldown expired regardless → spawn at current position
    - Generate new exercise, spawn panels, set ActiveExercise resource
  - System `tick_exercise_timer`:
    - Decrement time_remaining each frame
    - When time_remaining <= 0: set state to TimedOut, despawn panels

### [X] T015: Verify Phase 3 — exercises spawn and time out
- **Depends**: T010-T014
- **Details**:
  - Walking triggers first exercise after initial cooldown
  - 4 glowing panels appear in semicircle ahead of character
  - Panels have neon glow visible with bloom
  - Timer counts down (visible in next phase)
  - Panels despawn on timeout
  - New exercise spawns after cooldown
  - FPS stays above 25

---

## Phase 4: Ball Throwing + Collision

**Goal**: Player can throw a ball that hits panels and bounces.
**User Stories**: US3 (Answering Correctly — partial), US4 (Answering Incorrectly — partial)
**Checkpoint**: Spacebar throws ball, ball arcs with gravity, bounces on ground, registers hit on panel.

### [X] T016: Create projectile plugin with ball spawning
- **File**: `src/projectile/mod.rs` (new)
- **Story**: US3
- **Depends**: T005
- **Details**:
  - Create `src/projectile/` directory
  - Define `Projectile` marker component
  - Define `Velocity(Vec3)` component
  - Define `BounceCount { count: u8, max_bounces: u8 }` component (max 3)
  - Define `ProjectileLifetime { remaining: f32 }` component (5.0s)
  - Define `ProjectilePlugin`
  - Implement `spawn_ball` system:
    - On spacebar press (and active exercise, and no ball currently in flight)
    - Spawn sphere mesh (radius 0.15), emissive white/yellow material
    - Position at character location + slight height offset
    - Velocity: camera forward direction * throw_speed (~15 m/s), slight upward arc

### [X] T017: Implement ball physics (gravity, bouncing, lifetime)
- **File**: `src/projectile/physics.rs` (new)
- **Story**: US3
- **Depends**: T016
- **Details**:
  - System `apply_projectile_gravity` in FixedUpdate:
    - Add gravity (0, -9.81, 0) * dt to velocity each tick
    - Update position by velocity * dt
  - System `bounce_on_terrain`:
    - Sample terrain height at ball XZ position
    - If ball.y <= terrain_height: reflect velocity.y, multiply by 0.5 restitution
    - Increment bounce count
    - If bounce count >= max_bounces or velocity magnitude < 0.5: despawn with event
  - System `tick_lifetime`:
    - Decrement remaining, despawn if <= 0

### [X] T018: Implement ball-panel collision detection
- **File**: `src/projectile/collision.rs` (new)
- **Story**: US3, US4
- **Depends**: T016, T013
- **Details**:
  - System `check_ball_panel_collision`:
    - For each Projectile: construct BoundingSphere (pos, radius 0.15)
    - For each AnswerPanel: construct Aabb3d from transform and half-extents (0.5, 0.4, 0.05)
    - If intersecting: send `AnswerHitEvent { panel_entity, answer_value, is_correct }`
    - Despawn the ball
    - Only first hit counts (ignore subsequent hits for same exercise)
  - Define `AnswerHitEvent` as Bevy Event

### [X] T019: Add crosshair UI element
- **File**: `src/hud/mod.rs` (new — partial, expanded in Phase 5)
- **Story**: US3
- **Depends**: —
- **[P]**
- **Details**:
  - Create `src/hud/` directory
  - Define `HudPlugin`
  - Define `CrosshairMarker` component
  - Spawn centered crosshair dot (small white circle/dot, ~6px) using bevy::ui Node with absolute positioning at screen center
  - Only visible during `GameState::Playing`

### [X] T020: Verify Phase 4 — ball throwing works
- **Depends**: T016-T019
- **Details**:
  - Spacebar throws ball from character position toward crosshair
  - Ball arcs with gravity
  - Ball bounces on terrain with diminishing height
  - Ball despawns after 3 bounces
  - Hitting a panel triggers AnswerHitEvent
  - Crosshair visible at center of screen
  - Can throw again if ball missed all panels

---

## Phase 5: Scoring + HUD

**Goal**: Coins tracked, HUD shows all game info, game over works.
**User Stories**: US3 (Answering Correctly — complete), US4 (Answering Incorrectly — complete), US5 (Running Out of Time — complete), US6 (Game Over)
**Checkpoint**: HUD shows coins/timer/progress, coins change on answers, game over triggers at 0.

### [X] T021: Create scoring module
- **File**: `src/game/scoring.rs` (new)
- **Story**: US3, US4, US5
- **Depends**: T010, T018
- **Details**:
  - System `handle_answer_hit`:
    - Read `AnswerHitEvent`
    - If correct: +3 base coins. Speed bonus: if answered in first half of timer → +1 to +2 extra coins
    - If wrong: -2 coins
    - Send `ScoreChangeEvent { amount: i32, reason: ScoreReason }`
    - Update `ActiveExercise.state` to Answered
    - Update `GameSession` stats (correct_count, wrong_count, current_index)
  - System `handle_timeout`:
    - When ActiveExercise state becomes TimedOut: -3 coins
    - Send `ScoreChangeEvent`
    - Update GameSession.timeout_count
  - System `check_game_over`:
    - If `GameSession.coins <= 0`: transition to `GameState::GameOver`
  - Define `ScoreChangeEvent` and `ScoreReason` enum

### [X] T022: Create coin counter HUD display
- **File**: `src/hud/coins.rs` (new)
- **Story**: US5
- **Depends**: T019, T021
- **Details**:
  - Define `CoinDisplay` marker component
  - Spawn text node in top-left area: "Coins: 10"
  - System `update_coin_display`: read GameSession.coins, update text
  - Use large readable font (TextFont size ~28)
  - Gold/yellow text color

### [X] T023: Create countdown timer bar
- **File**: `src/hud/timer.rs` (new)
- **Story**: US2, US5
- **Depends**: T019, T010
- **Details**:
  - Define `TimerDisplay` marker component
  - Spawn horizontal progress bar near top of screen
  - System `update_timer_display`:
    - Read ActiveExercise.time_remaining / time_limit → fraction
    - Set bar width as fraction of max width
    - Color: green > 50%, yellow > 25%, red < 25%
  - Hide when no active exercise

### [X] T024: Create floating question text display
- **File**: `src/hud/question.rs` (new)
- **Story**: US2
- **Depends**: T019, T010
- **Details**:
  - Define `QuestionDisplay` marker component
  - System `update_question_display`:
    - Read ActiveExercise: format "a x b = ?" or "a / b = ?"
    - Use `Camera::world_to_viewport()` to project panel-center-above position to screen coords
    - Position text node at projected screen coords (PositionType::Absolute)
    - Large neon-styled text (TextFont size ~36, emissive-like color)
  - Show answer numbers on each panel as world-tracked text too

### [X] T025: Create progress counter display [P]
- **File**: `src/hud/coins.rs` (extend) or separate
- **Story**: US5
- **Depends**: T022
- **Details**:
  - Define `ProgressDisplay` marker component
  - Show "Question 5 / 20" text in top area
  - System `update_progress_display`: read GameSession.current_index / total_exercises

### [X] T026: Wire scoring events and game over transition
- **File**: `src/game/scoring.rs` (modify)
- **Story**: US6
- **Depends**: T021, T022
- **Details**:
  - Connect ScoreChangeEvent to GameSession.coins update
  - On game over: set `NextState(GameState::GameOver)`
  - Ensure DespawnOnExit(Playing) cleans up all game entities

### [X] T027: Verify Phase 5 — scoring and HUD
- **Depends**: T021-T026
- **Details**:
  - HUD shows coin count, timer bar, question text, progress counter
  - Correct answer: coins increase, celebration feedback
  - Wrong answer: coins decrease
  - Timeout: coins decrease (larger penalty)
  - Coins reach 0: game over state triggers
  - All game entities cleaned up on state transition

---

## Phase 6: Menus + Game Flow

**Goal**: Full game loop from menu to completion/game over and back.
**User Stories**: US1 (Starting a New Game — complete), US6 (Game Over — complete), US7 (Completing a Round — complete)
**Checkpoint**: Full loop: Menu → select difficulty → play → game over/complete → back to menu.

### [X] T028: Create menu screen with difficulty selection
- **File**: `src/hud/screens.rs` (new)
- **Story**: US1
- **Depends**: T019, T001, T002
- **Details**:
  - Define `MenuScreen` marker component
  - Spawn UI overlay with:
    - Title: "Math Tables Game" (large text)
    - Three buttons: Easy, Medium, Hard (with descriptions)
    - Styled with neon colors matching game aesthetic
  - System `handle_menu_input`:
    - On button click: set Difficulty in GameSession, transition to Playing
  - Use `DespawnOnExit(GameState::Menu)` on all menu entities
  - Spawn with `OnEnter(GameState::Menu)`

### [X] T029: Create game over screen with stats
- **File**: `src/hud/screens.rs` (extend)
- **Story**: US6
- **Depends**: T028, T021
- **Details**:
  - Define `GameOverScreen` marker component
  - Spawn on `OnEnter(GameState::GameOver)`:
    - "Game Over" title
    - Stats: total questions, correct answers, accuracy %, coins earned
    - Read from GameSession
    - "Play Again" button → reset session, transition to Menu
  - Use `DespawnOnExit(GameState::GameOver)`

### [X] T030: Create round completion screen
- **File**: `src/hud/screens.rs` (extend)
- **Story**: US7
- **Depends**: T028, T021
- **Details**:
  - Check in exercise system: if current_index >= total_exercises → show completion
  - Could use GameOver state or separate sub-state
  - Show: score, accuracy, average response time
  - Buttons: "Play Again" (same difficulty), "Change Difficulty" (back to menu)

### [X] T031: Initialize game session on state enter
- **File**: `src/game/mod.rs` (modify)
- **Story**: US1, US10
- **Depends**: T001, T028
- **Details**:
  - System on `OnEnter(GameState::Playing)`:
    - Reset GameSession: coins=10, current_index=0, counts=0
    - Set start_time
    - Ensure character is in correct position
  - Ensure terrain/vegetation/character remain across state transitions (no despawn on these)

### [X] T032: Remove auto-start-Playing hack from Phase 1
- **File**: `src/main.rs` (modify)
- **Story**: US10
- **Depends**: T028, T031
- **Details**:
  - Remove the temporary "immediately set state to Playing" from T005
  - Game now starts in Menu state properly

### [X] T033: Verify Phase 6 — full game loop
- **Depends**: T028-T032
- **Details**:
  - Game starts at menu screen
  - Can select difficulty
  - Playing starts with correct coin count
  - Game over triggers and shows stats
  - Round completion shows at 20 exercises
  - Can restart or return to menu from both screens
  - No entity leaks between state transitions

---

## Phase 7: Particle Effects + Polish

**Goal**: Visual feedback particles, ball trail, answer effects.
**User Stories**: US3 (celebration particles), US4 (error particles), US5 (timeout fade), US7 (polish)
**Checkpoint**: All particle effects trigger correctly, performance stays above 25 FPS.

### T034: Create particle burst system
- **File**: `src/effects/particles.rs` (new)
- **Story**: US3, US4
- **Depends**: T011
- **Details**:
  - Define `Particle` component with velocity (Vec3), lifetime (f32)
  - Define `ParticleBurstEvent { position: Vec3, color: Color, count: u32, spread: f32 }`
  - System `spawn_particle_burst`:
    - On event: spawn `count` small sphere meshes (radius 0.03-0.05)
    - Emissive material matching color
    - Random velocity in spread cone
    - Lifetime 0.5-1.5s
  - System `update_particles`:
    - Apply velocity + gravity (lighter gravity for floaty feel)
    - Decrement lifetime, despawn when expired
    - Fade out: scale down as lifetime approaches 0

### T035: Add correct answer celebration effect
- **File**: `src/effects/particles.rs` (extend)
- **Story**: US3
- **Depends**: T034, T021
- **Details**:
  - On correct answer ScoreChangeEvent:
    - Green/gold burst at panel position (30-50 particles)
    - Large spread, upward bias
  - Panel briefly flashes brighter (increase emissive for 0.3s then despawn)

### T036: Add wrong answer error effect
- **File**: `src/effects/particles.rs` (extend)
- **Story**: US4
- **Depends**: T034, T021
- **Details**:
  - On wrong answer ScoreChangeEvent:
    - Red/orange burst at hit panel (20-30 particles)
    - Smaller spread
  - Briefly highlight correct panel in green (flash for 1s before despawn)

### T037: Add timeout panel fade effect
- **File**: `src/effects/particles.rs` (extend)
- **Story**: US5
- **Depends**: T034, T014
- **Details**:
  - On timeout: panels scale down over 0.5s (lerp scale to 0)
  - Small dissolving particle burst per panel (10 particles each)
  - Then despawn panels

### T038: Add ball trail particles
- **File**: `src/effects/particles.rs` (extend)
- **Story**: US3
- **Depends**: T034, T016
- **Details**:
  - System `spawn_ball_trail`:
    - Every 0.05s while ball exists: spawn 1-2 tiny emissive particles at ball position
    - Low velocity (slight random drift)
    - Very short lifetime (0.3s)
    - White/yellow color
  - On ball despawn (after bouncing): small burst (10 particles)

### T039: Add coin change HUD particles [P]
- **File**: `src/effects/particles.rs` (extend)
- **Story**: US3, US4, US5
- **Depends**: T034, T022
- **Details**:
  - On coin gain: small gold particles floating upward near coin counter (screen-space positioned)
  - On coin loss: small red particles falling downward near coin counter
  - 5-10 particles per event
  - Use UI nodes rather than 3D particles for this (positioned relative to HUD)

### T040: Add run animation for faster movement [P]
- **File**: `src/character/animation.rs` (modify), `src/character/controller.rs` (modify)
- **Story**: Polish
- **Depends**: T005
- **Details**:
  - Add Run animation (index 16) to CharacterAnimations
  - Add `Running` variant to CharacterState
  - Shift key held → increase move speed by 1.5x, use Run animation at 1.5x speed
  - Update animate_character to handle Run state

### T041: Performance tuning pass
- **Depends**: T034-T040
- **Details**:
  - Cap max particles alive at ~200 (skip spawning if at limit)
  - Ensure bloom intensity doesn't tank FPS (reduce if needed)
  - Verify distance culling still works for vegetation
  - Profile with cargo-flamegraph if FPS < 25
  - Tune particle counts down if needed

### T042: Verify Phase 7 — final polish
- **Depends**: T034-T041
- **Details**:
  - Correct answer: green/gold burst at panel
  - Wrong answer: red burst, correct answer highlighted briefly
  - Timeout: panels fade out with dissolving particles
  - Ball has light trail while flying
  - Ball burst on despawn
  - Coin counter shows particles on gain/loss
  - Shift to run works with animation
  - FPS stays above 25 on Intel Iris Xe

---

## Dependency Graph

```
Phase 1 (Foundation):
  T001 ──► T002 ──► T010
  T001 ──► T005
  T003 ──► T004
  T003 ──► T005
  T005 ──► T006

Phase 2 (Camera):
  T005 ──► T007 ──► T008 ──► T009

Phase 3 (Exercises):
  T002 ──► T010 ──► T013 ──► T014 ──► T015
  T011 [P] ──► T012
  T011 ──► T013

Phase 4 (Ball):
  T005 ──► T016 ──► T017
  T016 + T013 ──► T018
  T019 [P] (independent)
  T016-T019 ──► T020

Phase 5 (Scoring + HUD):
  T010 + T018 ──► T021 ──► T022 ──► T025
  T010 ──► T023
  T010 ──► T024
  T021 + T022 ──► T026 ──► T027

Phase 6 (Menus):
  T001 + T002 ──► T028 ──► T029, T030
  T028 ──► T031 ──► T032 ──► T033

Phase 7 (Particles):
  T011 ──► T034 ──► T035, T036, T037, T038
  T034 + T022 ──► T039 [P]
  T005 ──► T040 [P]
  T034-T040 ──► T041 ──► T042
```

## Parallel Execution Opportunities

### Within Phase 1:
- T001 (game states) ∥ T003 (collision) — different directories

### Within Phase 3:
- T010 (exercise gen) ∥ T011 (neon materials) — different directories

### Within Phase 4:
- T016 (projectile) ∥ T019 (crosshair HUD) — different directories

### Within Phase 5:
- T023 (timer bar) ∥ T024 (question text) ∥ T025 (progress counter) — different files

### Within Phase 7:
- T039 (coin particles) ∥ T040 (run animation) — independent files
- T035, T036, T037, T038 are sequential within same file but could be batched

## Implementation Strategy

1. **MVP (Phases 1-4)**: Character with collision, auto-follow camera, exercise spawning, ball throwing. This is the core gameplay loop — playable but minimal UI.
2. **Complete Game (Phase 5-6)**: Scoring, HUD, menus, full game flow. The game is feature-complete.
3. **Polish (Phase 7)**: Particle effects and run animation. Visual reward and juice.

Suggested: Implement phases sequentially. Within each phase, parallelize where marked [P].
