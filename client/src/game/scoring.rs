use bevy::prelude::*;

use super::coins::PendingCoinSpawn;
use super::exercise::{ActiveExercise, ExerciseId, ExerciseState};
use super::panels::{AnswerPanel, PanelPole, QuestionText, TimerText};
use super::{ActiveExercises, GameSession, GameState};
use crate::effects::particles::ParticleBurstEvent;
use crate::hud::{AnswerFeedback, SpeedTier};

pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                process_pending_answer,
                check_game_over,
                check_round_complete,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Inserted by the projectile collision system when a ball hits a panel.
#[derive(Resource)]
pub struct PendingAnswer {
    #[allow(dead_code)]
    pub value: u32,
    pub is_correct: bool,
    pub hit_position: Vec3,
    pub exercise_id: u32,
}

/// Returns the combo multiplier for the given combo count.
fn combo_multiplier(combo: u32) -> f32 {
    match combo {
        0 | 1 => 1.0,
        2 => 1.5,
        _ => 2.0, // capped at x2 (was x3)
    }
}

fn process_pending_answer(
    mut commands: Commands,
    pending: Option<Res<PendingAnswer>>,
    mut exercises: Query<(Entity, &ExerciseId, &mut ActiveExercise)>,
    mut session: ResMut<GameSession>,
    panels: Query<(Entity, &AnswerPanel)>,
    poles: Query<(Entity, &ExerciseId), With<PanelPole>>,
    question_texts: Query<(Entity, &QuestionText)>,
    timer_texts: Query<(Entity, &TimerText)>,
    mut burst_events: MessageWriter<ParticleBurstEvent>,
) {
    let Some(answer) = pending else {
        return;
    };

    // Find the exercise entity matching the pending answer
    let Some((exercise_entity, _, mut exercise)) = exercises
        .iter_mut()
        .find(|(_, eid, _)| eid.0 == answer.exercise_id)
    else {
        commands.remove_resource::<PendingAnswer>();
        return;
    };

    if exercise.state != ExerciseState::Active {
        commands.remove_resource::<PendingAnswer>();
        return;
    }

    exercise.state = ExerciseState::Answered;
    session.current_index += 1;

    if answer.is_correct {
        session.correct_count += 1;
        session.combo += 1;
        if session.combo > session.max_combo {
            session.max_combo = session.combo;
        }

        let multiplier = combo_multiplier(session.combo);

        // Speed tier bonus
        let fraction_remaining = exercise.time_remaining / exercise.time_limit;
        let speed_tier = if fraction_remaining > 0.75 {
            SpeedTier::Lightning
        } else if fraction_remaining > 0.5 {
            SpeedTier::Fast
        } else {
            SpeedTier::Normal
        };

        let speed_bonus = match speed_tier {
            SpeedTier::Lightning => 3,
            SpeedTier::Fast => 1,
            SpeedTier::Normal => 0,
        };

        let base_reward = 3 + speed_bonus;
        let reward = (base_reward as f32 * multiplier).round() as i32;

        // Spawn a pending coin instead of adding coins directly
        commands.spawn(PendingCoinSpawn {
            position: answer.hit_position,
            value: reward,
            timer: 1.5,
        });

        commands.insert_resource(AnswerFeedback {
            correct: true,
            coins_delta: reward,
            combo: session.combo,
            speed_tier,
            hit_position: answer.hit_position,
        });

        // Scale particle effects by combo AND speed tier
        let combo_scale = 1.0 + (session.combo.min(3) as f32 - 1.0) * 0.25;
        let speed_scale = match speed_tier {
            SpeedTier::Lightning => 1.6,
            SpeedTier::Fast => 1.2,
            SpeedTier::Normal => 1.0,
        };
        let scale = combo_scale * speed_scale;

        match speed_tier {
            SpeedTier::Lightning => {
                // Multi-color neon burst — green, gold, cyan, magenta
                for &color in &[
                    Color::srgb(0.3, 1.0, 0.4),  // green
                    Color::srgb(1.0, 0.85, 0.0),  // gold
                    Color::srgb(0.2, 0.9, 1.0),   // cyan
                    Color::srgb(1.0, 0.3, 0.8),   // magenta
                ] {
                    burst_events.write(ParticleBurstEvent {
                        position: answer.hit_position,
                        color,
                        count: (25.0 * scale) as u32,
                        speed: 12.0 * scale,
                        lifetime: 2.5,
                        size: 0.25 * scale,
                    });
                }
            }
            SpeedTier::Fast => {
                // Green + gold burst
                burst_events.write(ParticleBurstEvent {
                    position: answer.hit_position,
                    color: Color::srgb(0.3, 1.0, 0.4),
                    count: (50.0 * scale) as u32,
                    speed: 10.0 * scale,
                    lifetime: 2.0,
                    size: 0.2 * scale,
                });
                burst_events.write(ParticleBurstEvent {
                    position: answer.hit_position,
                    color: Color::srgb(1.0, 0.85, 0.0),
                    count: (30.0 * scale) as u32,
                    speed: 8.0 * scale,
                    lifetime: 1.5,
                    size: 0.15 * scale,
                });
            }
            SpeedTier::Normal => {
                // Basic green burst
                burst_events.write(ParticleBurstEvent {
                    position: answer.hit_position,
                    color: Color::srgb(0.3, 1.0, 0.4),
                    count: (40.0 * scale) as u32,
                    speed: 8.0 * scale,
                    lifetime: 1.5,
                    size: 0.18 * scale,
                });
            }
        }
    } else {
        session.wrong_count += 1;
        session.combo = 0;
        session.coins -= 2;
        commands.insert_resource(AnswerFeedback {
            correct: false,
            coins_delta: -2,
            combo: 0,
            speed_tier: SpeedTier::Normal,
            hit_position: answer.hit_position,
        });

        // Red error burst
        burst_events.write(ParticleBurstEvent {
            position: answer.hit_position,
            color: Color::srgb(1.0, 0.15, 0.15),
            count: 40,
            speed: 7.0,
            lifetime: 1.2,
            size: 0.15,
        });
    }

    // Despawn panels, poles, and text matching this exercise
    for (entity, panel) in &panels {
        if panel.exercise_id == answer.exercise_id {
            commands.entity(entity).despawn();
        }
    }
    for (entity, eid) in &poles {
        if eid.0 == answer.exercise_id {
            commands.entity(entity).despawn();
        }
    }
    for (entity, qt) in &question_texts {
        if qt.exercise_id == answer.exercise_id {
            commands.entity(entity).despawn();
        }
    }
    for (entity, tt) in &timer_texts {
        if tt.exercise_id == answer.exercise_id {
            commands.entity(entity).despawn();
        }
    }

    // Despawn the exercise entity
    commands.entity(exercise_entity).despawn();
    commands.remove_resource::<PendingAnswer>();
}

fn check_game_over(session: Res<GameSession>, mut next_state: ResMut<NextState<GameState>>) {
    if session.coins <= 0 {
        next_state.set(GameState::GameOver);
    }
}

fn check_round_complete(
    session: Res<GameSession>,
    active_exercises_query: Query<&ActiveExercise>,
    active_exercises: Res<ActiveExercises>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Only check when no exercise is active (all exercises done)
    if !active_exercises_query.is_empty() {
        return;
    }
    if active_exercises.total_engaged >= session.total_exercises && session.coins > 0 {
        // Round complete - go to game over screen (will show stats)
        next_state.set(GameState::GameOver);
    }
}
