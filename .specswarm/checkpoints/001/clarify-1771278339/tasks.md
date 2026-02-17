<!-- Tech Stack Validation: PASSED -->
<!-- Validated against: .specswarm/tech-stack.md -->
<!-- No prohibited technologies found -->

# Tasks: Persona Walking in Outdoor Landscape

**Feature**: 001-persona-walking-in-outdoor-landscape
**Generated**: 2026-02-16
**Total Tasks**: 28
**Phases**: 7

---

## Phase 1: Project Setup (Foundation)

### T001: Initialize Cargo project with Bevy

**File**: `Cargo.toml`, `src/main.rs`
**Status**: pending
**Depends on**: none

Create a new Rust project:
- Run `cargo init` in the project root
- Add dependencies to `Cargo.toml`:
  ```toml
  [dependencies]
  bevy = "0.18"
  noise = "0.9"
  rand = "0.8"

  [profile.dev]
  opt-level = 1

  [profile.dev.package."*"]
  opt-level = 3
  ```
- In `src/main.rs`, create a minimal Bevy app with `DefaultPlugins` and
  a window title "3dt - Alpine Landscape". Verify it compiles and opens
  a window.

### T002: Create plugin module structure [P]

**File**: `src/terrain/mod.rs`, `src/vegetation/mod.rs`, `src/grass/mod.rs`, `src/character/mod.rs`, `src/camera/mod.rs`, `src/lighting/mod.rs`
**Status**: pending
**Depends on**: T001

Create the directory and module structure:
- `src/terrain/mod.rs` - empty `TerrainPlugin` implementing `Plugin`
- `src/vegetation/mod.rs` - empty `VegetationPlugin`
- `src/grass/mod.rs` - empty `GrassPlugin`
- `src/character/mod.rs` - empty `CharacterPlugin`
- `src/camera/mod.rs` - empty `CameraPlugin`
- `src/lighting/mod.rs` - empty `LightingPlugin`

Each plugin should have `pub struct XPlugin;` and `impl Plugin for XPlugin`
with an empty `build` method. Register all plugins in `main.rs`.

### T003: Create assets directory structure [P]

**File**: `assets/models/.gitkeep`, `assets/textures/.gitkeep`
**Status**: pending
**Depends on**: T001

Create the asset directory structure:
- `assets/models/` - for glTF character and vegetation models
- `assets/textures/` - for PBR terrain texture sets
- Add `.gitkeep` files so directories are tracked

---

## Phase 2: Terrain (Scenario 3 - Exploring the Environment)

### T004: Implement heightmap generation

**File**: `src/terrain/heightmap.rs`
**Status**: pending
**Depends on**: T002

Implement procedural alpine terrain heightmap:
- Create `pub struct HeightmapData` with `heights: Vec<f32>`, `width: u32`,
  `depth: u32`, `scale: f32`
- Use `noise::Fbm<noise::Perlin>` with multiple octaves to generate
  fractal terrain
- Configure noise parameters for alpine foothills: moderate frequency
  with high amplitude variation for hills
- Implement `pub fn generate_heightmap(width: u32, depth: u32, scale: f32) -> HeightmapData`
- Implement `pub fn sample_height(data: &HeightmapData, x: f32, z: f32) -> f32`
  for querying height at any world position (bilinear interpolation)
- Implement `pub fn sample_normal(data: &HeightmapData, x: f32, z: f32) -> Vec3`
  for querying surface normal (for slope detection)

### T005: Generate terrain mesh from heightmap

**File**: `src/terrain/mesh.rs`
**Status**: pending
**Depends on**: T004

Build a Bevy `Mesh` from the heightmap:
- Create `pub fn generate_terrain_mesh(heightmap: &HeightmapData) -> Mesh`
- Generate a subdivided plane (e.g., 256x256 vertices for 500x500 units)
- Set vertex positions with Y from heightmap
- Calculate vertex normals from heightmap gradients
- Generate UV coordinates for texture mapping
- Generate triangle indices

