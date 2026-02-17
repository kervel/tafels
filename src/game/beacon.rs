use bevy::prelude::*;
use rand::prelude::*;

use super::exercise::{generate_exercise, ExerciseId, ExerciseState};
use super::panels::NEON_COLORS;
use super::{ActiveExercises, GameSession, GameState};
use crate::character::CharacterMarker;
use crate::collision::VegetationCollider;
use crate::effects::particles::{ParticleStyle, StyledParticleEvent};

pub struct BeaconPlugin;

impl Plugin for BeaconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_beacon,
                tick_world_lifetime,
                check_proximity_trigger,
                update_timer_text,
                animate_beacons,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BeaconState {
    Dormant,
    Activated,
    #[allow(dead_code)]
    Resolved,
}

#[derive(Component)]
pub struct WorldLifetime {
    pub remaining: f32,
    pub initial: f32,
}

#[derive(Component)]
pub struct BeaconVisual;

#[derive(Component)]
pub struct BeaconFacing(pub Vec3);

#[allow(clippy::too_many_arguments)]
fn spawn_beacon(
    mut commands: Commands,
    time: Res<Time>,
    session: Res<GameSession>,
    mut active_exercises: ResMut<ActiveExercises>,
    existing_beacons: Query<&Transform, With<ExerciseId>>,
    character: Query<&Transform, (With<CharacterMarker>, Without<ExerciseId>)>,
    vegetation: Query<
        &Transform,
        (
            With<VegetationCollider>,
            Without<CharacterMarker>,
            Without<ExerciseId>,
        ),
    >,
    terrain: Option<Res<crate::terrain::TerrainResource>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut styled_events: MessageWriter<StyledParticleEvent>,
) {
    // Tick cooldown
    if active_exercises.cooldown_timer > 0.0 {
        active_exercises.cooldown_timer -= time.delta_secs();
    }

    let beacon_count = existing_beacons.iter().count() as u32;
    if beacon_count >= active_exercises.target_concurrent {
        return;
    }
    if active_exercises.total_engaged >= session.total_exercises {
        return;
    }
    if active_exercises.cooldown_timer > 0.0 {
        return;
    }

    let Ok(char_tf) = character.single() else {
        return;
    };
    let Some(ref terrain) = terrain else {
        return;
    };

    let mut rng = rand::thread_rng();

    // Try to find a valid spawn position
    let mut spawn_pos = None;
    for _ in 0..20 {
        let angle = rng.r#gen_range(0.0..std::f32::consts::TAU);
        let dist = rng.r#gen_range(30.0..60.0_f32);
        let candidate = Vec3::new(
            char_tf.translation.x + angle.cos() * dist,
            0.0,
            char_tf.translation.z + angle.sin() * dist,
        );

        // Check minimum distance from player
        let dx = candidate.x - char_tf.translation.x;
        let dz = candidate.z - char_tf.translation.z;
        if (dx * dx + dz * dz).sqrt() < 15.0 {
            continue;
        }

        // Check minimum separation from other beacons
        let mut too_close = false;
        for beacon_tf in &existing_beacons {
            let bx = candidate.x - beacon_tf.translation.x;
            let bz = candidate.z - beacon_tf.translation.z;
            if (bx * bx + bz * bz).sqrt() < 10.0 {
                too_close = true;
                break;
            }
        }
        if too_close {
            continue;
        }

        // Check not inside vegetation
        let mut in_vegetation = false;
        for veg_tf in &vegetation {
            let vx = candidate.x - veg_tf.translation.x;
            let vz = candidate.z - veg_tf.translation.z;
            if (vx * vx + vz * vz).sqrt() < 3.0 {
                in_vegetation = true;
                break;
            }
        }
        if in_vegetation {
            continue;
        }

        spawn_pos = Some(candidate);
        break;
    }

    let Some(pos) = spawn_pos else {
        return;
    };

    let ground_y =
        crate::terrain::heightmap::sample_height(&terrain.heightmap, pos.x, pos.z);
    let spawn_center = Vec3::new(pos.x, ground_y, pos.z);

    // Random facing direction
    let facing_angle = rng.r#gen_range(0.0..std::f32::consts::TAU);
    let facing_dir = Vec3::new(facing_angle.cos(), 0.0, facing_angle.sin());

    let eid = active_exercises.next_exercise_id;
    active_exercises.next_exercise_id += 1;
    active_exercises.total_spawned += 1;

    let exercise = generate_exercise(&session.difficulty);

    // Random world-lifetime between 30-60 seconds
    let lifetime = rng.r#gen_range(30.0..60.0_f32);

    // Random beacon color
    let color_idx = rng.r#gen_range(0..NEON_COLORS.len());
    let neon = NEON_COLORS[color_idx];

    // Beacon visual: tall emissive capsule with point light
    let beacon_height = 3.5;
    let beacon_radius = 0.3;
    let beacon_mesh = meshes.add(Capsule3d::new(beacon_radius, beacon_height - beacon_radius * 2.0));
    let beacon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(neon[0], neon[1], neon[2]),
        // Low emissive so the beacon is subtly visible from all sides,
        // but external lighting dominates — the front PointLight makes
        // the front side dramatically brighter
        emissive: bevy::color::LinearRgba::new(
            neon[0] * 1.5,
            neon[1] * 1.5,
            neon[2] * 1.5,
            1.0,
        ),
        ..default()
    });

    // Front light offset: place a bright PointLight just in front of the beacon
    let front_offset = facing_dir * 2.5;

    // Spawn exercise entity with beacon visual as child
    commands
        .spawn((
            ExerciseId(eid),
            exercise,
            BeaconState::Dormant,
            WorldLifetime {
                remaining: lifetime,
                initial: lifetime,
            },
            BeaconFacing(facing_dir),
            Transform::from_translation(spawn_center),
        ))
        .with_children(|parent| {
            // Beacon visual mesh
            parent.spawn((
                BeaconVisual,
                Mesh3d(beacon_mesh),
                MeshMaterial3d(beacon_material),
                Transform::from_translation(Vec3::new(0.0, beacon_height * 0.5, 0.0)),
            ));
            // Bright point light in FRONT of the beacon — illuminates the capsule front + ground
            parent.spawn((
                BeaconVisual,
                PointLight {
                    color: Color::srgb(neon[0], neon[1], neon[2]),
                    intensity: 1_500_000.0,
                    range: 30.0,
                    shadows_enabled: false,
                    ..default()
                },
                Transform::from_translation(Vec3::new(
                    front_offset.x,
                    beacon_height * 0.5,
                    front_offset.z,
                )),
            ));
        });

    // Rising sparkles — beacon materializing
    styled_events.write(StyledParticleEvent {
        position: spawn_center + Vec3::Y * 1.0,
        color: Color::srgb(neon[0], neon[1], neon[2]),
        count: 20,
        speed: 2.0,
        lifetime: 1.5,
        size: 0.08,
        style: ParticleStyle::RisingSparkle,
    });

    active_exercises.cooldown_timer = 5.0;
}

