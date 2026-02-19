use bevy::prelude::*;

use crate::game::{ActiveExercises, GameSession, GameState, MultiplayerRoundState};
use crate::network::{ConnectionStatus, RoundMessageBuffer, WsConnection};
use tafels_shared::protocol::{ClientMessage, encode};

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LobbyNotification>()
            .add_systems(
                Update,
                (
                    update_lobby_notification,
                    render_lobby_notification,
                    handle_notification_buttons,
                )
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

#[derive(Resource, Default)]
pub struct LobbyNotification {
    pub waiting_count: u32,
    pub dismissed: bool,
    pub last_count: u32,
}

#[derive(Component)]
struct NotificationBanner;

#[derive(Component)]
struct NotificationJoinButton;

#[derive(Component)]
struct NotificationDismissButton;

#[derive(Component)]
struct NotificationText;

fn update_lobby_notification(
    connection_status: Res<ConnectionStatus>,
    round_buf: Res<RoundMessageBuffer>,
    round_state: Res<MultiplayerRoundState>,
    mut notification: ResMut<LobbyNotification>,
) {
    // Only show notifications when playing solo or in a multiplayer round
    let should_track = connection_status.is_solo()
        || (connection_status.is_online()
            && matches!(
                *round_state,
                MultiplayerRoundState::Playing | MultiplayerRoundState::RoundOver
            ));

    if !should_track {
        notification.waiting_count = 0;
        notification.dismissed = false;
        notification.last_count = 0;
        return;
    }

    // Count other players waiting in lobby (not solo, not ready, not self)
    let my_id = connection_status.my_player_id();
    let waiting = round_buf
        .lobby_states
        .iter()
        .filter(|p| !p.playing_solo && !p.ready && Some(p.player_id) != my_id)
        .count() as u32;

    // Reset dismissed flag if count changed
    if waiting != notification.last_count {
        notification.dismissed = false;
        notification.last_count = waiting;
    }

    notification.waiting_count = waiting;
}

fn render_lobby_notification(
    mut commands: Commands,
    notification: Res<LobbyNotification>,
    existing: Query<Entity, With<NotificationBanner>>,
    mut text_query: Query<&mut Text, With<NotificationText>>,
) {
    let should_show = notification.waiting_count > 0 && !notification.dismissed;

    if should_show {
        let label = if notification.waiting_count == 1 {
            "1 player waiting in lobby".to_string()
        } else {
            format!("{} players waiting in lobby", notification.waiting_count)
        };

        if existing.is_empty() {
            // Spawn notification banner
            commands
                .spawn((
                    NotificationBanner,
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(60.0),
                        left: Val::Percent(50.0),
                        margin: UiRect {
                            left: Val::Px(-200.0),
                            ..default()
                        },
                        width: Val::Px(400.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(Val::Px(12.0)),
                        row_gap: Val::Px(8.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.05, 0.1, 0.2, 0.9)),
                    GlobalZIndex(25),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        NotificationText,
                        Text::new(label),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::srgb(0.4, 0.7, 1.0)),
                    ));

                    // Button row
                    parent
                        .spawn(Node {
                            flex_direction: FlexDirection::Row,
                            column_gap: Val::Px(12.0),
                            ..default()
                        })
                        .with_children(|row| {
                            // Join button
                            row.spawn((
                                NotificationJoinButton,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                                BorderColor::all(Color::srgb(0.3, 1.0, 0.5)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Join"),
                                    TextFont {
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgb(0.3, 1.0, 0.5)),
                                ));
                            });

                            // Dismiss button
                            row.spawn((
                                NotificationDismissButton,
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(16.0), Val::Px(6.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.8)),
                                BorderColor::all(Color::srgba(0.5, 0.5, 0.5, 0.6)),
                            ))
                            .with_children(|btn| {
                                btn.spawn((
                                    Text::new("Dismiss"),
                                    TextFont {
                                        font_size: 18.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgba(0.7, 0.7, 0.7, 0.8)),
                                ));
                            });
                        });
                });
        } else {
            // Update existing text
            for mut text in &mut text_query {
                **text = label.clone();
            }
        }
    } else {
        // Remove notification
        for entity in &existing {
            commands.entity(entity).despawn();
        }
    }
}

fn handle_notification_buttons(
    join_interaction: Query<&Interaction, (Changed<Interaction>, With<NotificationJoinButton>)>,
    dismiss_interaction: Query<
        &Interaction,
        (Changed<Interaction>, With<NotificationDismissButton>),
    >,
    mut notification: ResMut<LobbyNotification>,
    mut connection_status: ResMut<ConnectionStatus>,
    mut round_state: ResMut<MultiplayerRoundState>,
    mut active_exercises: ResMut<ActiveExercises>,
    mut session: ResMut<GameSession>,
    ws: Option<Res<WsConnection>>,
    beacons: Query<Entity, With<crate::game::beacon::BeaconVisual>>,
    mut commands: Commands,
) {
    for inter in &dismiss_interaction {
        if *inter == Interaction::Pressed {
            notification.dismissed = true;
        }
    }

    for inter in &join_interaction {
        if *inter == Interaction::Pressed {
            if connection_status.is_solo() {
                // End current solo round: despawn beacons
                for entity in &beacons {
                    commands.entity(entity).despawn();
                }

                // Send LeaveSolo and restore Connected state
                if let ConnectionStatus::ConnectedSolo {
                    my_player_id,
                    my_color_index,
                } = *connection_status
                {
                    if let Some(ws) = &ws {
                        ws.send_binary(encode(&ClientMessage::LeaveSolo));
                    }
                    *connection_status = ConnectionStatus::Connected {
                        my_player_id,
                        my_color_index,
                        send_timer: Timer::from_seconds(0.1, TimerMode::Repeating),
                    };
                }
                *round_state = MultiplayerRoundState::Lobby;
                // Reset session
                active_exercises.total_engaged = 0;
                active_exercises.cooldown_timer = 0.0;
                session.current_index = 0;
                session.correct_count = 0;
                session.wrong_count = 0;
                session.timeout_count = 0;
                session.coins = 0;
                session.prev_coins = 0;
                session.combo = 0;
                session.max_combo = 0;
            }
            // For players in an active multiplayer round, "Join" is informational —
            // they'll go to lobby when the round ends naturally.
        }
    }
}