### T006: Add TerrainData resource and spawn terrain entity

**File**: `src/terrain/mod.rs`
**Status**: pending
**Depends on**: T005

Wire up terrain in the TerrainPlugin:
- Define `TerrainData` component holding `HeightmapData`
- In plugin `build`, add a startup system that:
  1. Generates heightmap via `generate_heightmap(256, 256, 500.0)`
  2. Generates mesh via `generate_terrain_mesh`
  3. Inserts mesh + basic `StandardMaterial` (green color placeholder)
  4. Spawns terrain entity with `Mesh3d`, `MeshMaterial3d`, `Transform`
  5. Inserts `TerrainData` as a `Resource` for height queries

### T007: Download and add terrain PBR textures [P]

**File**: `assets/textures/grass_*.png`, `assets/textures/rock_*.png`, `assets/textures/dirt_*.png`
**Status**: pending
**Depends on**: T003

Download PBR texture sets from Poly Haven (CC0):
- Grass ground texture: albedo, normal map
- Rock texture: albedo, normal map
- Dirt/soil texture: albedo, normal map
- Resize to 1024x1024 if needed for performance
- Save as PNG in `assets/textures/`

### T008: Implement terrain splatmap material

**File**: `src/terrain/material.rs`
**Status**: pending
**Depends on**: T006, T007

Create a terrain material that blends textures based on terrain properties:
- Define a custom material using Bevy's `Material` trait or
  `ExtendedMaterial` with custom shader
- Create terrain splatmap shader (`assets/shaders/terrain.wgsl`):
  - Input: grass, rock, dirt textures (albedo + normal each)
  - Blend weights based on: vertex height (grass→dirt→rock),
    surface slope (steep = rock, flat = grass)
  - Smooth transitions between texture regions
- Apply the material to the terrain entity replacing the placeholder
- Include triplanar mapping for rock on steep slopes to avoid stretching

---

## Phase 3: Lighting and Atmosphere (Scenario 1 - Launching the Demo)

### T009: Configure sun and directional shadows

**File**: `src/lighting/sun.rs`
**Status**: pending
**Depends on**: T002

Set up the sun as a directional light:
- Spawn `DirectionalLight` entity with:
  - Warm white color (e.g., `Color::srgb(1.0, 0.95, 0.85)`)
  - High intensity for outdoor daylight
  - `Transform` rotated to represent late-morning sun angle
  - `shadows_enabled: true`
- Configure `CascadeShadowConfig` or equivalent for cascaded shadow maps
  with at least 3 cascades covering near to far range

### T010: Set up atmosphere and ambient lighting [P]

**File**: `src/lighting/atmosphere.rs`
**Status**: pending
**Depends on**: T002

Configure Bevy 0.18 built-in atmosphere:
- Set up `Atmosphere` component or `ScatteringMedium` resource
  (check Bevy 0.18 API for exact type names)
- Configure for clear daytime sky with blue atmosphere
- If Bevy 0.18 atmosphere API differs from expected, fall back to:
  - `EnvironmentMapLight` with an HDRI sky image, OR
  - Simple sky color background + `AmbientLight`
- Add `AmbientLight` for soft fill (low intensity, slightly blue)

### T011: Wire up LightingPlugin

**File**: `src/lighting/mod.rs`
**Status**: pending
**Depends on**: T009, T010

Combine sun and atmosphere in the LightingPlugin:
- Register startup systems from `sun.rs` and `atmosphere.rs`
- Ensure lighting works with the terrain (shadows visible on terrain)

---

## Phase 4: Character (Scenario 2 - Walking Through the Landscape)

### T012: Source and add character model [P]

**File**: `assets/models/character.glb`
**Status**: pending
**Depends on**: T003

