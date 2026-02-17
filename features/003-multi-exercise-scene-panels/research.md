# Research: Multi-Exercise Scene Panels

## Decision 1: Exercise Architecture — Resource vs Entity

**Decision:** Convert ActiveExercise from a Bevy Resource (singleton) to a Component on per-exercise entities.

**Rationale:** The current singleton Resource pattern fundamentally limits the system to one exercise at a time. Entity-based exercises are idiomatic Bevy ECS — each exercise becomes a first-class entity that can be queried, despawned, and managed independently. Systems iterate over exercises via Query, which naturally scales to N concurrent exercises.

**Alternatives Considered:**
- *Vec inside Resource:* Store multiple exercises in a single Resource. Rejected — requires manual index management, breaks ECS query patterns, harder to link panels to exercises.
- *Multiple named Resources:* Use Exercise1/Exercise2/Exercise3 resources. Rejected — inflexible, doesn't scale, not idiomatic.

## Decision 2: 3D Text Rendering — Bitmap Textures on Mesh Quads

**Decision:** Extend the existing `render_number_texture()` bitmap system to render arbitrary strings (question text, timer text) as textures applied to mesh quads in 3D space.

**Rationale:** The project already has a working bitmap font system that renders digits 0-9 as RGBA textures on panel cuboids. Extending this to additional characters (x, /, =, ?, s, space) is minimal effort. No new dependencies needed. The textures are crisp at the panel distances (5-20m). This approach is consistent with the existing neon glow aesthetic since the textures are applied to emissive materials.

**Alternatives Considered:**
- *Bevy Text2d:* Cannot render with Camera3d (documented in project memory). Not viable.
- *bevy_mod_billboard or third-party crate:* Adds dependency, may not be compatible with Bevy 0.18, violates tech stack principle of using built-in first.
- *SDF font rendering:* Overkill for simple math strings. Would require new shader pipeline.

## Decision 3: Text Facing — Fixed Direction, No Billboard

**Decision:** Question and timer text face a fixed direction (same as the answer panels), determined at spawn time. No per-frame billboard rotation. The player must walk around to the front of the exercise to read the text.

**Rationale:** User preference: the player should explore and discover — walking around to see the exercise is part of the gameplay. Fixed facing also means the proximity trigger can require the player to be in front of the exercise (within a forward arc), preventing activation from behind. This is simpler to implement (no per-frame system needed) and adds a spatial puzzle element to finding exercises.

**Alternatives Considered:**
- *Billboard (always face camera):* Rejected by user — removes the spatial discovery element. Player could read from any angle, reducing exploration incentive.
- *Shader-based billboarding:* Rejected — not desired for gameplay reasons.

## Decision 4: Beacon Visual — Emissive Mesh + Point Light

**Decision:** Beacons before activation appear as a tall emissive mesh (vertical pillar or sphere) with a child point light for ground illumination, using the existing neon color palette.

**Rationale:** Must be visible at 30-60m against the landscape backdrop. An emissive mesh with bloom already works well for the answer panels. A tall pillar (3-4m) peeks above most vegetation. The point light creates a colored glow on surrounding terrain that draws the eye. Randomizing the neon color per beacon adds visual variety.

**Alternatives Considered:**
- *Particle column:* Continuous upward particle stream. Rejected — too expensive for 3 concurrent beacons, each with ongoing particle spawning.
- *UI waypoint marker:* Screen-space indicator. Rejected — spec calls for in-world visual, and UI markers break the exploration feel.
- *Spotlight beam:* Vertical spotlight pointing upward. Rejected — may interact oddly with fog/atmosphere.

## Decision 5: Proximity Trigger — XZ Distance + Forward Arc Check

**Decision:** Use XZ-plane distance (ignoring Y axis) between player and beacon position, combined with a forward-arc check to ensure the player is in front of the exercise. Radius approximately 18 meters, forward arc approximately 120-180 degrees.

**Rationale:** The player must approach from the front to trigger the exercise. This works with the fixed-facing text/panels — the player naturally walks toward the readable side. The arc check uses the beacon's facing direction (determined at spawn): compute the angle between the beacon's forward vector and the vector from beacon to player. If the player is within the arc and within range, the exercise activates. Ignoring Y is consistent with existing collision patterns.

**Alternatives Considered:**
- *Distance only (no arc):* Player could trigger from behind, where they can't read the panels. Rejected — contradicts the fixed-facing design.
- *Frustum/visibility-based:* Only trigger when player sees the beacon. Rejected — too complex, camera direction is independent of movement direction.

## Decision 6: Timer Texture Caching

**Decision:** Pre-render timer textures for each second value ("12s", "11s", ..., "1s") at exercise activation time. Swap texture handles each second rather than regenerating textures per frame.

**Rationale:** `render_text_texture()` creates an Image and inserts it into Assets<Image>. Doing this every frame (60× per second per exercise) would generate excessive allocations and asset churn. Pre-rendering 12 textures at activation is cheap (one-time cost) and swapping the texture handle is a simple material update.

**Alternatives Considered:**
- *Per-frame regeneration:* Rejected for performance reasons (60 texture creations/sec × 3 exercises = 180 allocations/sec).
- *Shared texture atlas:* All timer values in one texture, UV shift to select. Rejected — adds UV complexity for minimal gain over handle swapping.

## Decision 7: Round Tracking — Engagements Not Spawns

**Decision:** Track `total_engaged` (exercises activated by proximity) toward the round limit, not `total_spawned` (beacons created). Beacons that vanish unvisited do not count.

**Rationale:** The player should not be penalized for beacons that expire naturally. If spawns counted, the round could end before the player has had 20 chances to answer. Tracking engagements ensures the player always gets exactly the configured number of opportunities.

**Alternatives Considered:**
- *Track spawns:* Round ends based on beacons spawned regardless of whether engaged. Rejected — unfair if beacons vanish before player can reach them.
- *Track total resolutions:* Only count answered or timed-out-after-engagement. This is functionally identical to tracking engagements since every engagement leads to a resolution.
