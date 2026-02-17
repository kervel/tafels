---
parent_branch: 002-math-tables-game
feature_number: 003
status: In Progress
created_at: 2026-02-17T00:00:00+01:00
---

# Feature: Multi-Exercise Scene Panels

## Overview

Transform the current single-exercise-at-a-time gameplay into a multi-exercise exploration experience. Multiple exercise groups are scattered across the landscape simultaneously as glowing beacons. The player explores the alpine landscape to discover these beacons, but the exercise content (question, answers, timer) is hidden until the player approaches. When the player enters a proximity trigger radius around a beacon, the exercise reveals itself — answer panels appear, the question and countdown timer become visible, and the clock starts ticking.

Exercises have a random world-lifetime: they appear and eventually vanish on their own if the player never approaches. There is no penalty for unvisited exercises disappearing. However, once the player triggers an exercise by proximity, the countdown timer starts and they must answer before time runs out — timeouts after engagement still cost coins.

The HUD is simplified to show only persistent game-level information: coins, combo multiplier, and round progress. All exercise-specific information (question, timer, answer choices) lives in the 3D world attached to each exercise group. This shifts the feel from a linear quiz conveyor belt to an open-world treasure hunt where math exercises are mystery beacons to discover and solve.

## Clarifications

### Session 2026-02-17

- Q: When does an exercise's countdown timer start ticking? → A: Timer starts on proximity — when the player enters a trigger radius around the exercise. Exercises have a random world-lifetime (spawn and vanish independently). Clear and playful visual feedback when entering proximity.
- Q: Do distant exercise timeouts reset the combo / deduct coins? → A: Timeouts cannot happen at a distance because the timer only starts on proximity. Unvisited exercises that vanish carry no penalty. Once engaged (proximity triggered), timeout still deducts coins — this is fair since the player chose to engage.
- Q: What consequence for timeout after engagement vs. unvisited vanish? → A: Coins deducted on timeout (after proximity engagement). Unvisited exercises that vanish naturally have no penalty.
- Q: Can players read the exercise from afar before approaching? → A: No — exercise text (question, answers) is hidden until the timer starts (proximity trigger). Only the beacon/glow is visible from a distance. This prevents cheating by reading the answer before engaging.
- Q: Should text billboard (always face camera) or have fixed facing? → A: Fixed facing direction — the player must walk around to the front of the exercise to see the text and panels. Proximity trigger requires being in front, not just nearby.

## User Scenarios

### Scenario 1: Starting a Round and Seeing Beacons

The player selects a difficulty and begins walking. They can see several glowing beacons scattered across the visible landscape at varying distances. The beacons are mysterious — the player cannot read what exercise awaits until they approach. The player heads toward the nearest beacon.

### Scenario 2: Approaching and Revealing an Exercise

As the player walks toward a beacon, they cross the proximity trigger radius. With playful visual feedback (e.g. the beacon transforms, panels unfold, a reveal animation), the exercise appears: answer panels with neon numbers, a floating question text (e.g. "7 x 8 = ?"), and a countdown timer. The clock is now ticking. The player aims and throws a ball at the correct panel.

### Scenario 3: Exercise Vanishes Before Player Arrives

While the player is walking toward a distant beacon or focused on another exercise, one of the beacons flickers and vanishes — its random world-lifetime expired. No coins are lost. A new beacon may spawn elsewhere. The player adjusts their route toward the remaining beacons.

### Scenario 4: Timeout After Engaging an Exercise

The player triggers an exercise but takes too long deciding or misses their throw. The countdown timer reaches zero. The panels flash with the timeout effect, particles play, and the exercise disappears. Coins are deducted. The combo resets. The player moves on to another beacon.

### Scenario 5: Answering Multiple Exercises in Quick Succession

The player approaches an area where two beacons are close together. They trigger the first, answer correctly, then quickly move to the second beacon and trigger it. Both answers come quickly, maintaining and building the combo streak. The combo multiplier rewards efficient exploration and fast answers.

### Scenario 6: New Beacons Replacing Resolved Ones

After the player answers an exercise or one vanishes, a new beacon spawns somewhere in the landscape after a short delay, maintaining the target number of beacons visible. The player always has multiple beacons to pursue.

### Scenario 7: Scanning the Landscape for Beacons

The player pauses and looks around. They can see several beacons at various distances. Some beacons may be flickering or fading (indicating their world-lifetime is running low and they may vanish soon). The player decides which beacons to prioritize based on proximity and visual urgency cues.

### Scenario 8: Game Over and Round Completion

