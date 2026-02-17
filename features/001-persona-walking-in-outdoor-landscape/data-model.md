# Data Model: ECS Entity-Component Architecture

## Entities and Components

### Character Entity

| Component | Type | Description |
|-----------|------|-------------|
| Transform | Built-in | Position, rotation, scale in world |
| SceneRoot | Built-in | Root of loaded glTF scene |
| AnimationPlayer | Built-in | Controls skeletal animation playback |
| CharacterController | Custom | Movement speed, grounded state |
| MovementInput | Custom | Current input direction (Vec2) |
| CharacterState | Custom enum | Idle, Walking |

### Terrain Entity

| Component | Type | Description |
|-----------|------|-------------|
| Transform | Built-in | World position of terrain chunk |
| Mesh | Built-in | Generated heightmap mesh |
| Material | Custom | Multi-texture splatted PBR material |
| TerrainData | Custom | Heightmap data, dimensions, scale |

### Vegetation Entity (per instance group)

| Component | Type | Description |
|-----------|------|-------------|
| Transform | Built-in | Position on terrain |
| SceneRoot | Built-in | Loaded vegetation glTF model |
| VegetationType | Custom enum | ConiferTree, AlpineShrub |

### Grass (GPU Instanced - not individual entities)

| Resource | Type | Description |
|----------|------|-------------|
| GrassSettings | Custom | Density, blade height range, sway params |
| GrassInstanceData | Custom | Instance buffer for GPU rendering |

### Camera Entity

| Component | Type | Description |
|-----------|------|-------------|
| Transform | Built-in | Camera position and orientation |
| Camera3d | Built-in | Projection settings |
| OrbitCamera | Custom | Distance, yaw, pitch, target entity |

### Lighting

| Component | Type | Description |
|-----------|------|-------------|
| DirectionalLight | Built-in | Sun direction, color, intensity |
| CascadeShadowConfig | Built-in | Shadow cascade distances |

## Custom Component Definitions

```
CharacterController { speed: f32, grounded: bool }
MovementInput { direction: Vec2 }
CharacterState { Idle | Walking }
TerrainData { heights: Vec<f32>, width: u32, depth: u32, scale: f32 }
VegetationType { ConiferTree | AlpineShrub }
OrbitCamera { distance: f32, yaw: f32, pitch: f32, target: Entity }
GrassSettings { density: f32, min_height: f32, max_height: f32, sway_speed: f32 }
```

## State Transitions

```
CharacterState: Idle --[input detected]--> Walking
CharacterState: Walking --[no input]--> Idle
```
