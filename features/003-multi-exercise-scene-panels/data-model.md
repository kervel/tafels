# Data Model: Multi-Exercise Scene Panels

## New Components

### ExerciseId
```
ExerciseId(u32)
```
- Unique identifier per exercise instance within a round
- Assigned sequentially from ActiveExercises.next_exercise_id
- Used to link AnswerPanels and PanelPoles to their parent exercise

### BeaconState
```
enum BeaconState {
    Dormant,      // Visible as beacon glow, question hidden, world-lifetime ticking
    Activated,    // Player triggered proximity, panels visible, answer timer ticking
    Resolved,     // Answered or timed out, pending despawn
}
```
- Component on exercise beacon entities
- State transitions: Dormant → Activated (proximity), Dormant → despawn (world-lifetime expired), Activated → Resolved (answered/timed out)

### WorldLifetime
```
WorldLifetime {
    remaining: f32,    // Seconds until beacon vanishes (random 30-60s at spawn)
}
```
- Component on exercise beacon entities
- Only ticks when BeaconState::Dormant
- When reaches zero: beacon despawns silently, no penalty

### BeaconVisual
```
BeaconVisual
```
- Marker component on the visual mesh entity (orb/pillar) that is a child of the exercise beacon
- Despawned when exercise transitions from Dormant to Activated

### QuestionText
```
QuestionText {
    exercise_id: u32,
}
```
- Marker component on the 3D question text mesh entity
- Fixed facing direction (same rotation as answer panels, determined at spawn)
- Spawned on exercise activation, despawned on exercise resolution

### TimerText
```
TimerText {
    exercise_id: u32,
    prerendered_textures: Vec<Handle<Image>>,    // One per second value ("12s" ... "1s")
    current_second: u32,                          // Tracks which texture is active
}
```
- Marker component on the 3D timer text mesh entity
- Fixed facing direction (same rotation as answer panels)
- Texture swapped each second from prerendered cache
- Color variants: green (>50%), yellow (25-50%), red (<25%)

## Modified Components

### AnswerPanel (modified)
```
AnswerPanel {
    value: u32,           // The answer number displayed (existing)
    is_correct: bool,     // Whether this is the correct answer (existing)
    panel_index: u8,      // Index 0-3 (existing)
    exercise_id: u32,     // NEW: links panel to parent exercise
}
```

### ActiveExercise (refactored from Resource to Component)
```
ActiveExercise {
    operation: Operation,
    operand_a: u32,
    operand_b: u32,
    correct_answer: u32,
    choices: [u32; 4],
    time_remaining: f32,
    time_limit: f32,
    state: ExerciseState,
}
```
- Previously: single Resource (only one exercise)
- Now: Component on exercise beacon entities (multiple concurrent)
- ExerciseState enum unchanged: Active, Answered, TimedOut

## New Resources

### ActiveExercises
```
ActiveExercises {
    total_engaged: u32,       // Exercises activated this round (counts toward round limit)
    total_spawned: u32,       // Total beacons created (including vanished)
    target_concurrent: u32,   // How many beacons should exist at once (default: 3)
    next_exercise_id: u32,    // Counter for unique ExerciseId values
}
```
- Inserted on OnEnter(GameState::Playing), reset each round
- Replaces the role of GameSession.current_index for round tracking

## Modified Resources

### PendingAnswer (modified)
```
PendingAnswer {
    value: u32,             // The answer value hit (existing)
    is_correct: bool,       // Whether correct (existing)
    hit_position: Vec3,     // Collision position (existing)
    exercise_id: u32,       // NEW: which exercise the panel belongs to
}
```

### GameSession (modified)
```
GameSession {
    difficulty: Difficulty,     // unchanged
    total_exercises: u32,       // unchanged (default: 20)
    current_index: u32,         // DEPRECATED — replaced by ActiveExercises.total_engaged
    coins: i32,                 // unchanged
    correct_count: u32,         // unchanged
    wrong_count: u32,           // unchanged
    timeout_count: u32,         // unchanged
    combo: u32,                 // unchanged
    max_combo: u32,             // unchanged
    start_time: f64,            // unchanged
}
```
- `current_index` can be removed or kept for backward compatibility with game over screen stats

## Entity Relationships

```
ExerciseBeacon Entity
├── Components: ExerciseId, BeaconState, WorldLifetime, ActiveExercise, Transform
│
├── Child: BeaconVisual Entity (during Dormant phase)
│   └── Components: BeaconVisual, Mesh3d, MeshMaterial3d, PointLight
│
├── Children (after Activation):
│   ├── QuestionText Entity
│   │   └── Components: QuestionText, Mesh3d, MeshMaterial3d, Transform
│   ├── TimerText Entity
│   │   └── Components: TimerText, Mesh3d, MeshMaterial3d, Transform
│   ├── AnswerPanel Entity × 4 (NOT children — separate entities for collision query)
│   │   └── Components: AnswerPanel, Mesh3d, MeshMaterial3d, Transform
│   │   └── Child: PointLight
│   └── PanelPole Entity × 4 (separate entities)
│       └── Components: PanelPole, ExerciseId, Mesh3d, MeshMaterial3d, Transform
```

Note: AnswerPanel and PanelPole entities are linked to their exercise via ExerciseId component, not via parent-child hierarchy. This preserves the existing collision query pattern that iterates flat over all panels.

## State Transitions

### Exercise Lifecycle

```
[Spawned] ─── BeaconState::Dormant
    │
    ├── WorldLifetime expires ──► [Despawn] (silent vanish, no penalty)
    │
    └── Player proximity ──► BeaconState::Activated
                                  │
                                  ├── Correct answer ──► BeaconState::Resolved
                                  │   (particles, +coins, combo++)
                                  │
                                  ├── Wrong answer ──► BeaconState::Resolved
                                  │   (particles, -coins, combo=0)
                                  │
                                  └── Timer expires ──► BeaconState::Resolved
                                      (particles, -coins, combo=0)
```

### Beacon Replenishment

```
Active beacon count < target_concurrent
    AND total_engaged < total_exercises (round budget remaining)
    AND cooldown elapsed
    ──► Spawn new beacon at random position
```

## Removed / Deprecated

- `ActiveExercise` as Resource (replaced by Component)
- `ExerciseSpawnState` resource (replaced by ActiveExercises + beacon.rs logic)
- `QuestionDisplay` HUD component
- `TimerDisplay` HUD component
- `TimerBar` HUD component
- `update_question_display` system
- `update_timer_display` system
