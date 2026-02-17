use bevy::prelude::*;

use super::Projectile;
use crate::game::panels::AnswerPanel;
use crate::game::scoring::PendingAnswer;

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
