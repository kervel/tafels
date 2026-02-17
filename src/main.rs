mod camera;
mod character;
mod collision;
mod effects;
mod game;
mod grass;
mod hud;
mod lighting;
mod projectile;
mod terrain;
mod vegetation;

use bevy::prelude::*;
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.35, 0.45, 0.58)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "3dt - Math Tables Game".to_string(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FpsOverlayPlugin {
            config: FpsOverlayConfig {
                text_config: TextFont {
                    font_size: 20.0,
                    ..default()
                },
                text_color: Color::srgb(1.0, 1.0, 0.0),
                enabled: true,
                ..default()
            },
        })
        .add_plugins(terrain::TerrainPlugin)
        .add_plugins(lighting::LightingPlugin)
        .add_plugins(character::CharacterPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(vegetation::VegetationPlugin)
        .add_plugins(grass::GrassPlugin)
        .add_plugins(collision::CollisionPlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(projectile::ProjectilePlugin)
        .add_plugins(hud::HudPlugin)
        .add_plugins(effects::EffectsPlugin)
        .run();
}
