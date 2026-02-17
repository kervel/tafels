use std::time::Duration;

use bevy::prelude::*;

use super::{CharacterAnimations, CharacterMarker, CharacterState};

/// Once the GLTF scene is spawned, find the AnimationPlayer child and set up
/// the animation graph with Idle and Walk clips. Immediately starts the idle animation.
pub fn setup_character_animations(
    mut commands: Commands,
    mut animation_graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    characters: Query<Entity, (With<CharacterMarker>, Without<CharacterAnimations>)>,
    children: Query<&Children>,
    mut player_query: Query<&mut AnimationPlayer>,
    mut transitions_query: Query<&mut AnimationTransitions>,
) {
    for character_entity in &characters {
        let Some(player_entity) =
            find_animation_player(character_entity, &children, &player_query)
        else {
            continue;
        };

        // char_adventurer.glb animation indices (alphabetical order in GLTF):
        //  4 = CharacterArmature|Idle
        // 16 = CharacterArmature|Run
        // 22 = CharacterArmature|Walk
        let idle_clip: Handle<AnimationClip> = asset_server.load(
            GltfAssetLabel::Animation(4)
                .from_asset("models/char_adventurer/char_adventurer.glb"),
        );
        let walk_clip: Handle<AnimationClip> = asset_server.load(
            GltfAssetLabel::Animation(22)
                .from_asset("models/char_adventurer/char_adventurer.glb"),
        );
        let run_clip: Handle<AnimationClip> = asset_server.load(
            GltfAssetLabel::Animation(16)
                .from_asset("models/char_adventurer/char_adventurer.glb"),
        );

        let (graph, indices) = AnimationGraph::from_clips([idle_clip, walk_clip, run_clip]);
        let idle_node = indices[0];
        let walk_node = indices[1];
        let run_node = indices[2];
        let graph_handle = animation_graphs.add(graph);

        commands
            .entity(player_entity)
            .insert(AnimationGraphHandle(graph_handle))
            .insert(AnimationTransitions::new());

        commands.entity(character_entity).insert(CharacterAnimations {
            idle_node,
            walk_node,
            run_node,
            player_entity,
        });

        // Start idle animation immediately so the character isn't static on spawn
        if let Ok(mut player) = player_query.get_mut(player_entity) {
            if let Ok(mut transitions) = transitions_query.get_mut(player_entity) {
                transitions
                    .play(&mut player, idle_node, Duration::from_millis(200))
                    .repeat();
            }
        }
    }
}

fn find_animation_player(
    entity: Entity,
    children: &Query<&Children>,
    player_check: &Query<&mut AnimationPlayer>,
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

/// Switch between Idle and Walk animations based on CharacterState changes.
pub fn animate_character(
    characters: Query<(&CharacterState, &CharacterAnimations), Changed<CharacterState>>,
    mut players: Query<&mut AnimationPlayer>,
    mut transitions: Query<&mut AnimationTransitions>,
) {
    for (state, anims) in &characters {
        let Ok(mut player) = players.get_mut(anims.player_entity) else {
            continue;
        };
        let Ok(mut transition) = transitions.get_mut(anims.player_entity) else {
            continue;
        };

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
