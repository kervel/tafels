use bevy::prelude::*;
use rand::prelude::*;

use super::difficulty::Difficulty;
use super::GameSession;
use crate::character::CharacterMarker;
use crate::effects::particles::ParticleBurstEvent;

pub struct ExercisePlugin;

impl Plugin for ExercisePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ExerciseSpawnState::default())
            .add_systems(
                Update,
                (check_exercise_spawn, tick_exercise_timer)
                    .run_if(in_state(super::GameState::Playing)),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Multiply,
    Divide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExerciseState {
    Active,
    Answered,
    TimedOut,
}

#[derive(Resource)]
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

#[derive(Resource)]
pub struct ExerciseSpawnState {
    pub cooldown: f32,
    pub last_spawn_pos: Vec3,
    pub exercises_started: bool,
}

impl Default for ExerciseSpawnState {
    fn default() -> Self {
        Self {
            cooldown: 3.0,
            last_spawn_pos: Vec3::ZERO,
            exercises_started: false,
        }
    }
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
        state: ExerciseState::Active,
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

fn check_exercise_spawn(
    mut commands: Commands,
    time: Res<Time>,
    session: Res<GameSession>,
    active: Option<Res<ActiveExercise>>,
    mut spawn_state: ResMut<ExerciseSpawnState>,
    character: Query<&Transform, With<CharacterMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    terrain: Option<Res<crate::terrain::TerrainResource>>,
) {
    if active.is_some() {
        return;
    }

    if session.current_index >= session.total_exercises {
        return;
    }

    spawn_state.cooldown -= time.delta_secs();

    let Ok(char_tf) = character.single() else {
        return;
    };

    let walked_far = char_tf
        .translation
        .distance(spawn_state.last_spawn_pos)
        > 6.0;

    if spawn_state.cooldown <= 0.0 || (walked_far && spawn_state.exercises_started) {
        let exercise = generate_exercise(&session.difficulty);

        let char_forward = char_tf.rotation * Vec3::NEG_Z;
        let spawn_center = char_tf.translation + char_forward.normalize_or_zero() * 20.0;

        let Some(ref terrain) = terrain else {
            return;
        };

        super::panels::spawn_answer_panels(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            &terrain.heightmap,
            spawn_center,
            char_tf.translation,
            &exercise.choices,
            exercise.correct_answer,
        );

        commands.insert_resource(exercise);
        spawn_state.cooldown = 5.0;
        spawn_state.last_spawn_pos = char_tf.translation;
        spawn_state.exercises_started = true;
    }
}

fn tick_exercise_timer(
    mut commands: Commands,
    time: Res<Time>,
    active: Option<ResMut<ActiveExercise>>,
    mut session: ResMut<GameSession>,
    panels: Query<(Entity, &Transform), With<super::panels::AnswerPanel>>,
    poles: Query<Entity, With<super::panels::PanelPole>>,
    mut burst_events: MessageWriter<ParticleBurstEvent>,
) {
    let Some(mut exercise) = active else {
        return;
    };

    if exercise.state != ExerciseState::Active {
        return;
    }

    exercise.time_remaining -= time.delta_secs();

    if exercise.time_remaining <= 0.0 {
        exercise.state = ExerciseState::TimedOut;
        session.timeout_count += 1;
        session.current_index += 1;
        session.coins -= 3;
        session.combo = 0;

        // Orange timeout particles at each panel
        for (_entity, panel_tf) in &panels {
            burst_events.write(ParticleBurstEvent {
                position: panel_tf.translation,
                color: Color::srgb(1.0, 0.5, 0.0),
                count: 25,
                speed: 6.0,
                lifetime: 1.0,
                size: 0.15,
            });
        }

        // Despawn panels and poles
        for (entity, _) in &panels {
            commands.entity(entity).despawn();
        }
        for entity in &poles {
            commands.entity(entity).despawn();
        }

        commands.remove_resource::<ActiveExercise>();
    }
}
