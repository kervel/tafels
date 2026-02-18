use bevy::prelude::*;

use crate::game::difficulty::Difficulty;
use crate::game::{ForceSinglePlayer, GameSession, GameState, Leaderboard, MultiplayerRoundState};
use crate::network::{RoundMessageBuffer, WsConnection};
use crate::touch::TouchDevice;
use crate::touch::keyboard::SoftKeyboardInput;
use tafels_shared::protocol::{ClientMessage, encode};

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Menu), spawn_menu_screen)
            .add_systems(OnExit(GameState::Menu), despawn_menu)
            .add_systems(
                Update,
                (handle_menu_input, handle_singleplayer_toggle)
                    .run_if(in_state(GameState::Menu)),
            )
            .add_systems(
                Update,
                (handle_name_input, handle_name_touch, handle_soft_keyboard_input)
                    .run_if(in_state(GameState::Menu)),
            )
            .add_systems(OnEnter(GameState::GameOver), spawn_game_over_screen)
            .add_systems(OnExit(GameState::GameOver), despawn_game_over)
            .add_systems(
                Update,
                handle_game_over_input.run_if(in_state(GameState::GameOver)),
            )
            .add_systems(
                Update,
                (
                    manage_lobby_screen,
                    handle_lobby_input,
                    manage_countdown_overlay,
                    manage_round_over_screen,
                )
                    .run_if(in_state(GameState::Playing)),
            )
            .add_systems(
                Update,
                tick_countdown.run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Component)]
struct MenuScreen;

#[derive(Component)]
struct DifficultyButton(Difficulty);

#[derive(Component)]
struct NameInputDisplay;

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
struct RestartButton;

#[derive(Component)]
struct LobbyScreen;

#[derive(Component)]
struct LobbyPlayerList;

#[derive(Component)]
struct ReadyButton;

#[derive(Component)]
struct CountdownOverlay;

#[derive(Component)]
struct CountdownText;

#[derive(Component)]
struct RoundOverScreen;

#[derive(Component)]
struct SinglePlayerButton;

#[derive(Component)]
struct SinglePlayerButtonText;

fn spawn_menu_screen(mut commands: Commands, session: Res<GameSession>, force_sp: Res<ForceSinglePlayer>) {
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

            // Name input label
            parent.spawn((
                Text::new("Enter your name:"),
                TextFont {
                    font_size: 22.0,
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.8, 0.8, 0.9)),
                Node {
                    margin: UiRect::bottom(Val::Px(8.0)),
                    ..default()
                },
            ));

            // Name input display (also a button for touch keyboard activation)
            let display_name = if session.player_name.is_empty() {
                "_".to_string()
            } else {
                format!("{}_", session.player_name)
            };
            parent.spawn((
                NameInputDisplay,
                Button,
                Text::new(display_name),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                Node {
                    margin: UiRect::bottom(Val::Px(24.0)),
                    padding: UiRect::axes(Val::Px(16.0), Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.15, 0.8)),
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

            // Single-player toggle
            let sp_label = if force_sp.0 { "Single Player" } else { "Multiplayer" };
            let sp_color = if force_sp.0 {
                Color::srgba(0.7, 0.7, 0.7, 0.6)
            } else {
                Color::srgba(0.4, 0.7, 1.0, 0.8)
            };
            parent
                .spawn((
                    SinglePlayerButton,
                    Button,
                    Node {
                        padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                        margin: UiRect::top(Val::Px(24.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        ..default()
                    },
                    BackgroundColor(Color::NONE),
                    BorderColor::all(sp_color),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        SinglePlayerButtonText,
                        Text::new(sp_label),
                        TextFont {
                            font_size: 16.0,
                            ..default()
                        },
                        TextColor(sp_color),
                    ));
                });
        });
}

fn despawn_menu(mut commands: Commands, query: Query<Entity, With<MenuScreen>>) {
    for entity in &query {
        commands.entity(entity).despawn();
    }
}

