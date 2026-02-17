# Research: Math Tables Game

## R1: 3D Text Rendering in Bevy 0.18

**Decision:** Use billboard quads with dynamically rendered text textures for floating question text and answer numbers. No external crate needed.

**Rationale:** Adding `bevy_rich_text3d` introduces a third-party dependency. The billboard approach (flat Mesh3d quad facing the camera) with a StandardMaterial using emissive properties works with the existing stack. The text can be pre-rendered as simple number meshes or use Bevy's UI overlay positioned via `Camera::world_to_viewport()`.

**Alternatives considered:**
- `bevy_rich_text3d` crate: Clean 3D text but adds dependency, against constitution preference for built-in solutions
- UI overlay positioned to match 3D coords: Simplest, no depth testing but acceptable for HUD-like floating text
- Mesh-based number shapes: Most performant but limited to digits

**Final approach:** Use Bevy UI text with `PositionType::Absolute`, positioned each frame via world-to-viewport projection. This avoids new crates and gives full font/style control. The text will appear to float in 3D space while actually being a 2D overlay tracked to world coordinates.

## R2: Ball Physics and Bouncing

**Decision:** Custom projectile system with gravity and simple bounce reflection. No physics engine needed.

**Rationale:** A full physics engine (Rapier, Avian) is overkill for a single ball projectile. The custom system is ~30 lines: a Velocity component, gravity in FixedUpdate, and velocity reflection on ground/panel collision with energy loss (restitution coefficient ~0.5).

**Bounce implementation:**
- On ground hit (y <= terrain height): reflect velocity.y, multiply by 0.5 restitution
- On panel hit: trigger answer scoring, then continue bouncing
- After 3 bounces or velocity < threshold: despawn with particle burst
- Ball lifetime cap: 5 seconds max before auto-despawn

## R3: Bloom and Neon Glow

**Decision:** Use Bevy's built-in `Bloom` component on the camera with HDR enabled. Emissive materials with LinearRgba values exceeding 1.0 produce the glow.

**Rationale:** Fully built-in, no crate needed. `Bloom::NATURAL` preset with slightly increased intensity (0.3) gives good neon effect without overwhelming the daylight scene.

**Key detail:** Emissive values need to be high (50-150 range) for visible bloom. The base_color should be a darker version of the emissive color so the panel is visible even without bloom.

## R4: Collision Detection Approaches

**Decision:** Use Bevy's built-in `bevy::math::bounding` module with `BoundingSphere` for ball-panel collision and simple radius checks for character-vegetation collision.

**Rationale:** Built-in, lightweight, no external physics crate needed.

**Ball-panel collision:** BoundingSphere (ball, radius 0.15) intersected against Aabb3d (panel, half-extents ~0.5x0.5x0.05).

**Character-vegetation collision:** Simple circle-circle test in XZ plane. Each tree/boulder gets a collision radius based on type. In the character movement system, after computing new position, check against all nearby vegetation positions. If overlapping, push the character position out.

## R5: Game State Management

**Decision:** Use Bevy's built-in `States` derive macro with `OnEnter`/`OnExit` schedules and `DespawnOnExit` for automatic cleanup.

**States:** Menu → Playing → GameOver

**Key patterns:**
- `DespawnOnExit(GameState::Playing)` on all game entities (panels, ball, HUD) for automatic cleanup
- `in_state(GameState::Playing)` run condition on game systems
- `NextState::set()` for transitions

## R6: Exercise Spawning Strategy

**Decision:** Spawn exercises in the direction the character is facing, 5-8m ahead. After completing an exercise, start a 5-second cooldown. If the player walks 6+ meters, spawn ahead. If cooldown expires first, spawn at current position.

**Distractor generation:** For multiplication `a * b = c`, distractors are: `c ± 1`, `c ± a`, `c ± b`, `a * (b±1)`. Pick 3 unique distractors that are positive and different from the correct answer.

## R7: Answer Panel Layout

**Decision:** 4 panels arranged in a semicircular arc facing the player. Each panel is a flat cuboid (1m wide, 0.8m tall, 0.1m deep) floating at ~1.2m height (roughly character chest level). Panels are spaced ~2m apart along the arc.

**Neon colors assigned randomly per panel from palette:** electric blue, hot pink, neon green, bright yellow.
