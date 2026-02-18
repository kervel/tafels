use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use rand::prelude::*;

use super::exercise::ExerciseId;

pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, _app: &mut App) {
        // Panel spawning/despawning is driven by exercise.rs directly
    }
}

/// Marker for an answer panel entity.
#[derive(Component)]
pub struct AnswerPanel {
    pub value: u32,
    pub is_correct: bool,
    #[allow(dead_code)]
    pub panel_index: u8,
    pub exercise_id: u32,
}

/// Marker for the pole underneath a panel.
#[derive(Component)]
pub struct PanelPole;

/// Marker for panels that belong to another player's activation (non-interactive).
#[derive(Component)]
pub struct SpectatorPanel;

/// Neon color palette for panels.
pub const NEON_COLORS: [[f32; 3]; 4] = [
    [0.2, 0.5, 1.0], // Electric Blue
    [1.0, 0.2, 0.6], // Hot Pink
    [0.2, 1.0, 0.4], // Neon Green
    [1.0, 0.9, 0.1], // Bright Yellow
];

/// 5x7 bitmap font for digits 0-9.
const DIGIT_BITMAPS: [[u8; 7]; 10] = [
    [
        0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
    ], // 0
    [
        0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
    ], // 1
    [
        0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111,
    ], // 2
    [
        0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110,
    ], // 3
    [
        0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
    ], // 4
    [
        0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110,
    ], // 5
    [
        0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
    ], // 6
    [
        0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
    ], // 7
    [
        0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
    ], // 8
    [
        0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
    ], // 9
];

/// Get the 5x7 bitmap for a character. Returns None for unsupported chars.
fn char_bitmap(c: char) -> Option<[u8; 7]> {
    match c {
        '0'..='9' => Some(DIGIT_BITMAPS[(c as u8 - b'0') as usize]),
        'x' => Some([
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ]),
        '/' => Some([
            0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000,
        ]),
        '=' => Some([
            0b00000, 0b11111, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000,
        ]),
        '?' => Some([
            0b01110, 0b10001, 0b00001, 0b00110, 0b00100, 0b00000, 0b00100,
        ]),
        's' => Some([
            0b00000, 0b01111, 0b10000, 0b01110, 0b00001, 0b11110, 0b00000,
        ]),
        '-' => Some([
            0b00000, 0b00000, 0b00000, 0b11111, 0b00000, 0b00000, 0b00000,
        ]),
        ' ' => Some([
            0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000, 0b00000,
        ]),
        _ => None,
    }
}

/// Render a text string into an RGBA texture using the bitmap font.
pub fn render_text_texture(
    images: &mut Assets<Image>,
    text: &str,
    fg: [u8; 3],
    bg: [u8; 3],
) -> Handle<Image> {
    let chars: Vec<[u8; 7]> = text.chars().filter_map(char_bitmap).collect();

    if chars.is_empty() {
        // Return a tiny transparent texture as fallback
        return images.add(Image::new(
            Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            vec![0, 0, 0, 0],
            TextureFormat::Rgba8UnormSrgb,
            default(),
        ));
    }

    let digit_w = 5;
    let digit_h = 7;
    let spacing = 1;
    let scale = 8;
    let padding = 2 * scale;

    let text_w = chars.len() * digit_w + chars.len().saturating_sub(1) * spacing;
    let tex_w = (text_w * scale + padding * 2) as u32;
    let tex_h = (digit_h * scale + padding * 2) as u32;

    let mut data = vec![0u8; (tex_w * tex_h * 4) as usize];

    // Fill background
    for i in 0..(tex_w * tex_h) as usize {
        data[i * 4] = bg[0];
        data[i * 4 + 1] = bg[1];
        data[i * 4 + 2] = bg[2];
        data[i * 4 + 3] = 255;
    }

    // Draw characters
    for (ci, bitmap) in chars.iter().enumerate() {
        let x_off = padding + ci * (digit_w + spacing) * scale;
        let y_off = padding;

        for row in 0..digit_h {
            for col in 0..digit_w {
                if (bitmap[row] >> (digit_w - 1 - col)) & 1 == 1 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = x_off + col * scale + sx;
                            let py = y_off + row * scale + sy;
                            let idx = (py * tex_w as usize + px) * 4;
                            if idx + 3 < data.len() {
                                data[idx] = fg[0];
                                data[idx + 1] = fg[1];
                                data[idx + 2] = fg[2];
                                data[idx + 3] = 255;
                            }
                        }
                    }
                }
            }
        }
    }

    // Flip vertically
    let mut flipped = vec![0u8; data.len()];
    for y in 0..tex_h as usize {
        let src_row = y * tex_w as usize * 4;
        let dst_row = (tex_h as usize - 1 - y) * tex_w as usize * 4;
        flipped[dst_row..dst_row + tex_w as usize * 4]
            .copy_from_slice(&data[src_row..src_row + tex_w as usize * 4]);
    }

    images.add(Image::new(
        Extent3d {
            width: tex_w,
            height: tex_h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        flipped,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    ))
}

