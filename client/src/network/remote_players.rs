use std::time::Duration;

use bevy::prelude::*;
use bevy::pbr::StandardMaterial;
use tafels_shared::protocol::AnimationState;

use super::{ConnectionStatus, RemotePlayerJoined, RemotePlayerLeft, RemotePlayerUpdated};
use crate::character::CharacterMarker;
use crate::hud::GameFont;
use crate::collision::VegetationCollider;
use crate::game::GameState;

/// Color palette for distinguishing players.
const PLAYER_COLORS: &[Color] = &[
    Color::srgb(0.2, 0.6, 1.0),  // blue
    Color::srgb(1.0, 0.3, 0.3),  // red
    Color::srgb(0.3, 1.0, 0.3),  // green
    Color::srgb(1.0, 0.8, 0.2),  // yellow
    Color::srgb(0.8, 0.3, 1.0),  // purple
    Color::srgb(1.0, 0.5, 0.2),  // orange
];

pub struct RemotePlayersPlugin;

impl Plugin for RemotePlayersPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_remote_players,
                despawn_remote_players,
                despawn_remote_players_on_solo,
                update_remote_positions,
                interpolate_remote_players,
                setup_remote_animations,
                animate_remote_players,
                tint_remote_players,
                tint_local_player,
                cull_distant_remote_players,
            )
                .run_if(in_state(GameState::Playing)),
        )
        .add_systems(OnExit(GameState::Playing), despawn_all_remote_players);
    }
}

#[derive(Component)]
pub struct RemotePlayer {
    pub player_id: u32,
    pub color_index: u8,
    pub tinted: bool,
}

#[derive(Component)]
pub struct InterpolationState {
    pub prev_pos: Vec3,
    pub target_pos: Vec3,
    pub prev_yaw: f32,
    pub target_yaw: f32,
    pub timer: f32,
    pub animation: AnimationState,
}

#[derive(Component)]
pub struct RemotePlayerNameplate;

/// Marker: local player has been tinted.
#[derive(Component)]
pub struct LocalPlayerTinted;

/// Animation state for a remote player (similar to CharacterAnimations).
#[derive(Component)]
pub struct RemoteAnimations {
    pub idle_node: AnimationNodeIndex,
    pub walk_node: AnimationNodeIndex,
    pub run_node: AnimationNodeIndex,
    pub player_entity: Entity,
    pub graph_handle: Handle<AnimationGraph>,
    pub active_animation: Option<AnimationState>,
}