fn handle_name_input(
    mut session: ResMut<GameSession>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut name_display: Query<&mut Text, With<NameInputDisplay>>,
) {
    let mut changed = false;

    let shift = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Map key codes to characters
    let key_map: &[(KeyCode, char, char)] = &[
        (KeyCode::KeyA, 'a', 'A'), (KeyCode::KeyB, 'b', 'B'), (KeyCode::KeyC, 'c', 'C'),
        (KeyCode::KeyD, 'd', 'D'), (KeyCode::KeyE, 'e', 'E'), (KeyCode::KeyF, 'f', 'F'),
        (KeyCode::KeyG, 'g', 'G'), (KeyCode::KeyH, 'h', 'H'), (KeyCode::KeyI, 'i', 'I'),
        (KeyCode::KeyJ, 'j', 'J'), (KeyCode::KeyK, 'k', 'K'), (KeyCode::KeyL, 'l', 'L'),
        (KeyCode::KeyM, 'm', 'M'), (KeyCode::KeyN, 'n', 'N'), (KeyCode::KeyO, 'o', 'O'),
        (KeyCode::KeyP, 'p', 'P'), (KeyCode::KeyQ, 'q', 'Q'), (KeyCode::KeyR, 'r', 'R'),
        (KeyCode::KeyS, 's', 'S'), (KeyCode::KeyT, 't', 'T'), (KeyCode::KeyU, 'u', 'U'),
        (KeyCode::KeyV, 'v', 'V'), (KeyCode::KeyW, 'w', 'W'), (KeyCode::KeyX, 'x', 'X'),
        (KeyCode::KeyY, 'y', 'Y'), (KeyCode::KeyZ, 'z', 'Z'),
        (KeyCode::Digit0, '0', '0'), (KeyCode::Digit1, '1', '1'),
        (KeyCode::Digit2, '2', '2'), (KeyCode::Digit3, '3', '3'),
        (KeyCode::Digit4, '4', '4'), (KeyCode::Digit5, '5', '5'),
        (KeyCode::Digit6, '6', '6'), (KeyCode::Digit7, '7', '7'),
        (KeyCode::Digit8, '8', '8'), (KeyCode::Digit9, '9', '9'),
        (KeyCode::Space, ' ', ' '), (KeyCode::Minus, '-', '-'),
    ];

    for &(key, lower, upper) in key_map {
        if keyboard.just_pressed(key) && session.player_name.len() < 20 {
            session.player_name.push(if shift { upper } else { lower });
            changed = true;
        }
    }

    if keyboard.just_pressed(KeyCode::Backspace) {
        session.player_name.pop();
        changed = true;
    }

    if changed {
        for mut text in &mut name_display {
            if session.player_name.is_empty() {
                **text = "_".to_string();
            } else {
                **text = format!("{}_", session.player_name);
            }
        }
    }
}

/// Show soft keyboard when name input is tapped on touch devices.
fn handle_name_touch(
    interaction: Query<&Interaction, (Changed<Interaction>, With<NameInputDisplay>)>,
    touch_device: Res<TouchDevice>,
) {
    if !touch_device.detected {
        return;
    }
    for inter in &interaction {
        if *inter == Interaction::Pressed {
            crate::touch::keyboard::show_soft_keyboard();
        }
    }
}

/// Read characters from the soft keyboard buffer into the player name.
fn handle_soft_keyboard_input(
    mut session: ResMut<GameSession>,
    mut soft_kb: ResMut<SoftKeyboardInput>,
    mut name_display: Query<&mut Text, With<NameInputDisplay>>,
) {
    if soft_kb.buffer.is_empty() {
        return;
    }

    let mut changed = false;
    for c in soft_kb.buffer.drain(..) {
        if c == '\x08' {
            // Backspace
            session.player_name.pop();
            changed = true;
        } else if c.is_alphanumeric() || c == ' ' || c == '-' {
            if session.player_name.len() < 20 {
                session.player_name.push(c);
                changed = true;
            }
        }
    }

    if changed {
        for mut text in &mut name_display {
            if session.player_name.is_empty() {
                **text = "_".to_string();
            } else {
                **text = format!("{}_", session.player_name);
            }
        }
    }
}

