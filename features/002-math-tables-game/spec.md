---
parent_branch: 001-persona-walking-in-outdoor-landscape
feature_number: 002
status: In Progress
created_at: 2026-02-17T00:00:00+01:00
---

# Feature: Math Tables Game (Maaltafels & Deeltafels)

## Overview

Transform the alpine landscape walking demo into an educational math game for children. Players walk through the 3D world encountering floating multiplication and division table exercises displayed as neon-lit multiple-choice questions. The player must aim and throw a ball at the correct answer before time runs out. A coin-based scoring system rewards correct and fast answers while penalizing mistakes and timeouts. Running out of coins triggers a game over sequence.

The visual style embraces a cartoon-neon aesthetic: the existing daylight landscape with oversized stylized vegetation is retained, but answer panels and UI elements glow with neon light effects. Particle bursts celebrate correct answers and warn of mistakes, making the experience visually rewarding and engaging for children.

## Clarifications

### Session 2026-02-17

- Q: Should exercises spawn on a timer, walk-triggered, or both? → A: Both combined — cooldown timer after each exercise; spawns at player position if they haven't walked far enough when timer expires
- Q: Should harder difficulty also reduce the answer timer? → A: Yes — Easy=12s, Medium=10s, Hard=7s, combined with harder table ranges

## User Scenarios

### Scenario 1: Starting a New Game

A child launches the game and sees the character standing in the alpine landscape. A start screen or initial prompt allows them to select a difficulty level. Each difficulty changes both the table range and the answer timer: Easy (tables 2-5, 12 seconds), Medium (tables 2-9, 10 seconds), Hard (tables 2-12, 7 seconds). The game begins with a starting pool of coins displayed on the HUD.

### Scenario 2: Encountering an Exercise

While walking through the landscape, the player approaches a cluster of floating answer panels arranged in a semicircle ahead of them. Each panel displays a number (a possible answer) rendered in glowing neon style. Above the panels, the math question floats (e.g. "7 x 8 = ?"). A countdown timer is visible. The player must aim at the correct panel and throw a ball to select their answer.

### Scenario 3: Answering Correctly

The player throws the ball at the panel showing "56". The ball hits, the panel explodes in a celebratory particle burst (sparks, stars), a satisfying sound cue plays, and coins are awarded. The amount of coins depends on how quickly they answered. The question disappears and the player continues walking to find the next exercise.

### Scenario 4: Answering Incorrectly

The player throws the ball at the wrong panel. The panel flashes red with an error particle effect, the correct answer is briefly highlighted in green, and coins are deducted. The exercise is dismissed and the player moves on.

### Scenario 5: Running Out of Time

The countdown timer reaches zero before the player throws. The panels flash and disappear with a timeout effect, coins are deducted (larger penalty than a wrong answer), and the exercise is lost. The player continues walking.

### Scenario 6: Game Over

The player's coin count drops to zero or below. The character performs a dramatic death animation, the screen shows a game over overlay with their statistics (total questions, correct answers, accuracy percentage, total time played). The player can restart or return to difficulty selection.

### Scenario 7: Completing a Round

After answering a configured number of exercises (e.g. 20 questions per round), the player sees a completion screen with their score, accuracy, and average response time. They can choose to play again with the same or different settings.

## Functional Requirements

### FR1: Exercise Generation

- The system generates multiplication exercises (e.g. 3 x 7 = ?) and division exercises (e.g. 56 / 8 = ?) based on the selected difficulty tier:
  - Easy: tables 2-5, 12-second timer
  - Medium: tables 2-9, 10-second timer
  - Hard: tables 2-12, 7-second timer
- Each exercise has exactly 4 answer choices: 1 correct and 3 plausible distractors
- Distractors are generated to be plausible wrong answers (e.g. off-by-one table entries, common mistakes) rather than random numbers
- Exercises are generated in a randomized sequence with no immediate repeats
- The system supports tables from 1 through 12

### FR2: Exercise Presentation

- Exercises appear as a set of 4 floating panels arranged in a semicircle close to the player (approximately 5-8 meters ahead), so the player does not need to walk long distances between exercises
- The question text (e.g. "6 x 9 = ?") floats above the answer panels
- Answer panels and question text are rendered with a neon glow effect (emissive material, bloom)
- A countdown timer bar is displayed near the question, visually depleting as time runs out
- After an exercise is completed, dismissed, or timed out, a short cooldown timer starts (e.g. 5 seconds). If the player has walked far enough (5-8m) before the timer expires, the next exercise spawns ahead of them. If not, the exercise spawns at the player's current position when the timer expires. This keeps the pace up without requiring long walks

### FR3: Ball Throwing Mechanic

- A crosshair is always visible at the center of the screen, indicating where the ball will be thrown
- The player presses spacebar to throw a ball toward the crosshair direction
- The ball travels in a visible arc toward the aimed panel
- Upon hitting a panel (or the ground), the ball bounces a couple of times with diminishing height, then disappears in a small particle burst
- If the ball hits an answer panel, that answer is selected and scored
- If the ball misses all panels and bounces away, no answer is registered and the player can throw again (within the time limit)
- Only one answer can be registered per exercise (first panel hit counts)

