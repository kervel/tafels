use bevy::prelude::*;
use rand::prelude::*;

use super::difficulty::Difficulty;
use super::GameSession;
use crate::effects::particles::{ParticleStyle, StyledParticleEvent};

pub struct ExercisePlugin;

impl Plugin for ExercisePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (tick_exercise_timer, tick_round_timer)
                .run_if(in_state(super::GameState::Playing)),
        );
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct ExerciseId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExerciseState {
    Pending,
    Active,
    Answered,
    TimedOut,
}

#[derive(Component)]
pub struct ActiveExercise {
    pub operation: Operation,
    pub operand_a: u32,
    pub operand_b: u32,
    pub correct_answer: u32,
    pub choices: [u32; 4],
    pub time_remaining: f32,
    pub time_limit: f32,
    pub state: ExerciseState,
}

pub fn generate_exercise(difficulty: &Difficulty) -> ActiveExercise {
    let mut rng = rand::thread_rng();
    let (table_range, timer) = difficulty.config();

    let a = rng.r#gen_range(table_range.clone());
    let b = rng.r#gen_range(table_range.clone());

    let (operation, operand_a, operand_b, correct_answer) = if rng.r#gen::<bool>() {
        (Operation::Multiply, a, b, a * b)
    } else {
        let product = a * b;
        (Operation::Divide, product, a, b)
    };

    let mut distractors = Vec::new();
    let candidates = [
        correct_answer.wrapping_add(1),
        correct_answer.wrapping_sub(1),
        correct_answer.wrapping_add(a),
        correct_answer.wrapping_sub(a),
        correct_answer.wrapping_add(b),
        correct_answer.wrapping_sub(b),
        a.wrapping_mul(b.wrapping_add(1)),
        a.wrapping_mul(b.wrapping_sub(1)),
        correct_answer.wrapping_add(2),
    ];

    for &c in &candidates {
        if c > 0 && c != correct_answer && !distractors.contains(&c) {
            distractors.push(c);
            if distractors.len() == 3 {
                break;
            }
        }
    }

    let mut fallback = correct_answer + 3;
    while distractors.len() < 3 {
        if fallback != correct_answer && !distractors.contains(&fallback) {
            distractors.push(fallback);
        }
        fallback += 1;
    }

    let mut choices = [correct_answer, distractors[0], distractors[1], distractors[2]];
    for i in (1..4).rev() {
        let j = rng.r#gen_range(0..=i);
        choices.swap(i, j);
    }

    ActiveExercise {
        operation,
        operand_a,
        operand_b,
        correct_answer,
        choices,
        time_remaining: timer,
        time_limit: timer,
        state: ExerciseState::Pending,
    }
}

impl ActiveExercise {
    pub fn question_text(&self) -> String {
        match self.operation {
            Operation::Multiply => format!("{} x {} = ?", self.operand_a, self.operand_b),
            Operation::Divide => format!("{} / {} = ?", self.operand_a, self.operand_b),
        }
    }
}

fn tick_exercise_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut exercises: Query<(Entity, &ExerciseId, &mut ActiveExercise, &super::beacon::BeaconState)>,
    mut session: ResMut<GameSession>,
    panels: Query<(Entity, &Transform, &super::panels::AnswerPanel)>,
    poles: Query<(Entity, &ExerciseId), With<super::panels::PanelPole>>,
    question_texts: Query<(Entity, &super::panels::QuestionText)>,
    timer_texts: Query<(Entity, &super::panels::TimerText)>,
    mut styled_events: MessageWriter<StyledParticleEvent>,
) {
    for (exercise_entity, exercise_eid, mut exercise, beacon_state) in &mut exercises {
        // Only tick timer for activated beacons
        if *beacon_state != super::beacon::BeaconState::Activated {
            continue;
        }
        if exercise.state != ExerciseState::Active {
            continue;
        }

        exercise.time_remaining -= time.delta_secs();

        if exercise.time_remaining <= 0.0 {
            exercise.state = ExerciseState::TimedOut;
            session.timeout_count += 1;
            session.current_index += 1;
            session.coins -= 3;
            session.combo = 0;

            // Falling embers at each panel — exercise timed out
            for (_entity, panel_tf, panel) in &panels {
                if panel.exercise_id == exercise_eid.0 {
                    styled_events.write(StyledParticleEvent {
                        position: panel_tf.translation,
                        color: Color::srgb(1.0, 0.4, 0.1),
                        count: 15,
                        speed: 1.5,
                        lifetime: 2.0,
                        size: 0.1,
                        style: ParticleStyle::FallingEmber,
                    });
                }
            }

            // Despawn panels, poles, and text matching this exercise
            for (entity, _, panel) in &panels {
                if panel.exercise_id == exercise_eid.0 {
                    commands.entity(entity).despawn();
                }
            }
            for (entity, pole_eid) in &poles {
                if pole_eid.0 == exercise_eid.0 {
                    commands.entity(entity).despawn();
                }
            }
            for (entity, qt) in &question_texts {
                if qt.exercise_id == exercise_eid.0 {
                    commands.entity(entity).despawn();
                }
            }
            for (entity, tt) in &timer_texts {
                if tt.exercise_id == exercise_eid.0 {
                    commands.entity(entity).despawn();
                }
            }

            commands.entity(exercise_entity).despawn();
        }
    }
}

fn tick_round_timer(
    time: Res<Time>,
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<super::GameState>>,
) {
    session.round_time_remaining -= time.delta_secs();
    if session.round_time_remaining <= 0.0 {
        session.round_time_remaining = 0.0;
        next_state.set(super::GameState::GameOver);
    }
}
