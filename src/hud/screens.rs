use bevy::prelude::*;

use crate::game::difficulty::Difficulty;
use crate::game::{GameSession, GameState};

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), spawn_menu_screen)
            .add_systems(OnExit(GameState::Menu), despawn_menu)
            .add_systems(
                Update,
                handle_menu_input.run_if(in_state(GameState::Menu)),
            )
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
            .add_systems(OnExit(GameState::GameOver), despawn_game_over)
            .add_systems(
                Update,
                handle_game_over_input.run_if(in_state(GameState::GameOver)),
            );
    }
}

#[derive(Component)]
struct MenuScreen;

#[derive(Component)]
struct DifficultyButton(Difficulty);

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
struct RestartButton;

fn spawn_menu_screen(mut commands: Commands) {
    commands
        .spawn((
            MenuScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("Math Tables Game"),
                TextFont {
                    font_size: 52.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 1.0, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Subtitle
            parent.spawn((
                Text::new("Choose Difficulty"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Difficulty buttons
            for (diff, color) in [
                (Difficulty::Easy, Color::srgb(0.2, 0.8, 0.3)),
                (Difficulty::Medium, Color::srgb(0.9, 0.7, 0.1)),
                (Difficulty::Hard, Color::srgb(0.9, 0.2, 0.2)),
            ] {
                parent
                    .spawn((
                        DifficultyButton(diff),
                        Button,
                        Node {
                            width: Val::Px(320.0),
                            height: Val::Auto,
                            padding: UiRect::axes(Val::Px(20.0), Val::Px(14.0)),
                            margin: UiRect::all(Val::Px(8.0)),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                        BorderColor::all(color),
                    ))
                    .with_children(|btn| {
                        // Difficulty name
                        btn.spawn((
                            Text::new(diff.label()),
                            TextFont {
                                font_size: 26.0,
                                ..default()
                            },
                            TextColor(color),
                        ));
                        // Description
                        btn.spawn((
                            Text::new(diff.description()),
                            TextFont {
                                font_size: 16.0,
                                ..default()
                            },
                            TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                            Node {
                                margin: UiRect::top(Val::Px(4.0)),
                                ..default()
                            },
                        ));
                    });
            }
        });
}

fn despawn_menu(mut commands: Commands, query: Query<Entity, With<MenuScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_menu_input(
    interaction: Query<(&Interaction, &DifficultyButton), Changed<Interaction>>,
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button) in &interaction {
        if *interaction == Interaction::Pressed {
            session.difficulty = button.0;
            next_state.set(GameState::Playing);
        }
    }
}

fn spawn_game_over_screen(mut commands: Commands, session: Res<GameSession>) {
    let total = session.correct_count + session.wrong_count + session.timeout_count;
    let accuracy = if total > 0 {
        (session.correct_count as f32 / total as f32 * 100.0) as u32
    } else {
        0
    };

    let is_round_complete = session.current_index >= session.total_exercises && session.coins > 0;
    let title = if is_round_complete {
        "Round Complete!"
    } else {
        "Game Over"
    };

    commands
        .spawn((
            GameOverScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 48.0,
                    ..default()
                },
                TextColor(if is_round_complete {
                    Color::srgb(0.3, 1.0, 0.5)
                } else {
                    Color::srgb(1.0, 0.3, 0.3)
                }),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Stats
            let stats = format!(
                "Questions: {}\nCorrect: {}\nWrong: {}\nTimeouts: {}\nAccuracy: {}%\nMax Combo: {}\nFinal Coins: {}",
                total,
                session.correct_count,
                session.wrong_count,
                session.timeout_count,
                accuracy,
                session.max_combo,
                session.coins,
            );
            parent.spawn((
                Text::new(stats),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            // Restart button
            parent
                .spawn((
                    RestartButton,
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                    BorderColor::all(Color::srgb(0.3, 1.0, 0.5)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Play Again"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 1.0, 0.5)),
                    ));
                });
        });
}

fn despawn_game_over(mut commands: Commands, query: Query<Entity, With<GameOverScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_game_over_input(
    interaction: Query<&Interaction, (Changed<Interaction>, With<RestartButton>)>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for interaction in &interaction {
        if *interaction == Interaction::Pressed {
            next_state.set(GameState::Menu);
        }
    }
}