The round ends when the total number of exercises have been resolved (answered or timed out after engagement), or when coins reach zero. Exercises that vanished unvisited do not count toward the round total. The game over / round complete screen shows the same statistics as before. The HUD coin/combo/progress counters are the player's primary indicators of how the round is going.

## Functional Requirements

### FR1: Multiple Simultaneous Exercise Beacons

- Multiple exercise beacons exist in the 3D world at the same time
- The target number of concurrent beacons is configurable (default: 3)
- When a beacon vanishes (world-lifetime expired) or an exercise is resolved, a replacement spawns after the cooldown period to maintain the target count
- Each beacon has a random world-lifetime — the duration it stays in the landscape before vanishing if the player never approaches
- The total number of exercises per round remains configurable (default: 20), tracked as exercises engaged (proximity triggered), not beacons spawned. Vanished unvisited beacons do not count toward the round total.

### FR2: Exercise Placement in the Landscape

- Exercise beacons spawn at scattered positions across the landscape within a reasonable radius around the player (approximately 30-60 meters)
- Beacons must not spawn too close to each other (minimum separation distance)
- Beacons must not spawn too close to the player (minimum 15 meters)
- Beacons must spawn on valid terrain (sampling the heightmap for ground level)
- New beacons spawn at positions that encourage exploration in different directions, not always directly ahead
- Beacons must be placed outside of dense vegetation clusters to remain visible and accessible

### FR3: Proximity Trigger and Exercise Reveal

- Each beacon has a proximity trigger zone: the player must be within range (e.g. 15-20 meters) AND in front of the exercise (within a forward-facing arc) to activate it
- When the player enters the trigger zone from the front, the exercise activates with clear and playful visual feedback (e.g. beacon transforms into panels, reveal animation, light burst)
- Upon activation: the answer panels appear, the question text becomes visible, and the countdown timer starts
- Before activation: only the beacon/glow is visible — no question text, no answer numbers, no timer. This prevents the player from reading the exercise at a distance and approaching with the answer already known
- The exercise panels and text face a fixed direction (determined at spawn). The player must approach from the front to read them — circling behind shows only the backs of the panels
- Once activated, the exercise cannot be deactivated by walking away — the timer continues regardless of player distance

### FR4: 3D Question Text Above Panels

- Each activated exercise displays its question text (e.g. "7 x 8 = ?") as a 3D rendered element floating above the answer panels
- The question text uses the same neon bitmap texture rendering approach as the answer panel numbers
- The question text is positioned centered above the panel arc, high enough to be readable at the engagement distance
- The text faces a fixed direction (same as the answer panels). The player must approach from the front to read it
- The question text is only visible after the exercise is activated (proximity triggered)

### FR5: 3D Timer Display Above Panels

- Each activated exercise displays its own countdown timer as a 3D rendered element near the question text
- The timer text shows remaining seconds (e.g. "10s")
- The timer text changes color as time runs low: normal color when above 50% time, warning color when between 25%-50%, urgent color when below 25%
- The timer is rendered using the same 3D text approach as the question
- The timer is only visible after the exercise is activated (proximity triggered)

### FR6: Beacon Visual Design

- Before activation, each exercise appears as a visual beacon — a glowing pillar, orb, or similar attention-grabbing effect
- The beacon must be visible from the spawn radius (30-60m) against the landscape
- As the beacon's world-lifetime nears expiration, it shows visual urgency cues (e.g. flickering, pulsing, fading) to signal it may vanish soon
- The beacon design should feel playful and inviting, encouraging the player to approach and discover what exercise awaits

### FR7: Simplified HUD

- The HUD no longer shows the exercise question text, countdown timer, or timer bar
- The HUD retains: coin counter, combo multiplier display, round progress counter (e.g. "5 / 20")
- The feedback popup system (e.g. "Correct! +5 coins") is retained on the HUD
- The HUD layout is adjusted to be minimal and unobtrusive since most game information is now in the 3D world

### FR8: Exercise Lifecycle

- Each exercise follows this lifecycle:
  1. **Beacon** — visible in the world as a glow/pillar. World-lifetime countdown active. Question hidden.
  2. **Activated** — player enters proximity trigger. Beacon transforms to reveal panels, question, and answer timer. Answer timer counting down.
  3. **Resolved** — answered correctly, answered incorrectly, or timed out. Particle effects play, exercise despawns.
