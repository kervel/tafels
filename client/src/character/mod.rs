pub mod animation;
pub mod controller;

use bevy::prelude::*;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_character)
            .add_systems(Update, controller::snap_to_terrain)
            .add_systems(
                Update,
                (
                    controller::read_movement_input,
                    controller::apply_movement,
                    controller::update_character_state,
                )
                    .chain()
                    .run_if(in_state(crate::game::GameState::Playing)),
            )
            .add_systems(
                Update,
                (
                    animation::setup_character_animations,
                    animation::animate_character,
                )
                    .chain(),
            );
    }
}

#[derive(Component)]
pub struct CharacterController {
    pub speed: f32,
    pub running: bool,
}

impl CharacterController {
    pub fn effective_speed(&self) -> f32 {
        if self.running {
            self.speed * 1.5
        } else {
            self.speed
        }
    }
}

#[derive(Component, Default)]
pub struct MovementInput {
    pub direction: Vec2,
}

#[derive(Component, Default, Debug, PartialEq, Clone, Copy)]
pub enum CharacterState {
    #[default]
    Idle,
    Walking,
    Running,
}

#[derive(Component)]
pub struct CharacterMarker;

#[derive(Component)]
pub struct CharacterAnimations {
    pub idle_node: AnimationNodeIndex,
    pub walk_node: AnimationNodeIndex,
    pub run_node: AnimationNodeIndex,
    pub player_entity: Entity,
    pub graph_handle: Handle<AnimationGraph>,
    pub active_state: Option<CharacterState>,
}

fn spawn_character(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    terrain: Option<Res<crate::terrain::TerrainResource>>,
) {
    let spawn_pos = if let Some(terrain) = &terrain {
        let y = crate::terrain::heightmap::sample_height(&terrain.heightmap, 0.0, 0.0);
        Vec3::new(0.0, y, 0.0)
    } else {
        Vec3::ZERO
    };

    // char_adventurer from Poly Pizza (1.86m native height, 10K tris)
    commands.spawn((
        SceneRoot(asset_server.load(
            GltfAssetLabel::Scene(0).from_asset("models/char_adventurer/char_adventurer.glb"),
        )),
        Transform::from_translation(spawn_pos).with_scale(Vec3::splat(1.0)),
        CharacterMarker,
        CharacterController {
            speed: 18.0,
            running: false,
        },
        MovementInput::default(),
        CharacterState::default(),
    ));
}
