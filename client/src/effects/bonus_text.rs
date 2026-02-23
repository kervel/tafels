use bevy::prelude::*;

use crate::hud::{AnswerFeedback, SpeedTier};

/// Resource holding pre-loaded digit/symbol scene handles.
#[derive(Resource)]
pub struct DigitAssets {
    /// Maps char to scene handle. Keys: '0'-'9', 'x', '+', '.'
    scenes: Vec<(char, Handle<Scene>)>,
}

impl DigitAssets {
    pub fn get(&self, c: char) -> Option<&Handle<Scene>> {
        self.scenes.iter().find(|(ch, _)| *ch == c).map(|(_, h)| h)
    }
}

/// Load digit GLB assets at startup.
pub fn load_digit_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    let chars_and_files: Vec<(char, &str)> = vec![
        ('0', "models/digits/0.glb"),
        ('1', "models/digits/1.glb"),
        ('2', "models/digits/2.glb"),
        ('3', "models/digits/3.glb"),
        ('4', "models/digits/4.glb"),
        ('5', "models/digits/5.glb"),
        ('6', "models/digits/6.glb"),
        ('7', "models/digits/7.glb"),
        ('8', "models/digits/8.glb"),
        ('9', "models/digits/9.glb"),
        ('x', "models/digits/x.glb"),
        ('+', "models/digits/plus.glb"),
        ('.', "models/digits/dot.glb"),
    ];

    let scenes: Vec<(char, Handle<Scene>)> = chars_and_files
        .into_iter()
        .map(|(c, path)| {
            let handle: Handle<Scene> = asset_server.load(format!("{}#Scene0", path));
            (c, handle)
        })
        .collect();

    commands.insert_resource(DigitAssets { scenes });
}

/// Marker for the root entity of a floating 3D bonus text.
#[derive(Component)]
pub struct BonusText {
    pub age: f32,
    pub max_lifetime: f32,
    pub start_y: f32,
    pub kind: BonusKind,
}

/// Marker for child glyph entities so we can set their material.
#[derive(Component)]
pub struct BonusGlyph;

/// What kind of bonus this text represents (drives emissive color).
#[derive(Clone, Copy, Debug)]
pub enum BonusKind {
    /// Coin reward — colored by speed tier.
    Speed(SpeedTier),
    /// Streak multiplier — always warm orange/red.
    Streak,
}

/// Message to spawn a 3D bonus text at a world position.
#[derive(Message)]
pub struct BonusTextEvent {
    pub position: Vec3,
    pub text: String,
    pub kind: BonusKind,
}

/// Listen for AnswerFeedback and emit BonusTextEvents for correct answers.
pub fn emit_bonus_text_on_feedback(
    feedback: Option<Res<AnswerFeedback>>,
    mut events: MessageWriter<BonusTextEvent>,
) {
    let Some(fb) = feedback else { return };
    if !fb.correct {
        return;
    }

    // Coin reward text — colored by speed tier
    events.write(BonusTextEvent {
        position: fb.hit_position,
        text: format!("+{}", fb.coins_delta),
        kind: BonusKind::Speed(fb.speed_tier),
    });

    // Streak multiplier text — separate, offset upward, warm color
    if fb.combo >= 2 {
        let mult = match fb.combo {
            2 => "x1.5",
            _ => "x2",
        };
        events.write(BonusTextEvent {
            position: fb.hit_position + Vec3::Y * 1.5,
            text: mult.to_string(),
            kind: BonusKind::Streak,
        });
    }
}

/// Spawn 3D bonus text from GLTF digit scenes.
pub fn handle_bonus_text_events(
    mut commands: Commands,
    mut events: MessageReader<BonusTextEvent>,
    digits: Option<Res<DigitAssets>>,
    camera: Query<&Transform, With<Camera3d>>,
) {
    let Some(digits) = digits else { return };
    let cam_tf = camera.single().ok();

    for event in events.read() {
        let pos = event.position + Vec3::Y * 3.0;

        // Face toward camera
        let rotation = if let Some(cam) = cam_tf {
            let dir = (cam.translation - pos).normalize_or_zero();
            Quat::from_rotation_y(dir.x.atan2(dir.z))
        } else {
            Quat::IDENTITY
        };

        // The GLB text objects lie in Blender's XY plane.
        // Rotate +90° around X to stand them upright.
        let stand_up = Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);

        // Layout: place each character side by side (skip spaces)
        let chars: Vec<char> = event.text.chars().filter(|c| *c != ' ').collect();
        let char_spacing = 0.7;
        let total_width = (chars.len() as f32 - 1.0) * char_spacing;
        let start_x = -total_width / 2.0;

        let parent = commands
            .spawn((
                BonusText {
                    age: 0.0,
                    max_lifetime: 2.5,
                    start_y: pos.y,
                    kind: event.kind,
                },
                Transform::from_translation(pos)
                    .with_rotation(rotation)
                    .with_scale(Vec3::ZERO),
                Visibility::default(),
            ))
            .id();

        for (i, c) in chars.iter().enumerate() {
            if let Some(scene) = digits.get(*c) {
                let x_offset = start_x + i as f32 * char_spacing;
                let child = commands
                    .spawn((
                        BonusGlyph,
                        SceneRoot(scene.clone()),
                        Transform::from_translation(Vec3::new(x_offset, 0.0, 0.0))
                            .with_rotation(stand_up),
                    ))
                    .id();
                commands.entity(parent).add_child(child);
            }
        }
    }
}