Obtain a stylized realistic humanoid character:
- Download from Quaternius, Mixamo, or similar CC0/free source
- Requirements: humanoid proportions, walk animation, idle animation
- Export/convert to glTF 2.0 (.glb) with embedded textures and animations
- If using Mixamo: download character + walk + idle animations, combine
  in Blender, export as single .glb
- Place in `assets/models/character.glb`
- Verify file loads correctly with a test Bevy scene

### T013: Implement character controller components

**File**: `src/character/controller.rs`
**Status**: pending
**Depends on**: T002

Define character ECS components and movement system:
- `#[derive(Component)] pub struct CharacterController { pub speed: f32 }`
- `#[derive(Component, Default)] pub struct MovementInput { pub direction: Vec2 }`
- `#[derive(Component, Default, PartialEq)] pub enum CharacterState { #[default] Idle, Walking }`
- System `read_movement_input`: read `KeyCode` (WASD + arrows) → update `MovementInput`
- System `apply_movement`: read `MovementInput`, move `Transform`:
  - Translate in XZ plane based on input direction and speed
  - Rotate character to face movement direction
  - Snap Y to terrain height (query `TerrainData` resource)
  - Clamp XZ position to terrain boundaries (invisible wall)
- System `update_character_state`: set `CharacterState` based on whether
  `MovementInput.direction` is zero or not

### T014: Spawn character and wire up CharacterPlugin

**File**: `src/character/mod.rs`
**Status**: pending
**Depends on**: T012, T013, T006

Wire up the CharacterPlugin:
- Startup system: load `character.glb` scene, spawn at terrain center
  position, add `CharacterController`, `MovementInput`, `CharacterState`
- Register `read_movement_input`, `apply_movement`,
  `update_character_state` systems in `Update` schedule
- Character Y position should match terrain height at spawn point

### T015: Implement character animation state machine

**File**: `src/character/animation.rs`
**Status**: pending
**Depends on**: T014

Animate the character based on movement state:
- On glTF scene load: extract `AnimationClip` handles for walk and idle
  from the loaded glTF (store as `Resource` or component)
- System `animate_character`: query `CharacterState` and `AnimationPlayer`
  - When state transitions to `Walking`: play walk clip in loop
  - When state transitions to `Idle`: play idle clip in loop
  - Use `play_with_transition(duration)` for smooth blending if available,
    otherwise crossfade manually
- Track previous state to detect transitions (avoid re-triggering same
  animation every frame)

---

## Phase 5: Camera (Scenario 4 - Camera Control)

### T016: Implement orbit camera component and systems

**File**: `src/camera/orbit.rs`
**Status**: pending
**Depends on**: T002

Create third-person orbit camera:
- `#[derive(Component)] pub struct OrbitCamera { pub distance: f32, pub yaw: f32, pub pitch: f32, pub target: Entity }`
- System `camera_mouse_input`: read `MouseMotion` events → update
  `OrbitCamera` yaw and pitch. Clamp pitch to prevent flipping
  (e.g., -80° to 80°). Only rotate when right mouse button held or
  always (choose appropriate UX).
- System `camera_follow`: compute camera position from target entity's
  `Transform` + orbit offset:
  - `position = target_pos + spherical_to_cartesian(distance, yaw, pitch)`
  - `Transform::looking_at(target_pos, Vec3::Y)`
  - Lerp position for smooth following
- System `camera_terrain_clamp`: ensure camera Y is at least
  `terrain_height + 1.0` at camera position to prevent underground view

### T017: Spawn camera and wire up CameraPlugin

**File**: `src/camera/mod.rs`
**Status**: pending
**Depends on**: T016, T014

Wire up the CameraPlugin:
- Startup system: spawn `Camera3d` entity with `OrbitCamera` component
  - Initial distance: ~10 units
  - Initial pitch: ~20° above horizontal
  - Set `target` to the character entity
- Register camera systems in `Update` schedule
- Ensure camera systems run after character movement systems
  (use system ordering or `after()`)

---

## Phase 6: Vegetation (Scenario 3 - Exploring the Environment)

