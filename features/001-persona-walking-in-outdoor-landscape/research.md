# Research: Persona Walking in Outdoor Landscape

## R1: Bevy Version

- **Decision**: Use Bevy 0.18 (latest stable as of Feb 2026)
- **Rationale**: 0.18 includes built-in atmospheric rendering, improved
  PBR shading, and built-in camera controllers. Most mature version.
- **Alternatives**: Bevy 0.15-0.16 (older, but more third-party crate
  compatibility). Rejected: built-in atmosphere support in 0.18 is worth
  the trade-off of fewer compatible third-party crates.

## R2: Terrain Rendering

- **Decision**: Custom heightmap mesh generation using procedural noise
- **Rationale**: No mature, Bevy 0.18-compatible terrain crate exists.
  `bevy_mesh_terrain` (0.15.11) targets Bevy ^0.11 and compatibility is
  uncertain. `bevy_terrain` is unpublished research. A custom approach
  using the `noise` crate for heightmap generation and Bevy's mesh API
  for rendering is the most reliable path.
- **Approach**: Generate a heightmap using fractal Brownian motion (fBm)
  noise, build a subdivided plane mesh with vertex heights from the
  heightmap, apply PBR terrain materials with texture splatting based on
  height and slope (grass on flat, rock on steep, dirt transitions).
- **Alternatives**: bevy_mesh_terrain (uncertain compat), bevy_terrain
  (not published), pre-built heightmap image (less flexible).

## R3: Grass Rendering

- **Decision**: Custom GPU-instanced grass rendering
- **Rationale**: `warbler_grass` (Bevy 0.13) and `bevy_procedural_grass`
  (Bevy 0.12) are both incompatible with Bevy 0.18. Custom implementation
  using Bevy's instancing API is required.
- **Approach**: Render grass as instanced quad billboards or geometry
  blades placed on terrain surface. Use a custom shader for wind sway
  animation. Frustum cull and distance-fade for performance. Density
  controlled by terrain slope (less grass on steep/rocky areas).
- **Alternatives**: Wait for crate updates (blocks progress), downgrade
  Bevy version (loses atmosphere features).

## R4: Atmospheric Rendering

- **Decision**: Use Bevy 0.18 built-in atmosphere features
- **Rationale**: Bevy 0.18 includes ScatteringMedium asset and atmosphere
  occlusion with PBR shading integration. This replaces the need for the
  `bevy_atmosphere` crate (which only supports Bevy 0.16).
- **Alternatives**: bevy_atmosphere 0.13 (incompatible), custom sky
  shader (unnecessary given built-in support).

## R5: Character Model and Animation

- **Decision**: Source from Mixamo + Quaternius, use Bevy's built-in
  AnimationPlayer
- **Rationale**: Bevy natively loads glTF with skeletal animations.
  Mixamo provides free motion-captured walk/idle animations. Quaternius
  provides CC0 character models in glTF. Blend in Blender if needed.
- **Approach**: Download stylized realistic character from Quaternius or
  similar CC0 source. Apply Mixamo walk+idle animations. Export as
  single .glb file with embedded animations. Use AnimationPlayer to
  switch between clips based on movement state.
- **Alternatives**: characters3d.com, Creative Characters FREE pack.

## R6: Third-Person Camera

- **Decision**: Custom implementation using Bevy transform math
- **Rationale**: `bevy_third_person_camera` (0.3.0) targets Bevy ^0.16
  and compatibility with 0.18 is uncertain. A third-person orbiting
  camera is straightforward to implement with quaternion rotation around
  a pivot point. Avoids dependency risk.
- **Approach**: Camera entity tracks character position with an offset.
  Mouse input rotits yaw/pitch around character. Smooth interpolation
  via lerp. Terrain collision check to prevent camera going underground.
- **Alternatives**: bevy_third_person_camera (compat risk),
  smooth_bevy_cameras (same risk).

## R7: Vegetation Models

- **Decision**: Source from Quaternius (CC0) and OpenGameArt
- **Rationale**: Quaternius provides CC0 tree and bush models in glTF
  format. OpenGameArt has CC0 plant packs. Both are free and
  attribution-free.
- **Asset sources**:
  - Conifer trees: Quaternius Nature Pack (CC0, glTF)
  - Alpine shrubs: Quaternius or OpenGameArt CC0 plants
  - Terrain textures: Poly Haven (CC0, full PBR sets)
- **Alternatives**: CGTrader free models, TurboSquid free models (check
  licenses carefully).

## R8: Terrain Textures

- **Decision**: Poly Haven PBR texture sets
- **Rationale**: CC0 license, high quality, complete PBR maps (albedo,
  normal, roughness, AO). Covers grass, rock, and dirt surfaces needed
  for alpine terrain splatting.
- **Alternatives**: FreePBR.com, 3DTextures.me, TextureCan.