/// Render a number into an RGBA texture.
fn render_number_texture(
    images: &mut Assets<Image>,
    number: u32,
    fg: [u8; 3],
    bg: [u8; 3],
) -> Handle<Image> {
    let digits: Vec<usize> = if number == 0 {
        vec![0]
    } else {
        number
            .to_string()
            .chars()
            .map(|c| (c as usize) - ('0' as usize))
            .collect()
    };

    let digit_w = 5;
    let digit_h = 7;
    let spacing = 1;
    let scale = 8;
    let padding = 2 * scale;

    let text_w = digits.len() * digit_w + digits.len().saturating_sub(1) * spacing;
    let tex_w = (text_w * scale + padding * 2) as u32;
    let tex_h = (digit_h * scale + padding * 2) as u32;

    let mut data = vec![0u8; (tex_w * tex_h * 4) as usize];

    // Fill background
    for i in 0..(tex_w * tex_h) as usize {
        data[i * 4] = bg[0];
        data[i * 4 + 1] = bg[1];
        data[i * 4 + 2] = bg[2];
        data[i * 4 + 3] = 255;
    }

    // Draw digits
    for (di, &digit) in digits.iter().enumerate() {
        let x_off = padding + di * (digit_w + spacing) * scale;
        let y_off = padding;
        let bitmap = &DIGIT_BITMAPS[digit];

        for row in 0..digit_h {
            for col in 0..digit_w {
                if (bitmap[row] >> (digit_w - 1 - col)) & 1 == 1 {
                    for sy in 0..scale {
                        for sx in 0..scale {
                            let px = x_off + col * scale + sx;
                            let py = y_off + row * scale + sy;
                            let idx = (py * tex_w as usize + px) * 4;
                            if idx + 3 < data.len() {
                                data[idx] = fg[0];
                                data[idx + 1] = fg[1];
                                data[idx + 2] = fg[2];
                                data[idx + 3] = 255;
                            }
                        }
                    }
                }
            }
        }
    }

    // Flip vertically — cuboid front face UVs have inverted Y
    let mut flipped = vec![0u8; data.len()];
    for y in 0..tex_h as usize {
        let src_row = y * tex_w as usize * 4;
        let dst_row = (tex_h as usize - 1 - y) * tex_w as usize * 4;
        flipped[dst_row..dst_row + tex_w as usize * 4]
            .copy_from_slice(&data[src_row..src_row + tex_w as usize * 4]);
    }

    images.add(Image::new(
        Extent3d {
            width: tex_w,
            height: tex_h,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        flipped,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    ))
}

pub fn spawn_answer_panels(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    heightmap: &crate::terrain::heightmap::HeightmapData,
    center: Vec3,
    facing_toward: Vec3,
    choices: &[u32; 4],
    correct_answer: u32,
    exercise_id: u32,
    interactive: bool,
) {
    let to_player = (facing_toward - center).normalize_or_zero();
    let to_player_xz = Vec3::new(to_player.x, 0.0, to_player.z).normalize_or_zero();
    let right = Vec3::new(-to_player_xz.z, 0.0, to_player_xz.x);

    let forward = to_player_xz;
    let panel_rotation = Transform::IDENTITY.looking_to(-forward, Vec3::Y).rotation;

    let arc_radius = 10.0; // wider spread
    let panel_height = 1.4;
    let pole_top_margin = 0.1;
    let mut rng = rand::thread_rng();

    let pole_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.4, 0.4),
        metallic: 0.8,
        ..default()
    });

    for (i, &value) in choices.iter().enumerate() {
        let t = (i as f32 - 1.5) / 1.5;

        // Base position + random offset
        let offset_x = rng.r#gen_range(-1.5..1.5_f32);
        let offset_z = rng.r#gen_range(-1.5..1.5_f32);
        let x = center.x + right.x * t * arc_radius + offset_x;
        let z = center.z + right.z * t * arc_radius + offset_z;
        let ground_y = crate::terrain::heightmap::sample_height(heightmap, x, z);

        let pole_height = 2.0;
        let panel_center_y = ground_y + pole_height + panel_height * 0.5 + pole_top_margin;
        let panel_pos = Vec3::new(x, panel_center_y, z);

        // Random Y rotation jitter (+-15 degrees) around the base facing direction
        let yaw_jitter = rng.r#gen_range(-0.26..0.26_f32);
        let jittered_rotation = panel_rotation * Quat::from_rotation_y(yaw_jitter);

        let panel_transform = Transform {
            translation: panel_pos,
            rotation: jittered_rotation,
            scale: Vec3::ONE,
        };

        let (bg, base_color, emissive) = if interactive {
            let color = NEON_COLORS[i % 4];
            (
                [
                    (color[0] * 255.0).min(255.0) as u8,
                    (color[1] * 255.0).min(255.0) as u8,
                    (color[2] * 255.0).min(255.0) as u8,
                ],
                Color::WHITE,
                bevy::color::LinearRgba::new(5.0, 5.0, 5.0, 1.0),
            )
        } else {
            (
                [100, 100, 110],
                Color::srgb(0.4, 0.4, 0.45),
                bevy::color::LinearRgba::new(0.3, 0.3, 0.35, 1.0),
            )
        };
        let texture = render_number_texture(images, value, [20, 20, 20], bg);

        let material = materials.add(StandardMaterial {
            base_color,
            base_color_texture: Some(texture.clone()),
            emissive,
            emissive_texture: Some(texture),
            ..default()
        });

        let mesh = meshes.add(Cuboid::new(1.8, panel_height, 0.12));

        // Panel sign with point light child that illuminates the ground
        let neon = NEON_COLORS[i % 4];
        let panel_light_color = if interactive {
            Color::srgb(neon[0], neon[1], neon[2])
        } else {
            Color::srgb(0.3, 0.3, 0.35)
        };
        let light_intensity = if interactive { 600_000.0 } else { 100_000.0 };

        let panel = AnswerPanel {
            value,
            is_correct: value == correct_answer,
            panel_index: i as u8,
            exercise_id,
        };

        let mut entity_commands = commands.spawn((
            Mesh3d(mesh),
            MeshMaterial3d(material),
            panel_transform,
            panel,
        ));
        if !interactive {
            entity_commands.insert(SpectatorPanel);
        }
        entity_commands.with_child((
            PointLight {
                color: panel_light_color,
                intensity: light_intensity,
                range: 20.0,
                shadows_enabled: false,
                ..default()
            },
            Transform::from_translation(Vec3::new(0.0, -0.5, 0.5)),
        ));

        // Pole underneath
        let pole_center_y = ground_y + pole_height * 0.5;
        let pole_mesh = meshes.add(Cylinder::new(0.05, pole_height));
        commands.spawn((
            PanelPole,
            ExerciseId(exercise_id),
            Mesh3d(pole_mesh),
            MeshMaterial3d(pole_material.clone()),
            Transform::from_translation(Vec3::new(x, pole_center_y, z)),
        ));
    }
}

