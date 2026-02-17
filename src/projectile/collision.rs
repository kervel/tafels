use bevy::prelude::*;

use super::Projectile;
use crate::game::beacon::{BeaconFacing, BeaconState, BeaconVisual};
use crate::game::exercise::{ActiveExercise, ExerciseId, ExerciseState};
use crate::game::panels::AnswerPanel;
use crate::game::scoring::PendingAnswer;
use crate::game::{ActiveExercises, GameSession};

/// Check if the ball intersects with any answer panel.
pub fn check_ball_panel_collision(
    mut commands: Commands,
    balls: Query<(Entity, &Transform), With<Projectile>>,
    panels: Query<(&Transform, &AnswerPanel)>,
    pending: Option<Res<PendingAnswer>>,
) {
    // Don't process if we already have a pending answer
    if pending.is_some() {
        return;
    }

    for (ball_entity, ball_tf) in &balls {
        let ball_pos = ball_tf.translation;
        let ball_radius = 0.25;

        for (panel_tf, panel) in &panels {
            // Transform ball position into panel's local space
            let local_pos = panel_tf
                .compute_affine()
                .inverse()
                .transform_point3(ball_pos);

            // Forgiving on height (Y), generous on X and Z.
            let half_x = 1.5;  // wider than panel for easier hits
            let half_z = 2.0;  // generous depth

            let dx = (local_pos.x.abs() - half_x).max(0.0);
            let dz = (local_pos.z.abs() - half_z).max(0.0);
            let dist_xz = (dx * dx + dz * dz).sqrt();

            if dist_xz <= ball_radius {
                // Hit! Register the answer
                commands.insert_resource(PendingAnswer {
                    value: panel.value,
                    is_correct: panel.is_correct,
                    hit_position: panel_tf.translation,
                    exercise_id: panel.exercise_id,
                });

                // Despawn the ball
                commands.entity(ball_entity).despawn();
                return;
            }
        }
    }
}

/// Check if a ball hits a dormant beacon to activate it from a distance.
#[allow(clippy::too_many_arguments)]
pub fn check_ball_beacon_collision(
    mut commands: Commands,
    balls: Query<(Entity, &Transform), With<Projectile>>,
    mut beacons: Query<(
        Entity,
        &ExerciseId,
        &mut BeaconState,
        &BeaconFacing,
        &Transform,
        &mut ActiveExercise,
    )>,
    beacon_visuals: Query<(Entity, &ChildOf), With<BeaconVisual>>,
    mut active_exercises: ResMut<ActiveExercises>,
    session: Res<GameSession>,
    terrain: Option<Res<crate::terrain::TerrainResource>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut styled_events: MessageWriter<crate::effects::particles::StyledParticleEvent>,
) {
    if active_exercises.total_engaged >= session.total_exercises {
        return;
    }
    let Some(ref terrain) = terrain else {
        return;
    };

    for (ball_entity, ball_tf) in &balls {
        let ball_pos = ball_tf.translation;

        for (entity, exercise_id, mut state, facing, beacon_tf, mut exercise) in &mut beacons {
            if *state != BeaconState::Dormant {
                continue;
            }

            // Sphere collision: beacon capsule is ~3.5m tall, 0.3m radius
            let beacon_center = beacon_tf.translation + Vec3::new(0.0, 1.75, 0.0);
            let diff = ball_pos - beacon_center;
            let dist = diff.length();

            if dist > 2.5 {
                continue;
            }

            // Hit! Activate the beacon
            *state = BeaconState::Activated;
            exercise.state = ExerciseState::Active;
            active_exercises.total_engaged += 1;

            // Despawn beacon visual children
            for (vis_entity, child_of) in &beacon_visuals {
                if child_of.parent() == entity {
                    commands.entity(vis_entity).despawn();
                }
            }

            // Spawn answer panels
            let facing_toward = beacon_tf.translation + facing.0 * 10.0;
            crate::game::panels::spawn_answer_panels(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut images,
                &terrain.heightmap,
                beacon_tf.translation,
                facing_toward,
                &exercise.choices,
                exercise.correct_answer,
                exercise_id.0,
            );

            // Text rotation
            let to_facing = facing.0.normalize_or_zero();
            let text_rotation = Transform::IDENTITY
                .looking_to(-to_facing, Vec3::Y)
                .rotation;

            // Spawn question text
            crate::game::panels::spawn_question_text(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut images,
                &exercise.question_text(),
                beacon_tf.translation,
                text_rotation,
                exercise_id.0,
            );

            // Spawn timer text
            crate::game::panels::spawn_timer_text(
                &mut commands,
                &mut meshes,
                &mut materials,
                &mut images,
                exercise.time_limit,
                beacon_tf.translation,
                text_rotation,
                exercise_id.0,
            );

            // Expanding ring — beacon activated by ball
            let neon = crate::game::panels::NEON_COLORS[exercise_id.0 as usize % crate::game::panels::NEON_COLORS.len()];
            styled_events.write(crate::effects::particles::StyledParticleEvent {
                position: beacon_tf.translation + Vec3::Y * 0.5,
                color: Color::srgb(neon[0], neon[1], neon[2]),
                count: 24,
                speed: 5.0,
                lifetime: 1.0,
                size: 0.15,
                style: crate::effects::particles::ParticleStyle::Ring,
            });

            // Despawn the ball
            commands.entity(ball_entity).despawn();
            return;
        }
    }
}
