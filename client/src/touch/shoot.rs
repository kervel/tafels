use bevy::prelude::*;

use crate::game::GameState;
use super::{TouchDevice, touch_playing};

pub struct ShootPlugin;

impl Plugin for ShootPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                OnEnter(GameState::Playing),
                spawn_shoot_button.run_if(|td: Res<TouchDevice>| td.detected),
            )
            .add_systems(OnExit(GameState::Playing), despawn_shoot_button)
            .add_systems(
                Update,
                handle_shoot_button.run_if(touch_playing),
            );
    }
}

/// Inserted as a resource to signal a shoot request from touch.
#[derive(Resource)]
pub struct ShootRequest;

#[derive(Component)]
struct ShootButtonMarker;

#[derive(Component)]
struct ShootButtonRoot;

fn spawn_shoot_button(mut commands: Commands) {
    commands
        .spawn((
            ShootButtonRoot,
            ShootButtonMarker,
            Button,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Percent(8.0),
                bottom: Val::Percent(12.0),
                width: Val::Px(140.0),
                height: Val::Px(140.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Percent(50.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.9, 0.15, 0.15, 0.45)),
            BorderColor::all(Color::srgba(1.0, 1.0, 1.0, 0.5)),
            GlobalZIndex(50),
        ))
        .with_children(|btn| {
            // Inner circle to look like a ball
            btn.spawn((
                Node {
                    width: Val::Px(60.0),
                    height: Val::Px(60.0),
                    border_radius: BorderRadius::all(Val::Percent(50.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(1.0, 0.3, 0.3, 0.8)),
            ));
        });
}

fn despawn_shoot_button(mut commands: Commands, query: Query<Entity, With<ShootButtonRoot>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_shoot_button(
    mut commands: Commands,
    interaction: Query<&Interaction, (Changed<Interaction>, With<ShootButtonMarker>)>,
) {
    for inter in &interaction {
        if *inter == Interaction::Pressed {
            commands.insert_resource(ShootRequest);
        }
    }
}
