pub mod bonus_text;
pub mod particles;

use crate::game::GameState;
use bevy::prelude::*;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<particles::ParticleBurstEvent>()
            .add_message::<particles::StyledParticleEvent>()
            .add_message::<bonus_text::BonusTextEvent>()
            .add_systems(Startup, bonus_text::load_digit_assets)
            .add_systems(
                Update,
                (
                    particles::handle_burst_events,
                    particles::handle_styled_events,
                    particles::update_particles,
                    particles::update_burst_lights,
                    particles::ball_trail_particles,
                    bonus_text::emit_bonus_text_on_feedback,
                    bonus_text::handle_bonus_text_events,
                    bonus_text::update_bonus_text,
                    bonus_text::billboard_bonus_text,
                    bonus_text::color_bonus_glyphs,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}
