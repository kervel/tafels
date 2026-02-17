pub mod particles;

use bevy::prelude::*;
use crate::game::GameState;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<particles::ParticleBurstEvent>()
            .add_systems(
                Update,
                (
                    particles::handle_burst_events,
                    particles::update_particles,
                    particles::update_burst_lights,
                    particles::ball_trail_particles,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
