# Implementation Plan: Persona Walking in Outdoor Landscape

## Technical Context

- **Language**: Rust (stable, edition 2024)
- **Engine**: Bevy 0.18
- **Graphics**: wgpu (Vulkan primary backend)
- **Asset format**: glTF 2.0 (.glb)
- **Terrain**: Custom heightmap mesh + noise crate for procedural generation
- **Grass**: Custom GPU-instanced rendering with wind shader
- **Atmosphere**: Bevy 0.18 built-in ScatteringMedium
- **Animation**: Bevy built-in AnimationPlayer for glTF skeletal animation
- **Camera**: Custom third-person orbit camera

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| P1: Idiomatic Rust | PASS | Standard Rust patterns, no unsafe needed |
| P2: ECS-First | PASS | All logic in Bevy systems and plugins |
| P3: Performance-Aware | PASS | GPU instancing for grass, LOD planned |
| P4: Modularity | PASS | Separate plugins: terrain, vegetation, character, camera, lighting |
| P5: Readable Over Clever | PASS | Explicit systems, minimal macro use |

## Tech Stack Compliance Report

### Approved Technologies (already in stack)

- Rust (stable toolchain)
- Bevy 0.18
- Cargo build system
- bevy_pbr (built-in)
- bevy_gltf (built-in)
- glTF 2.0 assets
- PBR textures from Poly Haven

### New Technologies (auto-added)

- **noise** crate - Procedural heightmap generation via fractal noise
  - No conflicts detected
  - Purpose: Generate alpine terrain heightmaps

### No Conflicts or Prohibited Technologies Detected

## Architecture: Plugin Structure

```
src/
  main.rs              - App setup, plugin registration
  terrain/
    mod.rs             - TerrainPlugin
    heightmap.rs       - Procedural heightmap generation
    mesh.rs            - Mesh generation from heightmap
    material.rs        - Terrain splatmap material
  vegetation/
    mod.rs             - VegetationPlugin
    placement.rs       - Procedural vegetation placement
    types.rs           - Vegetation type definitions
  grass/
    mod.rs             - GrassPlugin
    instancing.rs      - GPU instance buffer management
    shader.rs          - Grass wind sway shader
  character/
    mod.rs             - CharacterPlugin
    controller.rs      - Movement and state management
    animation.rs       - Animation state machine
  camera/
    mod.rs             - CameraPlugin
    orbit.rs           - Third-person orbit camera logic
  lighting/
    mod.rs             - LightingPlugin
    sun.rs             - Directional light and shadows
    atmosphere.rs      - Sky and atmospheric scattering setup
assets/
  models/
    character.glb      - Humanoid model with animations
    conifer_tree.glb   - Pine/spruce tree model
    alpine_shrub.glb   - Low alpine bush model
  textures/
    grass_albedo.png   - Grass PBR albedo
    grass_normal.png   - Grass PBR normal map
    rock_albedo.png    - Rock PBR albedo
    rock_normal.png    - Rock PBR normal map
    dirt_albedo.png    - Dirt PBR albedo
    dirt_normal.png    - Dirt PBR normal map
```

## Implementation Phases

### Phase 1: Project Scaffold and Basic Rendering

1. Initialize Cargo project with Bevy 0.18 dependency
2. Create plugin module structure (empty plugins)
3. Set up basic Bevy app with window, default plugins
4. Add a ground plane and directional light to verify rendering works

### Phase 2: Terrain System

1. Add `noise` crate dependency
2. Implement heightmap generation using fBm noise (alpine profile)
3. Generate subdivided plane mesh with vertex heights from heightmap
4. Apply basic PBR material (single grass texture initially)
5. Implement terrain height query function (for character/camera)
6. Add terrain boundary invisible wall (clamp character position)

### Phase 3: Terrain Materials

1. Download PBR texture sets from Poly Haven (grass, rock, dirt)
2. Implement splatmap-based terrain material (custom shader)
3. Blend textures based on height (grass low, rock high) and slope
   (rock on steep, grass on flat)
4. Apply normal maps for surface detail

### Phase 4: Lighting and Atmosphere

1. Configure DirectionalLight as sun (warm color, angled for shadows)
2. Set up CascadeShadowConfig for multi-distance shadow quality
3. Configure Bevy 0.18 built-in atmosphere (ScatteringMedium)
4. Add ambient light for soft fill
5. Tune lighting for convincing alpine daylight

### Phase 5: Character

1. Source and prepare character model (Quaternius/Mixamo → .glb)
2. Load character glTF scene into Bevy
3. Implement CharacterController component and movement system
4. Read WASD/arrow key input → MovementInput component
5. Apply movement: translate character, rotate to face direction
6. Snap character Y position to terrain height
7. Clamp character position to terrain boundaries

### Phase 6: Character Animation

1. Extract animation clip handles from loaded glTF
2. Implement animation state machine (Idle ↔ Walking)
3. Trigger walk animation on movement input, idle when stopped
4. Smooth transitions between animation states

### Phase 7: Third-Person Camera

1. Implement OrbitCamera component (distance, yaw, pitch)
2. Camera follows character position with offset
3. Mouse input rotates yaw/pitch around character
4. Smooth camera movement via interpolation
5. Prevent camera from going below terrain surface

### Phase 8: Vegetation

1. Source conifer tree and alpine shrub models (Quaternius CC0)
2. Load vegetation glTF models as assets
3. Implement procedural placement algorithm:
   - Scatter positions using Poisson disk sampling
   - Filter by terrain slope (no trees on steep rock)
   - Filter by height (vegetation below tree line)
   - Random rotation and slight scale variation
4. Spawn vegetation entities at computed positions

### Phase 9: Grass Rendering

1. Implement grass blade geometry (quad or triangle strip)
2. Build GPU instance buffer with positions sampled from terrain
3. Custom vertex shader for wind sway animation
4. Distance-based density falloff and fade-out
5. Skip grass on steep slopes and rocky areas
6. Tune density for 60 FPS performance target

### Phase 10: Polish and Performance

1. Profile frame time, identify bottlenecks
2. Add LOD for distant vegetation (simpler meshes or billboards)
3. Frustum culling verification
4. Tune grass density, vegetation count, shadow distances
5. Verify 60 FPS on mid-range hardware
6. Final visual quality pass (lighting, colors, atmosphere)

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Bevy 0.18 API changes from docs | Medium | Check Bevy 0.18 migration guide, use examples |
| Grass rendering performance | High | Start with low density, profile, increase gradually |
| Custom terrain shader complexity | Medium | Start with basic material, iterate on splatting |
| Asset compatibility (glTF) | Low | Test asset loading early in Phase 1 |
| Bevy atmosphere API uncertainty | Medium | Fall back to simple sky color if needed |

## Dependencies

```toml
[dependencies]
bevy = "0.18"
noise = "0.9"
rand = "0.8"
```