/// Override materials on bonus glyph children to apply speed-tier emissive colors.
pub fn color_bonus_glyphs(
    glyphs: Query<(&ChildOf, &Children), With<BonusGlyph>>,
    bonus_texts: Query<&BonusText>,
    mesh_entities: Query<&MeshMaterial3d<StandardMaterial>>,
    all_children: Query<&Children>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (child_of, glyph_children) in &glyphs {
        let parent_entity = child_of.parent();
        let Ok(bt) = bonus_texts.get(parent_entity) else {
            continue;
        };
        // Only color during first few frames (scenes load async)
        if bt.age > 0.5 {
            continue;
        }

        let emissive = match bt.kind {
            BonusKind::Speed(SpeedTier::Lightning) => {
                bevy::color::LinearRgba::new(8.0, 30.0, 35.0, 1.0) // cyan
            }
            BonusKind::Speed(SpeedTier::Fast) => {
                bevy::color::LinearRgba::new(30.0, 25.0, 5.0, 1.0) // gold
            }
            BonusKind::Speed(SpeedTier::Normal) => {
                bevy::color::LinearRgba::new(5.0, 25.0, 8.0, 1.0) // green
            }
            BonusKind::Streak => {
                bevy::color::LinearRgba::new(35.0, 12.0, 3.0, 1.0) // hot orange
            }
        };

        for child in glyph_children.iter() {
            color_descendants(child, &all_children, &mesh_entities, &mut materials, emissive);
        }
    }
}

fn color_descendants(
    entity: Entity,
    all_children: &Query<&Children>,
    mesh_entities: &Query<&MeshMaterial3d<StandardMaterial>>,
    materials: &mut Assets<StandardMaterial>,
    emissive: bevy::color::LinearRgba,
) {
    if let Ok(mat_handle) = mesh_entities.get(entity) {
        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.base_color = Color::WHITE;
            mat.emissive = emissive;
        }
    }
    if let Ok(children) = all_children.get(entity) {
        for child in children.iter() {
            color_descendants(child, all_children, mesh_entities, materials, emissive);
        }
    }
}

/// Animate bonus text: elastic bounce-in, float up, wobble, shrink out.
pub fn update_bonus_text(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut BonusText, &mut Transform)>,
) {
    let dt = time.delta_secs();

    for (entity, mut bt, mut tf) in &mut query {
        bt.age += dt;
        let t = bt.age / bt.max_lifetime;

        if t >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }

        // Elastic bounce-in for first 0.4s, then hold, then shrink out
        let scale = if bt.age < 0.4 {
            let p = bt.age / 0.4;
            elastic_ease_out(p)
        } else if t < 0.7 {
            1.0
        } else {
            let shrink_t = (t - 0.7) / 0.3;
            1.0 - shrink_t * shrink_t
        };

        tf.scale = Vec3::splat(scale.max(0.01));

        // Float upward
        tf.translation.y = bt.start_y + bt.age * 1.5;

        // Gentle wobble
        let wobble = (bt.age * 3.0).sin() * 0.05;
        let base_yaw = tf.rotation.to_euler(EulerRot::YXZ).0;
        tf.rotation = Quat::from_rotation_y(base_yaw) * Quat::from_rotation_z(wobble);
    }
}

/// Billboard bonus text toward the camera each frame.
pub fn billboard_bonus_text(
    camera: Query<&Transform, (With<Camera3d>, Without<BonusText>)>,
    mut texts: Query<&mut Transform, With<BonusText>>,
) {
    let Ok(cam_tf) = camera.single() else { return };
    for mut tf in &mut texts {
        let dir = (cam_tf.translation - tf.translation).normalize_or_zero();
        let yaw = dir.x.atan2(dir.z);
        let current_scale = tf.scale;
        let wobble_z = tf.rotation.to_euler(EulerRot::YXZ).2;
        tf.rotation = Quat::from_rotation_y(yaw) * Quat::from_rotation_z(wobble_z);
        tf.scale = current_scale;
    }
}

/// Elastic ease-out: overshoot then settle.
fn elastic_ease_out(t: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let p = 0.3_f32;
    let s = p / 4.0;
    2.0_f32.powf(-10.0 * t) * ((t - s) * std::f32::consts::TAU / p).sin() + 1.0
}