### T018: Source and add vegetation models [P]

**File**: `assets/models/conifer_tree.glb`, `assets/models/alpine_shrub.glb`
**Status**: pending
**Depends on**: T003

Obtain open-source vegetation models:
- Conifer tree (pine/spruce): download from Quaternius Nature Pack (CC0)
  or OpenGameArt. Needs to be low-to-mid poly with PBR textures.
- Alpine shrub/bush: download from same source
- Convert to glTF 2.0 (.glb) with embedded textures
- Place in `assets/models/`
- Ensure models are properly scaled (tree ~5-10 unit height, shrub ~1-2)

### T019: Implement vegetation placement algorithm

**File**: `src/vegetation/placement.rs`
**Status**: pending
**Depends on**: T004

Procedurally place vegetation across the terrain:
- `pub fn generate_vegetation_positions(heightmap: &HeightmapData, count: usize, seed: u64) -> Vec<VegetationInstance>`
- `VegetationInstance { position: Vec3, rotation_y: f32, scale: f32, vegetation_type: VegetationType }`
- Use Poisson disk sampling or jittered grid for natural spacing
- Filter rules:
  - Skip positions where slope > threshold (rocky areas)
  - Skip positions above tree line height
  - Sparse density: ~200-400 total vegetation entities
- Random rotation (0-360°) and slight scale variation (0.8-1.2x)
- Mix of conifer trees (~70%) and alpine shrubs (~30%)

### T020: Spawn vegetation entities and wire up plugin

**File**: `src/vegetation/mod.rs`, `src/vegetation/types.rs`
**Status**: pending
**Depends on**: T018, T019, T006

Wire up the VegetationPlugin:
- Define `#[derive(Component)] pub enum VegetationType { ConiferTree, AlpineShrub }`
  in `types.rs`
- Startup system:
  1. Load conifer and shrub glTF scenes
  2. Generate positions via `generate_vegetation_positions`
  3. For each position: spawn entity with correct scene, `Transform`
     (position + rotation + scale), and `VegetationType` component

---

## Phase 7: Grass and Polish

### T021: Implement grass blade geometry

**File**: `src/grass/instancing.rs`
**Status**: pending
**Depends on**: T002

Create grass blade mesh and instancing setup:
- Generate a single grass blade mesh: a narrow quad or 3-triangle cross
  shape (two crossed quads for 3D appearance from any angle)
- Grass blade size: ~0.1 wide, ~0.3-0.5 tall
- Define `GrassSettings` resource: density, blade height range,
  wind sway speed/amplitude
- Define instance data struct: position (Vec3), height (f32),
  color_variation (f32)

### T022: Build grass instance buffer from terrain

**File**: `src/grass/instancing.rs`
**Status**: pending
**Depends on**: T021, T004

Generate grass instance positions:
- Sample terrain surface at regular intervals (spacing from GrassSettings)
- For each sample point:
  - Skip if slope > threshold (no grass on rock)
  - Skip if height > tree line
  - Add random offset within cell for natural look
  - Set blade height with slight randomness
  - Set color variation for visual diversity
- Build the instance buffer for GPU instanced rendering
- Implement distance-based culling: no grass instances beyond a max
  render distance (e.g., 100 units from camera)
- Target: thousands of instances while maintaining 60 FPS

### T023: Implement grass wind shader

**File**: `assets/shaders/grass.wgsl`, `src/grass/mod.rs`
**Status**: pending
**Depends on**: T022

Create custom grass rendering with wind animation:
- Write WGSL vertex shader for grass instances:
  - Displace top vertices using sine wave based on world position + time
  - Wind direction uniform for consistent sway direction
  - Amplitude decreases toward blade base (bottom vertices fixed)
- Fragment shader: simple green color with variation from instance data,
  basic diffuse lighting
- Register as custom `Material` in Bevy
- Wire up `GrassPlugin`: startup system spawns grass mesh with instances,
  register the custom material