### FR4: Scoring System

- The player starts each game with a configurable starting coin count (default: 10 coins)
- Correct answer: +3 coins base, with a speed bonus of +1 to +2 coins for fast answers (answered within the first half of the timer)
- Wrong answer: -2 coins
- Timeout (no answer given): -3 coins
- The coin count is displayed prominently on the HUD at all times
- When coins reach 0 or below, the game over sequence triggers

### FR5: HUD and Interface

- A coin counter is always visible on screen showing the current coin count
- The countdown timer for the current exercise is visible when an exercise is active
- A question counter shows progress (e.g. "Question 5 / 20")
- Difficulty selection is available at game start
- Game over screen shows: total questions attempted, correct answers, accuracy %, coins earned
- A restart option is available from the game over screen

### FR6: Neon Visual Style

- Answer panels use emissive/unlit materials with bright neon colors (blues, pinks, greens, yellows)
- Bloom post-processing effect makes neon elements glow
- The existing daylight landscape, vegetation, skybox, and fog are retained as-is
- The overall aesthetic embraces the cartoon/stylized look of the existing scene

### FR7: Particle Effects

- Correct answer: celebratory burst of bright particles (sparks, stars) at the hit panel location
- Wrong answer: red/orange error particles at the hit panel
- Timeout: panels fade out with dissolving particles
- Coin gain: small upward-floating particle near the HUD coin counter
- Coin loss: downward-falling particle near the HUD coin counter
- Ball trail: light particle trail following the thrown ball

### FR8: Vegetation Collision

- The player character cannot walk through trees, boulders, or rock outcrops — these act as solid obstacles
- Grass and shrubs do not block movement
- Collision is based on a simple radius around each obstacle's position (not pixel-perfect mesh collision)
- When the player walks into an obstacle, they are pushed back to prevent overlap

### FR9: Camera Behavior

- The camera automatically positions itself behind the character, looking over the shoulder in the direction the character is facing
- When the character turns, the camera smoothly follows to remain behind them, ensuring the player always sees what is ahead
- The player can still adjust the camera angle with mouse drag for fine aiming, but the camera gently returns to the behind-the-character position when not being dragged
- The camera height and distance remain similar to the current setup but biased to look slightly forward (toward where exercises spawn)

### FR10: Game Flow

- Game starts at difficulty selection
- Player walks forward; exercises spawn ahead in sequence
- After the configured number of exercises (default: 20), the round ends with a summary screen
- Game over can occur at any point if coins reach zero
- Player can restart or change difficulty after completion or game over

## Success Criteria

- Children aged 6-12 can understand and play the game without adult instruction after a brief initial prompt
- A complete round of 20 questions can be played in under 10 minutes
- The correct answer is always present among the 4 choices
- Distractor answers are plausible (within the same multiplication table range, not obviously absurd)
- The timer provides enough pressure to encourage speed without being frustratingly short (7-12 seconds depending on difficulty level)
- Neon-lit answer panels are clearly readable from the player's approach distance
- Particle effects play immediately upon answer events with no perceptible delay
- The coin count accurately reflects all scoring events
- Game over triggers reliably when coins reach zero
- The game maintains playable frame rates (>25 FPS) on the target hardware (Intel Iris Xe integrated GPU)

## Key Entities

### Exercise
- operation: multiplication or division
- operand_a: first number (1-12)
- operand_b: second number (1-12)
- correct_answer: computed result
- distractor_answers: 3 plausible wrong answers
- time_limit: seconds allowed to answer

### GameSession
- difficulty: selected tier (Easy/Medium/Hard) — determines table range and timer duration
- total_exercises: number of exercises per round
- current_exercise_index: progress counter
- coins: current coin count
- correct_count: number of correct answers
- wrong_count: number of wrong answers
- timeout_count: number of timeouts
- start_time: when the session began

### AnswerPanel
- position: world-space location
- answer_value: the number displayed
- is_correct: whether this is the correct answer
- neon_color: the glow color assigned

## Assumptions

- The existing character controller (WASD movement) remains as the movement system; camera orbit is reworked to auto-follow behind the character
- The existing terrain, vegetation, lighting, fog, and skybox remain as the game world backdrop
- Tables of 1-12 cover the standard primary school curriculum for multiplication and division
- Division exercises only use clean divisions (no remainders) — e.g. 56/8=7, not 57/8
- Timer values per difficulty (Easy=12s, Medium=10s, Hard=7s) balance challenge and accessibility for children aged 6-12
- Starting with 10 coins gives enough runway for children to learn the mechanic before facing game over pressure
- 4 answer choices provides a good balance between too few (coin-flip guessing) and too many (overwhelming)
- Ball throwing uses spacebar with a screen-center crosshair; camera orbit remains on left-click drag but auto-returns behind the character
- Sound effects are out of scope for the initial implementation but the system should not preclude adding them later
- The neon glow effect can be achieved with Bevy's emissive materials and bloom post-processing without custom shaders
