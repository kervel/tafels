use std::time::Duration;

use bevy::prelude::*;

use super::{CharacterAnimations, CharacterMarker, CharacterState};
use crate::quality::CharacterModelAsset;

/// Once the GLTF scene is spawned, find the AnimationPlayer child and set up
/// the animation graph with Idle and Walk clips.
pub fn setup_character_animations(
    mut commands: Commands,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    model_asset: Res<CharacterModelAsset>,
    characters: Query<Entity, (With<CharacterMarker>, Without<CharacterAnimations>)>,
    children: Query<&Children>,
    player_query: Query<&AnimationPlayer>,
) {
    for character_entity in &characters {
        let Some(player_entity) = find_animation_player(character_entity, &children, &player_query)
        else {
            continue;
        };

        // Animation indices (alphabetical order in GLTF, same for both LOD variants):
        //  4 = CharacterArmature|Idle
        // 16 = CharacterArmature|Run
        // 22 = CharacterArmature|Walk
        let idle_clip: Handle<AnimationClip> = asset_server.load(
            GltfAssetLabel::Animation(4).from_asset(model_asset.path),
        );
        let walk_clip: Handle<AnimationClip> = asset_server.load(
            GltfAssetLabel::Animation(22).from_asset(model_asset.path),
        );
        let run_clip: Handle<AnimationClip> = asset_server.load(
            GltfAssetLabel::Animation(16).from_asset(model_asset.path),
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

        commands
            .entity(character_entity)
            .insert(CharacterAnimations {
                idle_node,
                walk_node,
                run_node,
                player_entity,
                graph_handle,
                active_state: None,
            });
    }
}

fn find_animation_player_mut(
    entity: Entity,
    children: &Query<&Children>,
    players: &Query<&mut AnimationPlayer>,
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

/// Re-discover the AnimationPlayer if the cached reference is stale,
/// and apply animations based on CharacterState.
pub fn animate_character(
    mut commands: Commands,
    mut characters: Query<(Entity, &CharacterState, &mut CharacterAnimations)>,
    children: Query<&Children>,
    mut players: Query<&mut AnimationPlayer>,
    mut transitions: Query<&mut AnimationTransitions>,
) {
    for (char_entity, state, mut anims) in &mut characters {
        // Check if cached player entity still has an AnimationPlayer
        if players.get(anims.player_entity).is_err() {
            // Re-discover the AnimationPlayer in the hierarchy
            if let Some(new_player) = find_animation_player_mut(char_entity, &children, &players) {
                if new_player != anims.player_entity {
                    anims.player_entity = new_player;
                    anims.active_state = None; // Force re-play after recovery
                    commands
                        .entity(new_player)
                        .insert(AnimationGraphHandle(anims.graph_handle.clone()))
                        .insert(AnimationTransitions::new());
                    return; // Wait one frame for deferred commands to apply
                }
            }
            continue;
        }

        // Only play when state actually changes
        if anims.active_state == Some(*state) {
            continue;
        }

        let Ok(mut player) = players.get_mut(anims.player_entity) else {
            continue;
        };
        let Ok(mut transition) = transitions.get_mut(anims.player_entity) else {
            continue;
        };

        anims.active_state = Some(*state);

        match state {
            CharacterState::Idle => {
                transition
                    .play(&mut player, anims.idle_node, Duration::from_millis(200))
                    .repeat();
            }
            CharacterState::Walking => {
                transition
                    .play(&mut player, anims.walk_node, Duration::from_millis(200))
                    .set_speed(2.0)
                    .repeat();
            }
            CharacterState::Running => {
                transition
                    .play(&mut player, anims.run_node, Duration::from_millis(200))
                    .set_speed(1.5)
                    .repeat();
            }
        }
    }
}