fn spawn_remote_players(
    mut commands: Commands,
    mut events: MessageReader<RemotePlayerJoined>,
    asset_server: Res<AssetServer>,
    existing: Query<&RemotePlayer>,
    game_font: Res<GameFont>,
    connection_status: Res<ConnectionStatus>,
) {
    if connection_status.is_solo() {
        events.read().last(); // drain events
        return;
    }
    for RemotePlayerJoined(ps) in events.read() {
        info!(
            "Remote player {} joined at ({:.1}, {:.1}, {:.1})",
            ps.player_id, ps.x, ps.y, ps.z
        );
        // Check if already spawned
        if existing.iter().any(|rp| rp.player_id == ps.player_id) {
            info!("  -> already spawned, skipping");
            continue;
        }

        let pos = Vec3::new(ps.x, ps.y, ps.z);

        commands
            .spawn((
                RemotePlayer {
                    player_id: ps.player_id,
                    color_index: ps.color_index,
                    tinted: false,
                },
                InterpolationState {
                    prev_pos: pos,
                    target_pos: pos,
                    prev_yaw: ps.yaw,
                    target_yaw: ps.yaw,
                    timer: 1.0,
                    animation: ps.animation,
                },
                VegetationCollider { radius: 0.3 },
                SceneRoot(
                    asset_server.load(
                        GltfAssetLabel::Scene(0)
                            .from_asset("models/char_adventurer/char_adventurer.glb"),
                    ),
                ),
                Transform::from_translation(pos).with_rotation(Quat::from_rotation_y(ps.yaw)),
            ))
            .with_children(|parent| {
                // Floating nameplate above character
                parent.spawn((
                    RemotePlayerNameplate,
                    Text2d::new(format!("Player {}", ps.player_id)),
                    TextFont {
                        font: game_font.0.clone(),
                        font_size: 24.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Transform::from_translation(Vec3::new(0.0, 2.5, 0.0))
                        .with_scale(Vec3::splat(0.02)),
                ));
            });
    }
}

fn despawn_remote_players(
    mut commands: Commands,
    mut events: MessageReader<RemotePlayerLeft>,
    players: Query<(Entity, &RemotePlayer)>,
) {
    for RemotePlayerLeft { player_id } in events.read() {
        for (entity, rp) in &players {
            if rp.player_id == *player_id {
                commands.entity(entity).despawn();
            }
        }
    }
}

fn despawn_all_remote_players(mut commands: Commands, players: Query<Entity, With<RemotePlayer>>) {
    for entity in &players {
        commands.entity(entity).despawn();
    }
}

/// Despawn all remote players when entering solo mode.
fn despawn_remote_players_on_solo(
    mut commands: Commands,
    status: Res<ConnectionStatus>,
    players: Query<Entity, With<RemotePlayer>>,
) {
    if status.is_changed() && status.is_solo() {
        for entity in &players {
            commands.entity(entity).despawn();
        }
    }
}

fn update_remote_positions(
    mut events: MessageReader<RemotePlayerUpdated>,
    mut players: Query<(&RemotePlayer, &Transform, &mut InterpolationState)>,
    status: Res<ConnectionStatus>,
) {
    if status.is_solo() {
        events.read().last(); // drain events
        return;
    }
    let my_id = match &*status {
        ConnectionStatus::Connected { my_player_id, .. } => Some(*my_player_id),
        _ => None,
    };

    for RemotePlayerUpdated(states) in events.read() {
        for ps in states {
            // Skip our own state
            if Some(ps.player_id) == my_id {
                continue;
            }

            for (rp, transform, mut interp) in &mut players {
                if rp.player_id == ps.player_id {
                    let new_pos = Vec3::new(ps.x, ps.y, ps.z);
                    // Only restart interpolation if state actually changed
                    if new_pos.distance_squared(interp.target_pos) > 0.0001
                        || (ps.yaw - interp.target_yaw).abs() > 0.001
                    {
                        interp.prev_pos = transform.translation;
                        interp.target_pos = new_pos;
                        interp.prev_yaw = interp.target_yaw;
                        interp.target_yaw = ps.yaw;
                        interp.timer = 0.0;
                    }
                    interp.animation = ps.animation;
                }
            }
        }
    }
}

fn interpolate_remote_players(
    time: Res<Time>,
    mut players: Query<(&mut Transform, &mut InterpolationState), With<RemotePlayer>>,
) {
    let interval = 0.05; // 50ms server tick

    for (mut transform, mut interp) in &mut players {
        interp.timer += time.delta_secs();
        let t = (interp.timer / interval).clamp(0.0, 1.0);

        transform.translation = interp.prev_pos.lerp(interp.target_pos, t);

        // Smooth yaw interpolation
        let yaw_diff = (interp.target_yaw - interp.prev_yaw + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let yaw = interp.prev_yaw + yaw_diff * t;
        transform.rotation = Quat::from_rotation_y(yaw);
    }
}

/// Set up animation graphs on remote player entities once their GLTF scene loads.
fn setup_remote_animations(
    mut commands: Commands,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    remote_players: Query<Entity, (With<RemotePlayer>, Without<RemoteAnimations>)>,
    children: Query<&Children>,
    player_query: Query<&AnimationPlayer>,
) {
    for entity in &remote_players {
        let Some(player_entity) = find_animation_player(entity, &children, &player_query) else {
            continue;
        };

        // Same animation clips as the local character
        let idle_clip: Handle<AnimationClip> = asset_server.load(
            GltfAssetLabel::Animation(4).from_asset("models/char_adventurer/char_adventurer.glb"),
        );
        let walk_clip: Handle<AnimationClip> = asset_server.load(
            GltfAssetLabel::Animation(22).from_asset("models/char_adventurer/char_adventurer.glb"),
        );
        let run_clip: Handle<AnimationClip> = asset_server.load(
            GltfAssetLabel::Animation(16).from_asset("models/char_adventurer/char_adventurer.glb"),
        );

        let (graph, indices) = AnimationGraph::from_clips([idle_clip, walk_clip, run_clip]);
        let idle_node = indices[0];
        let walk_node = indices[1];
        let run_node = indices[2];
        let graph_handle = animation_graphs.add(graph);

        commands
            .entity(player_entity)
            .insert(AnimationGraphHandle(graph_handle.clone()))
            .insert(AnimationTransitions::new());

        commands.entity(entity).insert(RemoteAnimations {
            idle_node,
            walk_node,
            run_node,
            player_entity,
            graph_handle,
            active_animation: None,
        });
    }
}

/// Drive animations on remote players based on their InterpolationState.
fn animate_remote_players(
    mut commands: Commands,
    mut remotes: Query<(Entity, &InterpolationState, &mut RemoteAnimations)>,
    children: Query<&Children>,
    mut players: Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) {
    for (entity, interp, mut anims) in &mut remotes {
        // Re-discover AnimationPlayer if stale
        if players.get(anims.player_entity).is_err() {
            if let Some(new_player) = find_animation_player_mut(entity, &children, &players)
                && new_player != anims.player_entity
            {
                info!(
                    "Re-discovered AnimationPlayer for remote {:?}: {:?} -> {:?}",
                    entity, anims.player_entity, new_player
                );
                anims.player_entity = new_player;
                anims.active_animation = None;
                commands
                    .entity(new_player)
                    .insert(AnimationGraphHandle(anims.graph_handle.clone()))
                    .insert(AnimationTransitions::new());
                continue;
            }
            continue;
        }

        if anims.active_animation == Some(interp.animation) {
            continue;
        }

        let Ok((mut player, mut transition)) = players.get_mut(anims.player_entity) else {
            continue;
        };

        info!(
            "Remote animation: {:?} -> {:?} (entity {:?}, player_entity {:?})",
            anims.active_animation, interp.animation, entity, anims.player_entity
        );
        anims.active_animation = Some(interp.animation);

        match interp.animation {
            AnimationState::Idle => {
                transition
                    .play(&mut player, anims.idle_node, Duration::from_millis(200))
                    .repeat();
            }
            AnimationState::Walking => {
                transition
                    .play(&mut player, anims.walk_node, Duration::from_millis(200))
                    .set_speed(2.0)
                    .repeat();
            }
            AnimationState::Running => {
                transition
                    .play(&mut player, anims.run_node, Duration::from_millis(200))
                    .set_speed(1.5)
                    .repeat();
            }
        }
    }
}

/// Hide remote players that are too far from the local character.
fn cull_distant_remote_players(
    character: Query<
        &Transform,
        (
            With<crate::character::CharacterMarker>,
            Without<RemotePlayer>,
        ),
    >,
    mut remote_players: Query<(&Transform, &mut Visibility), With<RemotePlayer>>,
) {
    let Ok(char_tf) = character.single() else {
        return;
    };

    const CULL_DISTANCE: f32 = 150.0;

    for (remote_tf, mut vis) in &mut remote_players {
        let dist_sq = char_tf.translation.distance_squared(remote_tf.translation);
        *vis = if dist_sq > CULL_DISTANCE * CULL_DISTANCE {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };
    }
}

/// Apply a color tint to remote player meshes once they're loaded.
/// Clones the shared material so the local player's material isn't affected.
fn tint_remote_players(
    mut commands: Commands,
    mut remote_players: Query<(Entity, &mut RemotePlayer)>,
    children: Query<&Children>,
    mesh_handles: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, mut rp) in &mut remote_players {
        if rp.tinted {
            continue;
        }
        let color = PLAYER_COLORS[rp.color_index as usize % PLAYER_COLORS.len()];
        let mut found_any = false;
        for desc in iter_descendants(entity, &children) {
            if let Ok((mesh_entity, mat_handle)) = mesh_handles.get(desc) {
                if let Some(mat) = materials.get(&mat_handle.0) {
                    // Clone the material so we don't mutate the shared asset
                    let mut tinted = mat.clone();
                    tinted.base_color = color;
                    let new_handle = materials.add(tinted);
                    commands
                        .entity(mesh_entity)
                        .insert(MeshMaterial3d(new_handle));
                    found_any = true;
                }
            }
        }
        if found_any {
            rp.tinted = true;
        }
    }
}

/// Apply the server-assigned color tint to the local player's model.
fn tint_local_player(
    mut commands: Commands,
    status: Res<ConnectionStatus>,
    local_player: Query<Entity, (With<CharacterMarker>, Without<LocalPlayerTinted>)>,
    children: Query<&Children>,
    mesh_handles: Query<(Entity, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let ConnectionStatus::Connected {
        my_color_index, ..
    } = &*status
    else {
        return;
    };

    for entity in &local_player {
        let color = PLAYER_COLORS[*my_color_index as usize % PLAYER_COLORS.len()];
        let mut found_any = false;
        for desc in iter_descendants(entity, &children) {
            if let Ok((mesh_entity, mat_handle)) = mesh_handles.get(desc) {
                if let Some(mat) = materials.get(&mat_handle.0) {
                    let mut tinted = mat.clone();
                    tinted.base_color = color;
                    let new_handle = materials.add(tinted);
                    commands
                        .entity(mesh_entity)
                        .insert(MeshMaterial3d(new_handle));
                    found_any = true;
                }
            }
        }
        if found_any {
            commands.entity(entity).insert(LocalPlayerTinted);
        }
    }
}

/// Iterate all descendants of an entity recursively.
fn iter_descendants(
    entity: Entity,
    children: &Query<&Children>,
) -> Vec<Entity> {
    let mut result = Vec::new();
    let mut stack = vec![entity];
    while let Some(e) = stack.pop() {
        if let Ok(kids) = children.get(e) {
            for child in kids.iter() {
                result.push(child);
                stack.push(child);
            }
        }
    }
    result
}

fn find_animation_player(
    entity: Entity,
    children: &Query<&Children>,
    player_check: &Query<&AnimationPlayer>,
) -> Option<Entity> {
    if player_check.get(entity).is_ok() {
        return Some(entity);
    }
    if let Ok(kids) = children.get(entity) {
        for child in kids.iter() {
            if let Some(found) = find_animation_player(child, children, player_check) {
                return Some(found);
            }
        }
    }
    None
}

fn find_animation_player_mut(
    entity: Entity,
    children: &Query<&Children>,
    players: &Query<(&mut AnimationPlayer, &mut AnimationTransitions)>,
) -> Option<Entity> {
    if players.get(entity).is_ok() {
        return Some(entity);
    }
    if let Ok(kids) = children.get(entity) {
        for child in kids.iter() {
            if let Some(found) = find_animation_player_mut(child, children, players) {
                return Some(found);
            }
        }
    }
    None
}