fn tick_world_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut beacons: Query<(Entity, &BeaconState, &Transform, &mut WorldLifetime)>,
    mut styled_events: MessageWriter<StyledParticleEvent>,
) {
    for (entity, state, tf, mut lifetime) in &mut beacons {
        if *state != BeaconState::Dormant {
            continue;
        }
        lifetime.remaining -= time.delta_secs();
        if lifetime.remaining <= 0.0 {
            // Falling embers — beacon dissolving
            styled_events.write(StyledParticleEvent {
                position: tf.translation + Vec3::Y * 2.0,
                color: Color::srgb(1.0, 0.4, 0.1),
                count: 18,
                speed: 1.5,
                lifetime: 2.0,
                size: 0.1,
                style: ParticleStyle::FallingEmber,
            });

            commands.entity(entity).despawn();
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_proximity_trigger(
    mut commands: Commands,
    mut beacons: Query<(
        Entity,
        &ExerciseId,
        &mut BeaconState,
        &BeaconFacing,
        &Transform,
        &mut super::exercise::ActiveExercise,
    )>,
    beacon_visuals: Query<(Entity, &ChildOf), With<BeaconVisual>>,
    character: Query<&Transform, (With<CharacterMarker>, Without<ExerciseId>)>,
    mut active_exercises: ResMut<ActiveExercises>,
    session: Res<GameSession>,
    terrain: Option<Res<crate::terrain::TerrainResource>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut styled_events: MessageWriter<StyledParticleEvent>,
) {
    let Ok(char_tf) = character.single() else {
        return;
    };
    let Some(ref terrain) = terrain else {
        return;
    };

    if active_exercises.total_engaged >= session.total_exercises {
        return;
    }

    for (entity, exercise_id, mut state, facing, beacon_tf, mut exercise) in &mut beacons {
        if *state != BeaconState::Dormant {
            continue;
        }

        // XZ distance check
        let dx = char_tf.translation.x - beacon_tf.translation.x;
        let dz = char_tf.translation.z - beacon_tf.translation.z;
        let xz_dist = (dx * dx + dz * dz).sqrt();

        if xz_dist > 18.0 {
            continue;
        }

        // Forward arc check: player must be in front of the beacon
        let to_player = Vec3::new(dx, 0.0, dz).normalize_or_zero();
        let dot = facing.0.dot(to_player);
        // dot > -0.3 means the player is within about 120 degrees of the front
        if dot < -0.3 {
            continue;
        }

        // Activate!
        *state = BeaconState::Activated;
        exercise.state = ExerciseState::Active;
        active_exercises.total_engaged += 1;

        // Despawn beacon visual children
        for (vis_entity, child_of) in &beacon_visuals {
            if child_of.parent() == entity {
                commands.entity(vis_entity).despawn();
            }
        }

        // Spawn answer panels at beacon position
        let facing_toward = beacon_tf.translation + facing.0 * 10.0;
        super::panels::spawn_answer_panels(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            &terrain.heightmap,
            beacon_tf.translation,
            facing_toward,
            &exercise.choices,
            exercise.correct_answer,
            exercise_id.0,
        );

        // Compute the panel rotation for text alignment
        let to_facing = facing.0.normalize_or_zero();
        let text_rotation = Transform::IDENTITY
            .looking_to(-to_facing, Vec3::Y)
            .rotation;

        // Spawn question text above panels
        super::panels::spawn_question_text(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            &exercise.question_text(),
            beacon_tf.translation,
            text_rotation,
            exercise_id.0,
        );

        // Spawn timer text below question
        super::panels::spawn_timer_text(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut images,
            exercise.time_limit,
            beacon_tf.translation,
            text_rotation,
            exercise_id.0,
        );

        // Expanding ring — beacon activating
        let neon = NEON_COLORS[exercise_id.0 as usize % NEON_COLORS.len()];
        styled_events.write(StyledParticleEvent {
            position: beacon_tf.translation + Vec3::Y * 0.5,
            color: Color::srgb(neon[0], neon[1], neon[2]),
            count: 24,
            speed: 5.0,
            lifetime: 1.0,
            size: 0.15,
            style: ParticleStyle::Ring,
        });
    }
}

fn update_timer_text(
    exercises: Query<(&ExerciseId, &super::exercise::ActiveExercise), With<BeaconState>>,
    mut timer_texts: Query<(
        &mut super::panels::TimerText,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut mat_assets: ResMut<Assets<StandardMaterial>>,
) {
    for (mut timer_text, mat_handle) in &mut timer_texts {
        // Find matching exercise
        let Some((_, exercise)) = exercises
            .iter()
            .find(|(eid, _)| eid.0 == timer_text.exercise_id)
        else {
            continue;
        };

        let current_sec = exercise.time_remaining.ceil().max(0.0) as u32;
        if current_sec == timer_text.current_second {
            continue;
        }
        timer_text.current_second = current_sec;

        // Swap texture on the material
        let tex_index = (current_sec as usize).min(timer_text.prerendered.len().saturating_sub(1));
        if let Some(mat) = mat_assets.get_mut(mat_handle.id()) {
            let new_tex = timer_text.prerendered[tex_index].clone();
            mat.base_color_texture = Some(new_tex.clone());
            mat.emissive_texture = Some(new_tex);

            // Update emissive color based on time fraction
            let fraction = exercise.time_remaining / exercise.time_limit;
            if fraction > 0.5 {
                mat.emissive = bevy::color::LinearRgba::new(2.0, 6.0, 2.0, 1.0);
            } else if fraction > 0.25 {
                mat.emissive = bevy::color::LinearRgba::new(6.0, 6.0, 2.0, 1.0);
            } else {
                mat.emissive = bevy::color::LinearRgba::new(6.0, 2.0, 2.0, 1.0);
            }
        }
    }
}

fn animate_beacons(
    time: Res<Time>,
    beacons: Query<(&BeaconState, &WorldLifetime, &Children)>,
    mut visuals: Query<(&mut Transform, &mut Visibility), With<BeaconVisual>>,
) {
    let t = time.elapsed_secs();

    for (state, lifetime, children) in &beacons {
        if *state != BeaconState::Dormant {
            continue;
        }

        let fraction = lifetime.remaining / lifetime.initial;

        for child in children.iter() {
            let Ok((mut tf, mut vis)) = visuals.get_mut(child) else {
                continue;
            };

            // Gentle vertical bob
            tf.translation.y += 0.3 * (t * 2.0).sin() * time.delta_secs() * 2.0;

            if fraction < 0.30 {
                // Pulse scale
                let pulse = 1.0 + 0.2 * (t * 4.0).sin();
                tf.scale = Vec3::splat(pulse);
            }

            if fraction < 0.15 {
                // Rapid flicker
                let flicker_phase = (t * 10.0) as u32;
                *vis = if flicker_phase % 2 == 0 {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            } else {
                *vis = Visibility::Inherited;
            }
        }
    }
}
