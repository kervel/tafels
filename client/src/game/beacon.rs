use bevy::prelude::*;
use rand::prelude::*;

use super::exercise::{ActiveExercise, ExerciseId, ExerciseState, Operation, generate_exercise};
use super::panels::NEON_COLORS;
use super::scoring::PendingAnswer;
use super::{ActiveExercises, GameSession, GameState};
use crate::character::CharacterMarker;
use crate::collision::VegetationCollider;
use crate::effects::particles::{ParticleStyle, StyledParticleEvent};
use crate::network::{
    BeaconActivatedEvent, BeaconExpiredEvent, BeaconResolvedEvent, BeaconSpawnedEvent,
    ConnectionStatus, WsConnection,
};
use tafels_shared::constants::MIN_BEACON_SEPARATION;
use tafels_shared::protocol::{ClientMessage, encode};

/// How much panels face the player vs the beacon's pre-set direction at activation.
/// 0.0 = panels face beacon direction (current behaviour)
/// 1.0 = panels face the player who activated them
const PANEL_FACE_PLAYER_BLEND: f32 = 0.7;

pub struct BeaconPlugin;

impl Plugin for BeaconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_beacon,
                spawn_server_beacon,
                tick_world_lifetime,
                check_proximity_trigger,
                update_timer_text,
                animate_beacons,
                handle_beacon_resolved,
                handle_beacon_expired,
                handle_beacon_activated,
                send_answer_to_server,
            )
                .run_if(in_state(GameState::Playing)),
        );
    }
}

