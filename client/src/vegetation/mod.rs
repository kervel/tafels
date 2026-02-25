pub mod placement;

use bevy::prelude::*;

pub struct VegetationPlugin;

impl Plugin for VegetationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            spawn_vegetation.after(crate::terrain::setup_terrain),
        )
        .add_systems(Update, distance_cull_vegetation);
    }
}

use crate::character::CharacterMarker;
use crate::collision::VegetationCollider;
use crate::grass::WindSway;
use crate::terrain;

#[derive(Component, Clone, Copy)]
pub enum VegetationType {
    PineSapling,
    FirSapling,
    ProceduralConifer,
    Shrub,
    RockOutcrop,
    DirtPatch,
}

/// Max render distance for vegetation - matches fog
const CULL_DISTANCE: f32 = 120.0;
const CULL_DISTANCE_SQ: f32 = CULL_DISTANCE * CULL_DISTANCE;

fn spawn_vegetation(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    terrain_res: Option<Res<terrain::TerrainResource>>,
) {
    let Some(terrain) = terrain_res else { return };

    // density 0.50 = ~50% of grid cells get vegetation
    let positions = placement::generate_vegetation_positions(&terrain.heightmap, 0.50, 12345);

    // Game-ready low-poly models from Poly Pizza (CC0, ~1-3K triangles each)
    let spruce_scene: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/pp_spruce2/pp_spruce2.glb"));
    let pine_scene: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/polypizza_pine/pine.glb"));
    let conifer_scene: Handle<Scene> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("models/pp_conifer2/pp_conifer2.glb"));

    // Poly Haven rock - only on native; causes rendering artifacts (bright flashes)
    // on WebGL2 that persist even after stripping all PBR textures.
    #[cfg(not(target_arch = "wasm32"))]
    let rock_scene: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/rock_07/rock_07_lod.glb"));

    let shrub_scene: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/alpine_shrub.glb"));

    let dirt_scene: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("models/dirt_patch.glb"));

    for instance in positions {
        // Skip rocks on WASM - they cause bright flashes on WebGL2
        #[cfg(target_arch = "wasm32")]
        if matches!(instance.vegetation_type, VegetationType::RockOutcrop) {
            continue;
        }

        let scene = match instance.vegetation_type {
            VegetationType::PineSapling => pine_scene.clone(),
            VegetationType::FirSapling => spruce_scene.clone(),
            VegetationType::ProceduralConifer => conifer_scene.clone(),
            VegetationType::Shrub => shrub_scene.clone(),
            #[cfg(not(target_arch = "wasm32"))]
            VegetationType::RockOutcrop => rock_scene.clone(),
            #[cfg(target_arch = "wasm32")]
            VegetationType::RockOutcrop => unreachable!(),
            VegetationType::DirtPatch => dirt_scene.clone(),
        };

        let base_rotation = Quat::from_rotation_y(instance.rotation_y);

        // Wind sway strength: trees sway gently, shrubs a bit more, rocks/dirt not at all
        let wind_strength = match instance.vegetation_type {
            VegetationType::PineSapling
            | VegetationType::FirSapling
            | VegetationType::ProceduralConifer => 0.02,
            VegetationType::Shrub => 0.05,
            VegetationType::RockOutcrop | VegetationType::DirtPatch => 0.0,
        };

        // Collision radii: trees ~1.5m, rocks ~1.0m, shrubs/grass/dirt: none
        let collider_radius = match instance.vegetation_type {
            VegetationType::PineSapling
            | VegetationType::FirSapling
            | VegetationType::ProceduralConifer => Some(1.5),
            VegetationType::RockOutcrop => Some(1.0),
            VegetationType::Shrub | VegetationType::DirtPatch => None,
        };

        let mut entity = commands.spawn((
            SceneRoot(scene),
            Transform::from_translation(instance.position)
                .with_rotation(base_rotation)
                .with_scale(Vec3::splat(instance.scale)),
            instance.vegetation_type,
        ));

        if let Some(radius) = collider_radius {
            entity.insert(VegetationCollider { radius });
        }

        if wind_strength > 0.0 {
            // Deterministic phase from position
            let phase = (instance.position.x * 12.9898 + instance.position.z * 78.233)
                .sin()
                .fract()
                .abs()
                * std::f32::consts::TAU;
            entity.insert(WindSway {
                phase,
                base_rotation,
                strength: wind_strength,
            });
        }
    }
}

/// Hide vegetation entities beyond fog distance so the GPU doesn't render them
fn distance_cull_vegetation(
    character_query: Query<&Transform, With<CharacterMarker>>,
    mut veg_query: Query<
        (&Transform, &mut Visibility),
        (With<VegetationType>, Without<CharacterMarker>),
    >,
) {
    let Some(char_tf) = character_query.iter().next() else {
        return;
    };
    let char_pos = char_tf.translation;

    for (veg_tf, mut vis) in &mut veg_query {
        let dx = veg_tf.translation.x - char_pos.x;
        let dz = veg_tf.translation.z - char_pos.z;
        let dist_sq = dx * dx + dz * dz;

        if dist_sq > CULL_DISTANCE_SQ {
            *vis = Visibility::Hidden;
        } else {
            *vis = Visibility::Inherited;
        }
    }
}