### T024: Implement grass distance fade [P]

**File**: `src/grass/mod.rs`
**Status**: pending
**Depends on**: T023

Add distance-based grass rendering optimization:
- System that updates grass visibility based on camera distance
- Fade out grass alpha at far distances
- Reduce density at medium distances (skip every other instance)
- Completely cull grass beyond max render distance
- Update each frame based on camera position

### T025: Source and add grass blade texture [P]

**File**: `assets/textures/grass_blade.png`
**Status**: pending
**Depends on**: T003

Get a grass blade texture:
- Simple grass blade alpha texture (or generate procedurally)
- Small size (64x128 or similar)
- Used as alpha mask on grass blade quads
- Place in `assets/textures/`

### T026: Performance profiling and tuning

**File**: various
**Status**: pending
**Depends on**: T023, T020, T015, T017

Profile and optimize for 60 FPS target:
- Enable Bevy's frame time diagnostics (`FrameTimeDiagnosticsPlugin`)
- Identify frame time bottlenecks
- Tune parameters:
  - Grass instance count and render distance
  - Vegetation entity count
  - Shadow cascade distances and resolution
  - Terrain mesh subdivision level
- Verify 60 FPS on the development machine

### T027: Visual quality polish

**File**: various
**Status**: pending
**Depends on**: T026

Final visual quality pass:
- Adjust sun angle, color, intensity for best alpine daylight look
- Tune atmosphere scattering for natural sky color
- Adjust terrain texture blending thresholds
- Verify shadows look natural
- Check character scale relative to vegetation
- Ensure no visual glitches (z-fighting, texture seams, etc.)

### T028: Verify all success criteria

**File**: none (manual verification)
**Status**: pending
**Depends on**: T027

Verify each success criterion from the spec:
- [ ] Application launches and shows landscape within 10 seconds
- [ ] Character responds to WASD/arrow input immediately
- [ ] Walking animation plays smoothly
- [ ] Scene is recognizable as natural outdoor environment
- [ ] Camera follows smoothly without jarring movements
- [ ] Frame rate stays above 60 FPS
- [ ] At least two vegetation types are distinguishable
- [ ] Terrain has alpine foothills character
- [ ] Character stops at terrain boundaries

---

## Dependency Graph

```
T001 → T002 → T004 → T005 → T006 ──→ T008 → T011
  │      │                    │           ↑
  │      │                    ↓           │
  │      ├──→ T009 ──────→ T011      T007 (parallel)
  │      ├──→ T010 ──────→ T011
  │      │
  ↓      ├──→ T013 ──→ T014 → T015 → T017
T003     │               ↑
  │      ├──→ T016 ──→ T017
  ├──→ T007 (parallel)
  ├──→ T012 (parallel) ──→ T014
  ├──→ T018 (parallel) ──→ T020
  └──→ T025 (parallel)

T004 → T019 → T020
T006 → T020

T021 → T022 → T023 → T024
                  ↓
T023 + T020 + T015 + T017 → T026 → T027 → T028
```

## Parallel Execution Opportunities

**Wave 1** (after T001): T002, T003 [P]
**Wave 2** (after T002+T003): T004, T007, T009, T010, T012, T013, T016, T018, T025 [P]
**Wave 3** (after T004): T005, T019, T021 [P]
**Wave 4** (after T005): T006 [P with above if deps met]
**Wave 5** (after T006): T008, T014, T020 [P]
**Wave 6** (after T014): T015, T017 [P]
**Wave 7** (after T022): T023, T024 [P]
**Wave 8** (sequential): T026 → T027 → T028

## Implementation Strategy

**MVP**: Phases 1-4 (terrain + lighting = visible landscape)
**Core**: Phase 5 (character walks on terrain)
**Complete**: Phases 6-7 (vegetation, grass, polish)

Start with the terrain rendering to get visual feedback early. Add
character movement as the next milestone. Vegetation and grass are
visual enhancements that can be tuned last.
