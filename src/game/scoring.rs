use bevy::prelude::*;

use super::exercise::{ActiveExercise, ExerciseState};
use super::{GameSession, GameState};
use super::panels::{AnswerPanel, PanelPole};
use crate::effects::particles::ParticleBurstEvent;
use crate::hud::AnswerFeedback;

pub struct ScoringPlugin;

impl Plugin for ScoringPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (process_pending_answer, check_game_over, check_round_complete)
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
}

fn process_pending_answer(
    mut commands: Commands,
    pending: Option<Res<PendingAnswer>>,
    active: Option<ResMut<ActiveExercise>>,
    mut session: ResMut<GameSession>,
    panels: Query<Entity, With<AnswerPanel>>,
    poles: Query<Entity, With<PanelPole>>,
    mut burst_events: MessageWriter<ParticleBurstEvent>,
) {
    let Some(answer) = pending else {
        return;
    };
    let Some(mut exercise) = active else {
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
        let mut reward = 3;

        // Speed bonus
        let fraction_remaining = exercise.time_remaining / exercise.time_limit;
        if fraction_remaining > 0.75 {
            reward += 2;
        } else if fraction_remaining > 0.5 {
            reward += 1;
        }

        session.coins += reward;
        commands.insert_resource(AnswerFeedback {
            correct: true,
            coins_delta: reward,
        });

        // Green/gold celebration burst - big and dramatic
        burst_events.write(ParticleBurstEvent {
            position: answer.hit_position,
            color: Color::srgb(0.3, 1.0, 0.4),
            count: 60,
            speed: 10.0,
            lifetime: 2.0,
            size: 0.2,
        });
        // Extra gold sparkles
        burst_events.write(ParticleBurstEvent {
            position: answer.hit_position,
            color: Color::srgb(1.0, 0.85, 0.0),
            count: 40,
            speed: 8.0,
            lifetime: 1.5,
            size: 0.15,
        });
    } else {
        session.wrong_count += 1;
        session.coins -= 2;
        commands.insert_resource(AnswerFeedback {
            correct: false,
            coins_delta: -2,
        });

        // Red error burst - visible but less celebratory
        burst_events.write(ParticleBurstEvent {
            position: answer.hit_position,
            color: Color::srgb(1.0, 0.15, 0.15),
            count: 40,
            speed: 7.0,
            lifetime: 1.2,
            size: 0.15,
        });
    }

    // Despawn panels and poles
    for entity in &panels {
        commands.entity(entity).despawn();
    }
    for entity in &poles {
        commands.entity(entity).despawn();
    }

    commands.remove_resource::<ActiveExercise>();
    commands.remove_resource::<PendingAnswer>();
}

fn check_game_over(session: Res<GameSession>, mut next_state: ResMut<NextState<GameState>>) {
    if session.coins <= 0 {
        next_state.set(GameState::GameOver);
    }
}

fn check_round_complete(
    session: Res<GameSession>,
    active: Option<Res<ActiveExercise>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Only check when no exercise is active (all exercises done)
    if active.is_some() {
        return;
    }
    if session.current_index >= session.total_exercises && session.coins > 0 {
        // Round complete - go to game over screen (will show stats)
        next_state.set(GameState::GameOver);
    }
}