/// Marker for the 3D question text above an exercise.
#[derive(Component)]
pub struct QuestionText {
    pub exercise_id: u32,
}

/// Marker for the 3D timer text above an exercise.
#[derive(Component)]
pub struct TimerText {
    pub exercise_id: u32,
    pub prerendered: Vec<Handle<Image>>,
    pub current_second: u32,
}

/// Spawn a 3D question text mesh above the exercise panels.
#[allow(clippy::too_many_arguments)]
pub fn spawn_question_text(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    text: &str,
    center: Vec3,
    rotation: Quat,
    exercise_id: u32,
) {
    let texture = render_text_texture(images, text, [220, 255, 220], [10, 20, 10]);

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(texture.clone()),
        emissive: bevy::color::LinearRgba::new(4.0, 6.0, 4.0, 1.0),
        emissive_texture: Some(texture),
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });

    // Text quad: wide and tall for readability at distance
    let text_width = text.len() as f32 * 0.8;
    let text_height = 1.3;
    let mesh = meshes.add(Cuboid::new(text_width.max(5.0), text_height, 0.05));

    let pos = Vec3::new(center.x, center.y + 5.0, center.z);
    commands.spawn((
        QuestionText { exercise_id },
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform {
            translation: pos,
            rotation,
            scale: Vec3::ONE,
        },
    ));
}

/// Spawn a 3D timer text mesh below the question text.
#[allow(clippy::too_many_arguments)]
pub fn spawn_timer_text(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    time_limit: f32,
    center: Vec3,
    rotation: Quat,
    exercise_id: u32,
) {
    let max_seconds = time_limit.ceil() as u32;

    // Pre-render timer textures for each second in green
    let mut prerendered = Vec::new();
    for s in 0..=max_seconds {
        let label = format!("{}s", s);
        // Choose color based on fraction of time
        let (fg, bg) = if s as f32 / time_limit > 0.5 {
            ([50, 255, 50], [10, 30, 10]) // green
        } else if s as f32 / time_limit > 0.25 {
            ([255, 255, 50], [30, 30, 10]) // yellow
        } else {
            ([255, 50, 50], [30, 10, 10]) // red
        };
        prerendered.push(render_text_texture(images, &label, fg, bg));
    }

    // Start with the max seconds texture
    let initial_texture = prerendered[max_seconds as usize].clone();

    let material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(initial_texture.clone()),
        emissive: bevy::color::LinearRgba::new(4.0, 6.0, 4.0, 1.0),
        emissive_texture: Some(initial_texture),
        alpha_mode: AlphaMode::Opaque,
        ..default()
    });

    let mesh = meshes.add(Cuboid::new(1.8, 0.6, 0.05));

    let pos = Vec3::new(center.x, center.y + 4.0, center.z);
    commands.spawn((
        TimerText {
            exercise_id,
            prerendered,
            current_second: max_seconds,
        },
        Mesh3d(mesh),
        MeshMaterial3d(material),
        Transform {
            translation: pos,
            rotation,
            scale: Vec3::ONE,
        },
    ));
}