- When an exercise is answered correctly: celebratory particles, coin spawns at exercise location, exercise despawns
- When an exercise is answered incorrectly: error particles, coin penalty, correct answer briefly highlighted, exercise despawns
- When an activated exercise times out: timeout particles, coin penalty, combo reset, exercise despawns
- When a beacon's world-lifetime expires (never activated): the beacon quietly vanishes with a subtle fade/dissolve. No coin penalty. Does not count toward round total.
- After any resolution or vanish, a new beacon spawns (if the round's engagement budget permits) after the standard cooldown

### FR9: Retained Mechanics

- The ball throwing mechanic, crosshair, projectile physics, and panel collision remain unchanged
- The scoring system (coin rewards, penalties, combo multiplier) remains unchanged
- The difficulty system (table ranges, timer durations) remains unchanged
- The menu screen, game over screen, and difficulty selection remain unchanged
- Vegetation collision, camera behavior, and character controller remain unchanged

## Success Criteria

- Players can see at least 3 exercise beacons visible in the landscape simultaneously during active gameplay
- Beacons are visible from at least 30 meters away as glowing elements against the landscape
- Exercise content (question, answers, timer) is completely hidden until the player enters the proximity trigger radius
- A clear visual reveal animation plays when the player triggers an exercise
- The HUD shows only coins, combo, and progress — no exercise-specific question or timer
- Exercises spawn in varied directions around the player, encouraging exploration rather than walking in a straight line
- When one exercise is resolved or a beacon vanishes, a replacement appears within the cooldown period, maintaining the target concurrent count
- Each activated exercise's timer counts down independently; answering one exercise does not affect other exercises' timers
- Unvisited beacons that vanish naturally do not deduct coins or count toward the round total
- Timeouts on activated exercises correctly deduct coins and reset combo
- The game maintains playable frame rates (>20 FPS) on the target hardware (Intel Iris Xe integrated GPU) with 3 concurrent exercise groups
- A complete round of 20 exercises can be played through with the multi-exercise system without crashes or state inconsistencies
- The combo system works across exercises — quickly answering multiple nearby exercises builds combo as before

## Key Entities

### ExerciseBeacon

- exercise_id: unique identifier per exercise instance
- exercise_data: the math problem (operation, operands, correct answer, distractor choices) — generated at spawn but hidden until activation
- world_position: where this beacon is placed in the landscape
- world_lifetime_remaining: how long until this beacon vanishes if not activated
- state: Beacon (dormant, visible as glow), Activated (timer running, panels visible), Resolved (answered/timed out)

### ActivatedExercise

- beacon_id: reference to the parent ExerciseBeacon
- time_remaining: countdown from activation moment
- time_limit: based on difficulty
- resolution: Correct, Incorrect, or TimedOut (set on resolution)

### ActiveExercises (replaces singular ActiveExercise)

- beacons: collection of currently active ExerciseBeacon instances (both dormant and activated)
- total_engaged: how many exercises have been activated this round (tracks toward round limit)
- target_concurrent: how many beacons should be active at once (default: 3)

### ExerciseWorldDisplay

- beacon_entity: the visibility beacon/glow (visible before activation)
- question_entity: the 3D text showing the math question (visible after activation)
- timer_entity: the 3D text showing the countdown (visible after activation)
- panel_entities: the answer panel entities (visible after activation)
- pole_entities: the panel support poles (visible after activation)

## Assumptions

- The existing neon bitmap texture rendering system can be extended to render question strings (e.g. "7 x 8 = ?") and timer text ("10s"), not just single numbers
- Billboard behavior for 3D text can be achieved by rotating text entities to face the camera each frame
- Three concurrent exercise groups with their panels, text, and lights will stay within the performance budget on Intel Iris Xe — approximately 3x the current single-exercise entity count; dormant beacons are cheaper than activated exercises (fewer entities)
- The existing projectile collision system can handle checking against panels from multiple activated exercise groups without modification to the core collision logic
- The proximity trigger radius (15-20m) is large enough that the player enters it naturally while walking toward a beacon, but small enough that exercises cannot be read from a distance
- The spawn radius (30-60m) keeps beacons within the terrain's fog distance so they remain visible
- Beacon world-lifetime is long enough that the player can reasonably reach nearby beacons (e.g. 30-60 seconds), but short enough to create a sense of ephemeral opportunity
- The cooldown period between exercise resolution and replacement beacon spawn remains at 5 seconds
- Sound effects remain out of scope
- The existing particle effect system can handle effects from multiple exercise locations simultaneously
- The answer timer durations per difficulty (Easy=12s, Medium=10s, Hard=7s) remain appropriate since timers only start when the player is already at the exercise — no travel time needs to be compensated