fn handle_menu_input(
    interaction: Query<(&Interaction, &DifficultyButton), Changed<Interaction>>,
    mut session: ResMut<GameSession>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, button) in &interaction {
        if *interaction == Interaction::Pressed {
            crate::touch::keyboard::hide_soft_keyboard();
            session.difficulty = button.0;
            next_state.set(GameState::Playing);
        }
    }
}

fn handle_singleplayer_toggle(
    interaction: Query<&Interaction, (Changed<Interaction>, With<SinglePlayerButton>)>,
    mut force_sp: ResMut<ForceSinglePlayer>,
    mut text_query: Query<(&mut Text, &mut TextColor), With<SinglePlayerButtonText>>,
    mut border_query: Query<&mut BorderColor, With<SinglePlayerButton>>,
) {
    for inter in &interaction {
        if *inter == Interaction::Pressed {
            force_sp.0 = !force_sp.0;
            let (label, color) = if force_sp.0 {
                ("Single Player", Color::srgba(0.7, 0.7, 0.7, 0.6))
            } else {
                ("Multiplayer", Color::srgba(0.4, 0.7, 1.0, 0.8))
            };
            for (mut text, mut tc) in &mut text_query {
                **text = label.to_string();
                tc.0 = color;
            }
            for mut bc in &mut border_query {
                *bc = BorderColor::all(color);
            }
        }
    }
}

// --- Lobby Screen ---

fn manage_lobby_screen(
    mut commands: Commands,
    round_state: Res<MultiplayerRoundState>,
    existing: Query<Entity, With<LobbyScreen>>,
    round_buf: Res<RoundMessageBuffer>,
    player_list: Query<Entity, With<LobbyPlayerList>>,
) {
    let should_show = matches!(*round_state, MultiplayerRoundState::Lobby);

    if should_show {
        // Spawn lobby screen if it doesn't exist
        if existing.is_empty() {
            spawn_lobby_screen(&mut commands);
        }

        // Update player list when lobby state changes (always refresh from buffer)
        if !round_buf.lobby_states.is_empty() {
            // Remove old entries
            for entity in &player_list {
                commands.entity(entity).despawn();
            }

            // Find the lobby screen to add children
            for lobby_entity in &existing {
                commands.entity(lobby_entity).with_children(|parent| {
                    for p in &round_buf.lobby_states {
                        let status = if p.ready { "Ready" } else { "---" };
                        let color = if p.ready {
                            Color::srgb(0.3, 1.0, 0.5)
                        } else {
                            Color::srgba(0.7, 0.7, 0.7, 0.9)
                        };
                        parent.spawn((
                            LobbyPlayerList,
                            Text::new(format!("{} {}", p.name, status)),
                            TextFont {
                                font_size: 22.0,
                                ..default()
                            },
                            TextColor(color),
                            Node {
                                margin: UiRect::bottom(Val::Px(4.0)),
                                ..default()
                            },
                        ));
                    }
                });
            }
        }
    } else {
        // Despawn lobby screen
        for entity in &existing {
            commands.entity(entity).despawn();
        }
    }
}

fn spawn_lobby_screen(commands: &mut Commands) {
    commands
        .spawn((
            LobbyScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
            GlobalZIndex(10),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new("Lobby - Waiting for players"),
                TextFont {
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(0.3, 1.0, 0.5)),
                Node {
                    margin: UiRect::bottom(Val::Px(30.0)),
                    ..default()
                },
            ));

            // Ready button
            parent
                .spawn((
                    ReadyButton,
                    Button,
                    Node {
                        width: Val::Px(200.0),
                        height: Val::Px(56.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border: UiRect::all(Val::Px(2.0)),
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                    BorderColor::all(Color::srgb(0.3, 1.0, 0.5)),
                ))
                .with_children(|btn| {
                    btn.spawn((
                        Text::new("Ready!"),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.3, 1.0, 0.5)),
                    ));
                });
        });
}