/// Tracks which server beacon_id this entity corresponds to (multiplayer only).
#[derive(Component)]
pub struct ServerBeaconId(pub u32);

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

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
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
    connection_status: Res<ConnectionStatus>,
) {
    // In multiplayer mode (or while connecting), beacons are managed by the server.
    // In solo mode or disconnected mode, spawn beacons locally.
    if !matches!(
        *connection_status,
        ConnectionStatus::Disconnected | ConnectionStatus::ConnectedSolo { .. }
    ) {
        return;
    }
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
            if (bx * bx + bz * bz).sqrt() < MIN_BEACON_SEPARATION {
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

    let ground_y = crate::terrain::heightmap::sample_height(&terrain.heightmap, pos.x, pos.z);
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
    let beacon_mesh = meshes.add(Capsule3d::new(
        beacon_radius,
        beacon_height - beacon_radius * 2.0,
    ));
    let beacon_material = materials.add(StandardMaterial {
        base_color: Color::srgb(neon[0], neon[1], neon[2]),
        // Low emissive so the beacon is subtly visible from all sides,
        // but external lighting dominates — the front PointLight makes
        // the front side dramatically brighter
        emissive: bevy::color::LinearRgba::new(neon[0] * 1.5, neon[1] * 1.5, neon[2] * 1.5, 1.0),
        ..default()
    });

    // Front light offset: place a bright PointLight just in front of the beacon
    #[cfg(not(target_arch = "wasm32"))]
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
            // Bright point light in FRONT of the beacon — illuminates the capsule front + ground.
            // Skipped on WASM: point lights are expensive on WebGL2 forward rendering.
            #[cfg(not(target_arch = "wasm32"))]
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

    active_exercises.cooldown_timer = 0.5;
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
        Option<&ServerBeaconId>,
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
    ws: Option<Res<WsConnection>>,
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

    for (entity, exercise_id, mut state, facing, beacon_tf, mut exercise, server_beacon_id) in &mut beacons {
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

        // Notify server (multiplayer)
        if let (Some(ServerBeaconId(bid)), Some(ws)) = (server_beacon_id, &ws) {
            let msg = ClientMessage::ActivateBeacon { beacon_id: *bid };
            ws.send_binary(encode(&msg));
        }

        // Despawn beacon visual children
        for (vis_entity, child_of) in &beacon_visuals {
            if child_of.parent() == entity {
                commands.entity(vis_entity).despawn();
            }
        }

        // Blend beacon facing with player direction for panel orientation
        let beacon_dir = facing.0.normalize_or_zero();
        let to_player_dir = Vec3::new(dx, 0.0, dz).normalize_or_zero();
        let blended_dir = (beacon_dir * (1.0 - PANEL_FACE_PLAYER_BLEND)
            + to_player_dir * PANEL_FACE_PLAYER_BLEND)
            .normalize_or_zero();

        // Spawn answer panels at beacon position
        let facing_toward = beacon_tf.translation + blended_dir * 10.0;
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
            true, // interactive — this player activated it
        );

        // Compute the panel rotation for text alignment
        let text_rotation = Transform::IDENTITY.looking_to(-blended_dir, Vec3::Y).rotation;

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
                *vis = if flicker_phase.is_multiple_of(2) {
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

// ===== Server-managed beacon systems (multiplayer) =====

/// Parse a question_text like "12 x 3 = ?" into (Operation, operand_a, operand_b).
fn parse_question_text(text: &str) -> (Operation, u32, u32) {
    if let Some((left, _)) = text.split_once(" = ") {
        if let Some((a, b)) = left.split_once(" x ") {
            let a = a.trim().parse().unwrap_or(0);
            let b = b.trim().parse().unwrap_or(0);
            return (Operation::Multiply, a, b);
        }
        if let Some((a, b)) = left.split_once(" / ") {
            let a = a.trim().parse().unwrap_or(0);
            let b = b.trim().parse().unwrap_or(0);
            return (Operation::Divide, a, b);
        }
    }
    (Operation::Multiply, 0, 0)
}

/// Spawn beacon entities from server BeaconSpawnedEvent messages.
#[allow(clippy::too_many_arguments)]
fn spawn_server_beacon(
    mut commands: Commands,
    mut events: MessageReader<BeaconSpawnedEvent>,
    connection_status: Res<ConnectionStatus>,
    mut active_exercises: ResMut<ActiveExercises>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut styled_events: MessageWriter<StyledParticleEvent>,
) {
    if !connection_status.is_online() {
        // Drain events if offline (shouldn't happen, but be safe)
        for _ in events.read() {}
        return;
    }

    for BeaconSpawnedEvent(info) in events.read() {
        let (operation, operand_a, operand_b) = parse_question_text(&info.question_text);
        let correct_answer = info.choices[info.correct_index as usize];

        let eid = active_exercises.next_exercise_id;
        active_exercises.next_exercise_id += 1;

        let exercise = ActiveExercise {
            operation,
            operand_a,
            operand_b,
            correct_answer,
            choices: info.choices,
            time_remaining: info.lifetime,
            time_limit: info.lifetime,
            state: ExerciseState::Pending,
        };

        let spawn_center = Vec3::new(info.x, info.y, info.z);

        // Deterministic facing and color from beacon_id so all clients agree
        let facing_angle = (info.beacon_id as f32) * 2.654_435_8; // golden angle in radians
        let facing_dir = Vec3::new(facing_angle.cos(), 0.0, facing_angle.sin());

        let color_idx = info.beacon_id as usize % NEON_COLORS.len();
        let neon = NEON_COLORS[color_idx];
        let base_color = Color::srgb(neon[0], neon[1], neon[2]);
        let emissive =
            bevy::color::LinearRgba::new(neon[0] * 1.5, neon[1] * 1.5, neon[2] * 1.5, 1.0);

        let beacon_height = 3.5;
        let beacon_radius = 0.3;
        let beacon_mesh = meshes.add(Capsule3d::new(
            beacon_radius,
            beacon_height - beacon_radius * 2.0,
        ));
        let beacon_material = materials.add(StandardMaterial {
            base_color,
            emissive,
            ..default()
        });

        #[cfg(not(target_arch = "wasm32"))]
        let front_offset = facing_dir * 2.5;

        commands
            .spawn((
                ExerciseId(eid),
                ServerBeaconId(info.beacon_id),
                exercise,
                BeaconState::Dormant,
                WorldLifetime {
                    remaining: info.lifetime,
                    initial: info.lifetime,
                },
                BeaconFacing(facing_dir),
                Transform::from_translation(spawn_center),
            ))
            .with_children(|parent| {
                parent.spawn((
                    BeaconVisual,
                    Mesh3d(beacon_mesh),
                    MeshMaterial3d(beacon_material),
                    Transform::from_translation(Vec3::new(0.0, beacon_height * 0.5, 0.0)),
                ));
                #[cfg(not(target_arch = "wasm32"))]
                parent.spawn((
                    BeaconVisual,
                    PointLight {
                        color: base_color,
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

        styled_events.write(StyledParticleEvent {
            position: spawn_center + Vec3::Y * 1.0,
            color: base_color,
            count: 20,
            speed: 2.0,
            lifetime: 1.5,
            size: 0.08,
            style: ParticleStyle::RisingSparkle,
        });
    }
}

/// Handle server beacon resolved events: despawn the beacon and its panels.
fn handle_beacon_resolved(
    mut commands: Commands,
    mut events: MessageReader<BeaconResolvedEvent>,
    beacons: Query<(Entity, &ServerBeaconId, &ExerciseId, &Transform)>,
    panels: Query<(Entity, &super::panels::AnswerPanel)>,
    poles: Query<(Entity, &ExerciseId), With<super::panels::PanelPole>>,
    question_texts: Query<(Entity, &super::panels::QuestionText)>,
    timer_texts: Query<(Entity, &super::panels::TimerText)>,
    mut styled_events: MessageWriter<StyledParticleEvent>,
) {
    for BeaconResolvedEvent {
        beacon_id,
        claimed_by: _,
    } in events.read()
    {
        for (entity, sbid, eid, tf) in &beacons {
            if sbid.0 == *beacon_id {
                // Celebration burst
                styled_events.write(StyledParticleEvent {
                    position: tf.translation + Vec3::Y * 1.0,
                    color: Color::srgb(0.3, 1.0, 0.4),
                    count: 30,
                    speed: 5.0,
                    lifetime: 1.5,
                    size: 0.12,
                    style: ParticleStyle::Ring,
                });

                // Clean up panels/poles/texts associated with this exercise
                let exercise_id = eid.0;
                for (e, panel) in &panels {
                    if panel.exercise_id == exercise_id {
                        commands.entity(e).despawn();
                    }
                }
                for (e, pole_eid) in &poles {
                    if pole_eid.0 == exercise_id {
                        commands.entity(e).despawn();
                    }
                }
                for (e, qt) in &question_texts {
                    if qt.exercise_id == exercise_id {
                        commands.entity(e).despawn();
                    }
                }
                for (e, tt) in &timer_texts {
                    if tt.exercise_id == exercise_id {
                        commands.entity(e).despawn();
                    }
                }

                commands.entity(entity).despawn();
            }
        }
    }
}

/// Handle server beacon expired events: despawn the beacon with dissolve effects.
#[allow(clippy::too_many_arguments)]
fn handle_beacon_expired(
    mut commands: Commands,
    mut events: MessageReader<BeaconExpiredEvent>,
    beacons: Query<(Entity, &ServerBeaconId, &Transform, &ExerciseId)>,
    panels: Query<(Entity, &super::panels::AnswerPanel)>,
    poles: Query<(Entity, &ExerciseId), With<super::panels::PanelPole>>,
    question_texts: Query<(Entity, &super::panels::QuestionText)>,
    timer_texts: Query<(Entity, &super::panels::TimerText)>,
    mut styled_events: MessageWriter<StyledParticleEvent>,
) {
    for BeaconExpiredEvent { beacon_id } in events.read() {
        for (entity, sbid, tf, eid) in &beacons {
            if sbid.0 == *beacon_id {
                // Falling embers dissolve effect
                styled_events.write(StyledParticleEvent {
                    position: tf.translation + Vec3::Y * 2.0,
                    color: Color::srgb(1.0, 0.4, 0.1),
                    count: 18,
                    speed: 1.5,
                    lifetime: 2.0,
                    size: 0.1,
                    style: ParticleStyle::FallingEmber,
                });

                // Clean up any panels/poles/texts associated with this exercise
                let exercise_id = eid.0;
                for (e, panel) in &panels {
                    if panel.exercise_id == exercise_id {
                        commands.entity(e).despawn();
                    }
                }
                for (e, pole_eid) in &poles {
                    if pole_eid.0 == exercise_id {
                        commands.entity(e).despawn();
                    }
                }
                for (e, qt) in &question_texts {
                    if qt.exercise_id == exercise_id {
                        commands.entity(e).despawn();
                    }
                }
                for (e, tt) in &timer_texts {
                    if tt.exercise_id == exercise_id {
                        commands.entity(e).despawn();
                    }
                }

                commands.entity(entity).despawn();
            }
        }
    }
}

/// Handle server beacon activated events: when another player activates a beacon,
/// show greyed-out (spectator) panels for that beacon.
#[allow(clippy::too_many_arguments)]
fn handle_beacon_activated(
    mut commands: Commands,
    mut events: MessageReader<BeaconActivatedEvent>,
    mut beacons: Query<(
        Entity,
        &ExerciseId,
        &ServerBeaconId,
        &mut BeaconState,
        &BeaconFacing,
        &Transform,
        &mut super::exercise::ActiveExercise,
    )>,
    beacon_visuals: Query<(Entity, &ChildOf), With<BeaconVisual>>,
    connection_status: Res<ConnectionStatus>,
    terrain: Option<Res<crate::terrain::TerrainResource>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let my_player_id = match &*connection_status {
        ConnectionStatus::Connected { my_player_id, .. } => *my_player_id,
        _ => return,
    };
    let Some(ref terrain) = terrain else {
        return;
    };

    for BeaconActivatedEvent {
        beacon_id,
        activated_by,
    } in events.read()
    {
        // Only handle activations by OTHER players — we already activated locally
        if *activated_by == my_player_id {
            continue;
        }

        for (entity, exercise_id, sbid, mut state, facing, beacon_tf, mut exercise) in &mut beacons {
            if sbid.0 != *beacon_id {
                continue;
            }
            if *state != BeaconState::Dormant {
                continue;
            }

            *state = BeaconState::Activated;
            exercise.state = ExerciseState::Active;

            // Despawn beacon visual children
            for (vis_entity, child_of) in &beacon_visuals {
                if child_of.parent() == entity {
                    commands.entity(vis_entity).despawn();
                }
            }

            // Spawn greyed-out (spectator) panels
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
                false, // not interactive — spectator panels
            );

            // Text rotation
            let to_facing = facing.0.normalize_or_zero();
            let text_rotation = Transform::IDENTITY.looking_to(-to_facing, Vec3::Y).rotation;

            // Spawn question text
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

            // Spawn timer text
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
        }
    }
}

/// When a PendingAnswer is created in multiplayer mode, also send it to the server.
fn send_answer_to_server(
    pending: Option<Res<PendingAnswer>>,
    connection_status: Res<ConnectionStatus>,
    ws: Option<Res<WsConnection>>,
    beacons: Query<(&ExerciseId, &ServerBeaconId)>,
) {
    let Some(answer) = pending else {
        return;
    };
    if !answer.is_changed() {
        return;
    }
    if !connection_status.is_online() {
        return;
    }
    let Some(ws) = ws else {
        return;
    };

    // Find the server beacon_id for this exercise
    for (eid, sbid) in &beacons {
        if eid.0 == answer.exercise_id {
            let msg = ClientMessage::AnswerBeacon {
                beacon_id: sbid.0,
                correct: answer.is_correct,
            };
            ws.send_binary(encode(&msg));
            break;
        }
    }
}
