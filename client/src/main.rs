mod camera;
mod character;
mod collision;
mod effects;
mod game;
mod grass;
mod hud;
mod lighting;
mod network;
mod projectile;
mod quality;
mod terrain;
mod vegetation;

mod touch;

use bevy::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.35, 0.45, 0.58)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "3dt - Math Tables Game".to_string(),
                fit_canvas_to_parent: true,
                prevent_default_event_handling: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(quality::QualityPlugin)
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
        .add_plugins(network::NetworkPlugin)
        .add_plugins(network::remote_players::RemotePlayersPlugin)
        .add_plugins(touch::TouchPlugin)
        .run();
}
