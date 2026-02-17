use bevy::prelude::*;

use crate::character::CharacterMarker;

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, check_vegetation_collision);
    }
}

/// Attach to trees and rocks to block character movement.
#[derive(Component)]
pub struct VegetationCollider {
    pub radius: f32,
}

const CHARACTER_RADIUS: f32 = 0.3;

fn check_vegetation_collision(
    colliders: Query<(&Transform, &VegetationCollider), Without<CharacterMarker>>,
    mut character: Query<&mut Transform, With<CharacterMarker>>,
) {
    let Ok(mut char_tf) = character.single_mut() else {
        return;
    };

    let char_xz = Vec2::new(char_tf.translation.x, char_tf.translation.z);

    for (collider_tf, collider) in &colliders {
        let col_xz = Vec2::new(collider_tf.translation.x, collider_tf.translation.z);
        let diff = char_xz - col_xz;
        let dist = diff.length();
        let min_dist = collider.radius + CHARACTER_RADIUS;

        if dist < min_dist && dist > 0.001 {
            // Push character out of the collider
            let push = diff.normalize() * (min_dist - dist);
            char_tf.translation.x += push.x;
            char_tf.translation.z += push.y;
        }
    }
}
