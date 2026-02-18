use bevy::prelude::*;

use super::coins::PendingCoinSpawn;
use super::exercise::{ActiveExercise, ExerciseId, ExerciseState};
use super::panels::{AnswerPanel, PanelPole, QuestionText, TimerText};
use super::{ActiveExercises, GameSession, GameState};
use crate::effects::particles::ParticleBurstEvent;
use crate::hud::AnswerFeedback;

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
        3 => 2.0,
        _ => 3.0,
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

        let mut base_reward = 3;

        // Speed bonus
        let fraction_remaining = exercise.time_remaining / exercise.time_limit;
        if fraction_remaining > 0.75 {
            base_reward += 2;
        } else if fraction_remaining > 0.5 {
            base_reward += 1;
        }

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
        });

        // Scale particle effects by combo level
        let combo_scale = 1.0 + (session.combo.min(4) as f32 - 1.0) * 0.3;
        let base_count = (60.0 * combo_scale) as u32;
        let base_speed = 10.0 * combo_scale;
        let base_size = 0.2 * combo_scale;

        // Green/gold celebration burst
        burst_events.write(ParticleBurstEvent {
            position: answer.hit_position,
            color: Color::srgb(0.3, 1.0, 0.4),
            count: base_count,
            speed: base_speed,
            lifetime: 2.0,
            size: base_size,
        });
        // Extra gold sparkles
        burst_events.write(ParticleBurstEvent {
            position: answer.hit_position,
            color: Color::srgb(1.0, 0.85, 0.0),
            count: (40.0 * combo_scale) as u32,
            speed: 8.0 * combo_scale,
            lifetime: 1.5,
            size: 0.15 * combo_scale,
        });
    } else {
        session.wrong_count += 1;
        session.combo = 0;
        session.coins -= 2;
        commands.insert_resource(AnswerFeedback {
            correct: false,
            coins_delta: -2,
            combo: 0,
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