fn handle_lobby_input(
    interaction: Query<&Interaction, (Changed<Interaction>, With<ReadyButton>)>,
    ws: Option<Res<WsConnection>>,
) {
    for inter in &interaction {
        if *inter == Interaction::Pressed {
            if let Some(ws) = &ws {
                ws.send_binary(encode(&ClientMessage::Ready));
            }
        }
    }
}

// --- Countdown Overlay ---

fn manage_countdown_overlay(
    mut commands: Commands,
    round_state: Res<MultiplayerRoundState>,
    existing: Query<Entity, With<CountdownOverlay>>,
    mut text_query: Query<&mut Text, With<CountdownText>>,
) {
    match *round_state {
        MultiplayerRoundState::Countdown(remaining) => {
            let display = remaining.ceil() as u8;
            if existing.is_empty() {
                commands
                    .spawn((
                        CountdownOverlay,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            position_type: PositionType::Absolute,
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
                        GlobalZIndex(20),
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            CountdownText,
                            Text::new(format!("{}", display)),
                            TextFont {
                                font_size: 120.0,
                                ..default()
                            },
                            TextColor(Color::srgb(1.0, 1.0, 1.0)),
                        ));
                    });
            } else {
                for mut text in &mut text_query {
                    **text = format!("{}", display);
                }
            }
        }
        _ => {
            for entity in &existing {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn tick_countdown(
    mut round_state: ResMut<MultiplayerRoundState>,
    time: Res<Time>,
) {
    if let MultiplayerRoundState::Countdown(ref mut remaining) = *round_state {
        *remaining -= time.delta_secs();
        if *remaining < 0.0 {
            *remaining = 0.0;
        }
    }
}

// --- Round Over Screen ---

fn manage_round_over_screen(
    mut commands: Commands,
    round_state: Res<MultiplayerRoundState>,
    existing: Query<Entity, With<RoundOverScreen>>,
    leaderboard: Res<Leaderboard>,
) {
    let should_show = matches!(*round_state, MultiplayerRoundState::RoundOver);

    if should_show && existing.is_empty() {
        commands
            .spawn((
                RoundOverScreen,
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    position_type: PositionType::Absolute,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.8)),
                GlobalZIndex(15),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new("Round Over!"),
                    TextFont {
                        font_size: 48.0,
                        ..default()
                    },
                    TextColor(Color::srgb(1.0, 0.85, 0.0)),
                    Node {
                        margin: UiRect::bottom(Val::Px(30.0)),
                        ..default()
                    },
                ));

                parent.spawn((
                    Text::new("Final Standings"),
                    TextFont {
                        font_size: 28.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        margin: UiRect::bottom(Val::Px(20.0)),
                        ..default()
                    },
                ));

                let mut sorted = leaderboard.entries.clone();
                sorted.sort_by(|a, b| b.coins.cmp(&a.coins));

                for (i, entry) in sorted.iter().enumerate() {
                    let medal = match i {
                        0 => "1st",
                        1 => "2nd",
                        2 => "3rd",
                        _ => "",
                    };
                    let color = match i {
                        0 => Color::srgb(1.0, 0.85, 0.0),
                        1 => Color::srgb(0.75, 0.75, 0.75),
                        2 => Color::srgb(0.8, 0.5, 0.2),
                        _ => Color::WHITE,
                    };
                    let label = if medal.is_empty() {
                        format!("{}. {} - {} coins", i + 1, entry.name, entry.coins)
                    } else {
                        format!("{} {} - {} coins", medal, entry.name, entry.coins)
                    };
                    parent.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: 24.0,
                            ..default()
                        },
                        TextColor(color),
                        Node {
                            margin: UiRect::bottom(Val::Px(6.0)),
                            ..default()
                        },
                    ));
                }

                parent.spawn((
                    Text::new("Next round starting soon..."),
                    TextFont {
                        font_size: 18.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.7, 0.7, 0.7, 0.8)),
                    Node {
                        margin: UiRect::top(Val::Px(20.0)),
                        ..default()
                    },
                ));
            });
    } else if !should_show {
        for entity in &existing {
            commands.entity(entity).despawn();
        }
    }
}

// --- Game Over Screen (single-player) ---

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
                        height: Val::Px(56.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(16.0)),
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
