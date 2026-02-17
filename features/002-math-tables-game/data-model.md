# Data Model: Math Tables Game

## ECS Components

### Game State

```
GameState (States enum):
  - Menu
  - Playing
  - GameOver
```

### Game Session (Resource)

```
GameSession:
  - difficulty: Difficulty (Easy | Medium | Hard)
  - total_exercises: u32 (default: 20)
  - current_index: u32
  - coins: i32 (default: 10)
  - correct_count: u32
  - wrong_count: u32
  - timeout_count: u32
  - start_time: f64
```

### Difficulty (enum)

```
Difficulty:
  - Easy:   tables 2-5,  timer 12s
  - Medium: tables 2-9,  timer 10s
  - Hard:   tables 2-12, timer 7s
```

### Exercise (Resource — current active exercise)

```
ActiveExercise:
  - operation: Operation (Multiply | Divide)
  - operand_a: u32
  - operand_b: u32
  - correct_answer: u32
  - choices: [u32; 4] (shuffled, one is correct)
  - time_remaining: f32
  - time_limit: f32
  - state: ExerciseState (Active | Answered | TimedOut)
  - spawn_cooldown: Option<f32> (countdown to next exercise)
```

### Answer Panel (Component)

```
AnswerPanel:
  - value: u32
  - is_correct: bool
  - panel_index: u8 (0-3, position in semicircle)
```

### Projectile (Component)

```
Projectile:
  - (marker component)

Velocity:
  - Vec3

BounceCount:
  - count: u8
  - max_bounces: u8 (default: 3)

ProjectileLifetime:
  - remaining: f32 (default: 5.0s)
```

### HUD Markers (Components)

```
CoinDisplay (marker)
TimerDisplay (marker)
QuestionDisplay (marker)
ProgressDisplay (marker)
CrosshairMarker (marker)
```

### Collision (Component)

```
VegetationCollider:
  - radius: f32
```

## Entity Relationships

```
Camera
  └── has: Bloom, DistanceFog, Skybox, OrbitCamera

Character
  ├── has: CharacterMarker, CharacterController, MovementInput, CharacterState
  └── has: CharacterAnimations

AnswerPanel (x4 per exercise)
  ├── has: AnswerPanel, Mesh3d, MeshMaterial3d, DespawnOnExit(Playing)
  └── tracked by: ActiveExercise

Projectile
  ├── has: Projectile, Velocity, BounceCount, ProjectileLifetime
  ├── has: Mesh3d, MeshMaterial3d, DespawnOnExit(Playing)
  └── collision target: AnswerPanel

Vegetation (existing)
  ├── has: VegetationType, WindSway
  └── new: VegetationCollider (on trees + rocks only)

HUD Root
  ├── CoinDisplay (text node)
  ├── TimerDisplay (progress bar)
  ├── ProgressDisplay (text node)
  ├── QuestionDisplay (text node, world-tracked position)
  └── CrosshairMarker (centered dot/circle)
```

## State Transitions

```
                    ┌─────────┐
          start     │  Menu   │
          ─────────►│ (select │
                    │  diff)  │
                    └────┬────┘
                         │ select difficulty
                         ▼
                    ┌─────────┐
                    │ Playing │◄──── restart
                    │         │
                    └──┬───┬──┘
                       │   │
            coins=0    │   │  exercises done
                       ▼   ▼
                 ┌──────┐ ┌──────────┐
                 │Game  │ │Round     │
                 │Over  │ │Complete  │
                 └──┬───┘ └────┬─────┘
                    │          │
                    └──► Menu ◄┘
```

## Exercise Lifecycle

```
[Cooldown] ──timer/walk──► [Spawning] ──panels appear──► [Active]
                                                            │
                                          ┌────────┬────────┤
                                          ▼        ▼        ▼
                                      [Correct] [Wrong] [Timeout]
                                          │        │        │
                                          └────────┴────────┘
                                                   │
                                                   ▼
                                              [Cooldown]
```
