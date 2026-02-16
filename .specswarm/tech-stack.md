# 3dt Tech Stack

**Last Updated:** 2026-02-16
**Auto-Generated:** No

---

## Core Technologies

### Language

- **Rust** (stable toolchain, latest edition 2024)
  - All code MUST compile without warnings under `cargo clippy`
  - Use `cargo fmt` for consistent formatting

### Engine

- **Bevy** (latest 0.15.x)
  - Purpose: 3D rendering, ECS, asset management, input handling
  - Notes: Use official Bevy plugins where available before third-party

### Build Tool

- **Cargo** (Rust standard)
  - Use workspace layout if the project grows beyond a single crate
  - Enable LTO for release builds

---

## Rendering & Graphics

### Approved Libraries

- **bevy_pbr** (built-in) - Physically-based rendering for materials
- **bevy_gltf** (built-in) - Loading 3D models in glTF format
- **bevy_atmosphere** or equivalent - Sky and atmospheric scattering
- **bevy_terrain** or custom - Heightmap-based terrain generation

### Terrain & Vegetation

- Heightmap-based terrain with texture splatting
- GPU instancing for grass and vegetation rendering
- LOD system for distant vegetation

### Character & Animation

- glTF models with skeletal animation
- Bevy's animation system for walk cycles
- Third-person camera following the character

---

## Asset Pipeline

### 3D Models

- Format: glTF 2.0 (.glb preferred for binary packing)
- Source: Free/open-licensed assets only

### Textures

- Format: PNG or KTX2 (compressed)
- PBR texture sets: albedo, normal, roughness/metallic

### Audio (if applicable)

- Format: OGG Vorbis
- Ambient environmental sounds only

---

## Development Tools

### Required

- `rustfmt` - Code formatting
- `clippy` - Linting
- `cargo-watch` - Hot reloading during development

### Recommended

- `cargo-flamegraph` - Performance profiling
- `RenderDoc` or `wgpu` profiling - GPU debugging

---

## Prohibited Patterns

- Do NOT use `unsafe` without a `// SAFETY:` comment justifying it
- Do NOT use `Box<dyn Any>` for component storage (use Bevy ECS)
- Do NOT use synchronous file I/O in systems (use Bevy asset loader)
- Do NOT add non-Rust scripting layers (Lua, Python) for this demo
- Do NOT use deprecated Bevy APIs - follow migration guides

---

## Notes

- This file was created by `/specswarm:init`
- Update this file when adding new dependencies or patterns
- All new crate dependencies MUST be justified in the PR description
