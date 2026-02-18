# 3D Models for Bevy Game Project

This directory contains CC0/open-source 3D models in GLB format for use in the Bevy game project.

## Models

### 1. character.glb (428 KB)
- **Source**: KhronosGroup glTF Sample Assets - CesiumMan
- **License**: CC0 / Public Domain
- **URL**: https://github.com/KhronosGroup/glTF-Sample-Assets
- **Features**:
  - Animated humanoid character
  - 1 mesh, 22 nodes (rigged skeleton)
  - 1 animation (suitable for walk/idle cycles)
  - Fully rigged and ready for game use

### 2. conifer_tree.glb (21 KB)
- **Source**: Poly Haven - Pine Tree 01 (1K resolution)
- **License**: CC0 / Public Domain
- **URL**: https://polyhaven.com/a/pine_tree_01
- **Features**:
  - 3 tree meshes (variations)
  - 3 nodes
  - 12 primitives total
  - Low-poly pine tree suitable for forest scenes
  - Converted from GLTF to GLB format

### 3. alpine_shrub.glb (2.9 KB)
- **Source**: Procedurally generated
- **License**: CC0 / Public Domain
- **Features**:
  - Simple sphere-based bush model
  - 63 vertices, 96 triangles
  - Created using pygltflib
  - Basic low-poly shrub for vegetation

## Usage in Bevy

All models are in GLB (binary glTF) format and can be loaded in Bevy using:

```rust
commands.spawn(SceneBundle {
    scene: asset_server.load("models/character.glb#Scene0"),
    ..default()
});
```

## Additional Resources

### Free CC0 3D Model Sources:
- **Poly Haven**: https://polyhaven.com/models - Hundreds of CC0 models (HDRIs, textures, models)
- **Poly Pizza**: https://poly.pizza/ - 10,500+ free low-poly models
- **Quaternius**: https://quaternius.com/ - Game-ready CC0 model packs
- **Kenney**: https://kenney.nl/assets - 60,000+ game assets including 3D models
- **KhronosGroup glTF Samples**: https://github.com/KhronosGroup/glTF-Sample-Assets
- **OpenGameArt**: https://opengameart.org/ - Community-contributed game art

### Recommended Model Packs:
- Quaternius Stylized Nature MegaKit (110+ nature models): https://quaternius.itch.io/stylized-nature-megakit
- KayKit Character Packs (rigged and animated): https://kaylousberg.itch.io/kaykit-adventurers
- Kenney Nature Kit (330 assets): https://kenney.nl/assets/nature-kit

## Notes

- All models in this directory are CC0 licensed and free to use in personal and commercial projects
- No attribution required (though appreciated!)
- Models were downloaded on 2026-02-16
- The character model has one animation that can be used for basic movement
- Tree and shrub models are static (no animations)
