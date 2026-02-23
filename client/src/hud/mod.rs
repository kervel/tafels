pub mod notification;
pub mod screens;

use bevy::prelude::*;

use crate::game::ActiveExercises;
use crate::game::GameSession;
use crate::game::GameState;
use crate::game::Leaderboard;
use crate::game::MultiplayerRoundState;
use crate::network::ConnectionStatus;
use crate::quality::{QualityLevel, QualitySettings};

#[derive(Resource, Clone)]
pub struct GameFont(pub Handle<Font>);

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        // Load font eagerly so the resource is available before any state systems run
        let font: Handle<Font> = app.world_mut().resource_mut::<AssetServer>().load("fonts/BubblegumSans-Regular.ttf");
        app.insert_resource(GameFont(font));

        app.add_systems(OnEnter(GameState::Playing), spawn_hud)
            .add_systems(OnExit(GameState::Playing), despawn_hud)
            .add_systems(
                Update,
                (
                    update_coin_display,
                    update_progress_display,
                    update_combo_display,
                    update_round_timer,
                    update_connection_indicator,
                    consume_answer_feedback,
                    show_quality_notification,
                    tick_feedback,
                    update_leaderboard,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_plugins(screens::ScreensPlugin)
            .add_plugins(notification::NotificationPlugin);
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
struct ConnectionIndicator;

#[derive(Component)]
struct FeedbackPopup {
    timer: f32,
}

#[derive(Component)]
struct LeaderboardPanel;

#[derive(Component)]
struct LeaderboardEntryText;

fn spawn_hud(mut commands: Commands, game_font: Res<GameFont>) {
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
                            font: game_font.0.clone(),
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
                            font: game_font.0.clone(),
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
                                    font: game_font.0.clone(),
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
                                    font: game_font.0.clone(),
                                    font_size: 22.0,
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                });

            // Connection status indicator (bottom-left)
            parent.spawn((
                ConnectionIndicator,
                Text::new("Offline"),
                TextFont {
                    font: game_font.0.clone(),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.3, 0.3)),
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(10.0),
                    left: Val::Px(10.0),
                    ..default()
                },
            ));

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

            // Leaderboard panel (top-right, only visible in multiplayer)
            parent
                .spawn((
                    LeaderboardPanel,
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(60.0),
                        right: Val::Px(10.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(8.0)),
                        min_width: Val::Px(180.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                    Visibility::Hidden,
                ))
                .with_children(|lb| {
                    lb.spawn((
                        Text::new("Leaderboard"),
                        TextFont {
                            font: game_font.0.clone(),
                            font_size: 18.0,
                            ..default()
                        },
                        TextColor(Color::srgb(1.0, 0.85, 0.0)),
                        Node {
                            margin: UiRect::bottom(Val::Px(4.0)),
                            ..default()
                        },
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
                _ => 2.0,
            };
            **text = format!("x{} Combo!", multiplier);

            color.0 = match session.combo {
                2 => Color::srgb(1.0, 0.6, 0.0),   // orange
                _ => Color::srgb(1.0, 0.2, 0.1),   // bright red-orange
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

fn update_connection_indicator(
    status: Res<ConnectionStatus>,
    mut query: Query<(&mut Text, &mut TextColor), With<ConnectionIndicator>>,
) {
    for (mut text, mut color) in &mut query {
        let (label, c) = if status.is_solo() {
            ("Solo (Online)", Color::srgb(0.4, 0.7, 1.0))
        } else if status.is_online() {
            ("Online", Color::srgb(0.3, 0.9, 0.3))
        } else {
            ("Offline", Color::srgb(0.9, 0.3, 0.3))
        };
        **text = label.to_string();
        color.0 = c;
    }
}

/// How quickly the player answered.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeedTier {
    /// Answered with > 75% time remaining.
    Lightning,
    /// Answered with > 50% time remaining.
    Fast,
    /// Answered with ≤ 50% time remaining.
    Normal,
}

/// Resource to trigger feedback popup from scoring system.
#[derive(Resource)]
pub struct AnswerFeedback {
    pub correct: bool,
    pub coins_delta: i32,
    pub combo: u32,
    pub speed_tier: SpeedTier,
    pub hit_position: Vec3,
}

/// Remove the AnswerFeedback resource each frame (3D bonus text handles the visuals now).
fn consume_answer_feedback(mut commands: Commands, feedback: Option<Res<AnswerFeedback>>) {
    if feedback.is_some() {
        commands.remove_resource::<AnswerFeedback>();
    }
}

/// Show a notification when adaptive quality changes.
fn show_quality_notification(
    mut commands: Commands,
    quality: Res<QualitySettings>,
    existing: Query<Entity, With<FeedbackPopup>>,
    game_font: Res<GameFont>,
    mut last_level: Local<Option<QualityLevel>>,
) {
    if *last_level == Some(quality.level) {
        return;
    }
    *last_level = Some(quality.level);

    let msg = match quality.level {
        QualityLevel::Low => "Slow GPU, reducing effects",
        QualityLevel::High => return,
    };

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    commands.spawn((
        FeedbackPopup { timer: 3.0 },
        Text::new(msg),
        TextFont {
            font: game_font.0.clone(),
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.7, 0.3)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(40.0),
            margin: UiRect {
                left: Val::Px(-180.0),
                ..default()
            },
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
    ));
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

fn update_leaderboard(
    mut commands: Commands,
    leaderboard: Res<Leaderboard>,
    round_state: Res<MultiplayerRoundState>,
    status: Res<ConnectionStatus>,
    mut panel_query: Query<(Entity, &mut Visibility), With<LeaderboardPanel>>,
    entry_query: Query<Entity, With<LeaderboardEntryText>>,
    game_font: Res<GameFont>,
) {
    let is_multiplayer = status.is_online()
        && matches!(
            *round_state,
            MultiplayerRoundState::Playing | MultiplayerRoundState::RoundOver
        );

    for (panel_entity, mut vis) in &mut panel_query {
        if !is_multiplayer || leaderboard.entries.is_empty() {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;

        // Remove old entries
        for entity in &entry_query {
            commands.entity(entity).despawn();
        }

        // Sort entries by coins desc
        let mut sorted = leaderboard.entries.clone();
        sorted.sort_by(|a, b| b.coins.cmp(&a.coins));

        let my_player_id = match &*status {
            ConnectionStatus::Connected { my_player_id, .. } => Some(*my_player_id),
            _ => None,
        };

        for (i, entry) in sorted.iter().enumerate() {
            let is_me = my_player_id == Some(entry.player_id);
            let color = if is_me {
                Color::srgb(0.3, 1.0, 0.5)
            } else {
                Color::WHITE
            };
            let label = format!("{}. {} - {}", i + 1, entry.name, entry.coins);
            commands.entity(panel_entity).with_children(|lb| {
                lb.spawn((
                    LeaderboardEntryText,
                    Text::new(label),
                    TextFont {
                        font: game_font.0.clone(),
                        font_size: 15.0,
                        ..default()
                    },
                    TextColor(color),
                    Node {
                        margin: UiRect::bottom(Val::Px(2.0)),
                        ..default()
                    },
                ));
            });
        }
    }
}
