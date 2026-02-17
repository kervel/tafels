pub mod screens;

use bevy::prelude::*;

use crate::game::ActiveExercises;
use crate::game::GameSession;
use crate::game::GameState;

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(OnExit(GameState::Playing), despawn_hud)
            .add_systems(
                Update,
                (
                    update_coin_display,
                    update_progress_display,
                    update_combo_display,
                    update_round_timer,
                    show_answer_feedback,
                    tick_feedback,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_plugins(screens::ScreensPlugin);
    }
}

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct CoinDisplay;

#[derive(Component)]
struct ProgressDisplay;

#[derive(Component)]
struct ComboDisplay;

#[derive(Component)]
struct RoundTimerBar;

#[derive(Component)]
struct RoundTimerText;

#[derive(Component)]
struct FeedbackPopup {
    timer: f32,
}

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            HudRoot,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
        ))
        .with_children(|parent| {
            // Top bar: coins (left), combo + progress (right)
            parent
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(50.0),
                        flex_direction: FlexDirection::Row,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                ))
                .with_children(|top_bar| {
                    // Coin counter (left)
                    top_bar.spawn((
                        CoinDisplay,
                        Text::new("Coins: 10"),
                        TextFont {
                            font_size: 28.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.0)),
                    ));

                    // Round timer text
                    top_bar.spawn((
                        RoundTimerText,
                        Text::new("3:00"),
                        TextFont {
                            font_size: 26.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.2, 0.9, 0.2)),
                    ));

                    // Right section: combo + progress
                    top_bar
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(20.0),
                            ..default()
                        })
                        .with_children(|right| {
                            // Combo display (hidden when combo < 2)
                            right.spawn((
                                ComboDisplay,
                                Text::new(""),
                                TextFont {
                                    font_size: 26.0,
                                    ..default()
                                },
                                TextColor(Color::srgb(1.0, 0.6, 0.0)),
                            ));

                            // Progress counter
                            right.spawn((
                                ProgressDisplay,
                                Text::new("0 / 20"),
                                TextFont {
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            // Round timer bar (full width, below top bar)
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    height: Val::Px(6.0),
                    ..default()
                })
                .with_children(|bar_container| {
                    // Background
                    bar_container.spawn((
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.2, 0.2, 0.2, 0.5)),
                    ));
                    // Fill
                    bar_container.spawn((
                        RoundTimerBar,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.2, 0.9, 0.2)),
                    ));
                });
        });
}

fn despawn_hud(mut commands: Commands, hud: Query<Entity, With<HudRoot>>) {
    for entity in &hud {
        commands.entity(entity).despawn();
    }
}

fn update_coin_display(session: Res<GameSession>, mut query: Query<&mut Text, With<CoinDisplay>>) {
    for mut text in &mut query {
        **text = format!("Coins: {}", session.coins);
    }
}

fn update_progress_display(
    active_exercises: Res<ActiveExercises>,
    session: Res<GameSession>,
    mut query: Query<&mut Text, With<ProgressDisplay>>,
) {
    for mut text in &mut query {
        **text = format!(
            "{} / {}",
            active_exercises.total_engaged, session.total_exercises
        );
    }
}

fn update_combo_display(
    session: Res<GameSession>,
    mut query: Query<(&mut Text, &mut TextColor), With<ComboDisplay>>,
) {
    for (mut text, mut color) in &mut query {
        if session.combo >= 2 {
            let multiplier = match session.combo {
                2 => 1.5,
                3 => 2.0,
                _ => 3.0,
            };
            **text = format!("x{} Combo!", multiplier);

            color.0 = match session.combo {
                2 => Color::srgb(1.0, 0.6, 0.0),       // orange
                3 => Color::srgb(1.0, 0.3, 0.0),       // deep orange
                _ => Color::srgb(1.0, 0.15, 0.15),     // red
            };
        } else {
            **text = String::new();
        }
    }
}

fn update_round_timer(
    session: Res<GameSession>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<RoundTimerText>>,
    mut bar_query: Query<(&mut Node, &mut BackgroundColor), With<RoundTimerBar>>,
) {
    let remaining = session.round_time_remaining.max(0.0);
    let fraction = (remaining / session.round_time_limit).clamp(0.0, 1.0);
    let minutes = (remaining / 60.0).floor() as u32;
    let seconds = (remaining % 60.0).floor() as u32;

    for (mut text, mut color) in &mut text_query {
        **text = format!("{}:{:02}", minutes, seconds);
        color.0 = if fraction > 0.5 {
            Color::srgb(0.2, 0.9, 0.2) // green
        } else if fraction > 0.25 {
            Color::srgb(0.9, 0.9, 0.2) // yellow
        } else {
            Color::srgb(0.9, 0.2, 0.2) // red
        };
    }

    for (mut node, mut bg) in &mut bar_query {
        node.width = Val::Percent(fraction * 100.0);
        *bg = if fraction > 0.5 {
            BackgroundColor(Color::srgb(0.2, 0.9, 0.2))
        } else if fraction > 0.25 {
            BackgroundColor(Color::srgb(0.9, 0.9, 0.2))
        } else {
            BackgroundColor(Color::srgb(0.9, 0.2, 0.2))
        };
    }
}

/// Resource to trigger feedback popup from scoring system.
#[derive(Resource)]
pub struct AnswerFeedback {
    pub correct: bool,
    pub coins_delta: i32,
    pub combo: u32,
}

/// Spawn a centered feedback text when an answer is processed.
fn show_answer_feedback(
    mut commands: Commands,
    feedback: Option<Res<AnswerFeedback>>,
    existing: Query<Entity, With<FeedbackPopup>>,
) {
    let Some(fb) = feedback else {
        return;
    };

    // Remove old popup if any
    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let (msg, color) = if fb.correct {
        let combo_info = if fb.combo >= 2 {
            let mult = match fb.combo {
                2 => "1.5x",
                3 => "2x",
                _ => "3x",
            };
            format!(" ({})", mult)
        } else {
            String::new()
        };
        (
            format!("Correct! +{} coins{}", fb.coins_delta, combo_info),
            Color::srgb(0.2, 1.0, 0.3),
        )
    } else {
        (
            format!("Wrong! {} coins", fb.coins_delta),
            Color::srgb(1.0, 0.3, 0.2),
        )
    };

    commands.spawn((
        FeedbackPopup { timer: 1.5 },
        Text::new(msg),
        TextFont {
            font_size: 42.0,
            ..default()
        },
        TextColor(color),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(40.0),
            margin: UiRect {
                left: Val::Px(-150.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    ));

    commands.remove_resource::<AnswerFeedback>();
}

/// Fade out and despawn feedback popup.
fn tick_feedback(
    mut commands: Commands,
    time: Res<Time>,
    mut popups: Query<(Entity, &mut FeedbackPopup, &mut TextColor)>,
) {
    for (entity, mut popup, mut color) in &mut popups {
        popup.timer -= time.delta_secs();
        if popup.timer <= 0.0 {
            commands.entity(entity).despawn();
        } else if popup.timer < 0.5 {
            // Fade out
            let alpha = popup.timer / 0.5;
            color.0 = color.0.with_alpha(alpha);
        }
    }
}
