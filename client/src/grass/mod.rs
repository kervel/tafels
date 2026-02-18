use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;
use rand::prelude::*;

use crate::character::CharacterMarker;
use crate::terrain;
use crate::terrain::heightmap;

pub struct GrassPlugin;

impl Plugin for GrassPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_grass.after(terrain::setup_terrain))
            .add_systems(Update, (animate_wind, cull_grass));
    }
}

#[derive(Component)]
pub struct GrassTuft;

/// Attach to any entity that should sway in the wind (grass, trees, shrubs).
#[derive(Component)]
pub struct WindSway {
    pub phase: f32,
    pub base_rotation: Quat,
    pub strength: f32,
}

const GRASS_CULL_DISTANCE: f32 = 30.0;
const GRASS_CULL_DISTANCE_SQ: f32 = GRASS_CULL_DISTANCE * GRASS_CULL_DISTANCE;

/// Create a grass clump mesh with the given number of blades.
/// Each blade is a slender rectangle that tapers to a pointed tip.
fn create_grass_mesh(blade_count: usize, rng: &mut StdRng) -> Mesh {
    let base_color: [f32; 4] = [0.14, 0.32, 0.07, 1.0];
    let mid_color: [f32; 4] = [0.22, 0.42, 0.10, 1.0];
    let tip_color: [f32; 4] = [0.30, 0.50, 0.12, 1.0];

    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut idx = Vec::new();

    let spread = 0.06; // how far blades spread from center

    for i in 0..blade_count {
        let rot = rng.r#gen_range(0.0..std::f32::consts::TAU);
        let ox = rng.r#gen_range(-spread..spread);
        let oz = rng.r#gen_range(-spread..spread);
        let h = rng.r#gen_range(0.25..0.50);
        let w = 0.008; // half-width — slender blade
        let mid_y = h * 0.55;
        let cos_r = rot.cos();
        let sin_r = rot.sin();
        let bi = (i * 5) as u32;

        let rx = |x: f32, z: f32, y: f32| -> [f32; 3] {
            [ox + x * cos_r - z * sin_r, y, oz + x * sin_r + z * cos_r]
        };

        positions.push(rx(-w, 0.0, 0.0));
        positions.push(rx(w, 0.0, 0.0));
        positions.push(rx(w, 0.0, mid_y));
        positions.push(rx(-w, 0.0, mid_y));
        positions.push(rx(0.0, 0.0, h));

        let n = Vec3::new(sin_r, 0.2, cos_r).normalize();
        for _ in 0..5 {
            normals.push([n.x, n.y, n.z]);
        }

        colors.extend_from_slice(&[base_color, base_color, mid_color, mid_color, tip_color]);

        idx.extend_from_slice(&[
            bi,
            bi + 1,
            bi + 2,
            bi,
            bi + 2,
            bi + 3,
            bi + 3,
            bi + 2,
            bi + 4,
        ]);
    }

    Mesh::new(PrimitiveTopology::TriangleList, default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
        .with_inserted_indices(Indices::U32(idx))
}

/// Pre-generate several mesh variants with different blade counts for visual variety.
fn create_grass_variants(meshes: &mut Assets<Mesh>) -> Vec<Handle<Mesh>> {
    let blade_counts = [4, 5, 6, 7, 8, 8, 9, 10];
    blade_counts
        .iter()
        .enumerate()
        .map(|(i, &count)| {
            let mut rng = StdRng::seed_from_u64(31337 + i as u64 * 7919);
            meshes.add(create_grass_mesh(count, &mut rng))
        })
        .collect()
}

fn spawn_grass(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    terrain_res: Option<Res<terrain::TerrainResource>>,
) {
    let Some(terrain) = terrain_res else { return };

    let variants = create_grass_variants(&mut meshes);
    let grass_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        double_sided: true,
        cull_mode: None,
        perceptual_roughness: 0.9,
        ..default()
    });

    let mut rng = StdRng::seed_from_u64(77777);
    let half = terrain.heightmap.world_size / 2.0;
    let tree_line = terrain.heightmap.height_scale * 0.75;
    let cell_size = 2.0;
    let usable = half * 0.85;
    let density = 0.70;
    let grid_count = (2.0 * usable / cell_size) as i32;

    for gz in 0..grid_count {
        for gx in 0..grid_count {
            if rng.r#gen::<f32>() > density {
                continue;
            }

            let cx = -usable + (gx as f32 + 0.5) * cell_size;
            let cz = -usable + (gz as f32 + 0.5) * cell_size;
            let jitter = cell_size * 0.45;
            let x = cx + rng.r#gen_range(-jitter..jitter);
            let z = cz + rng.r#gen_range(-jitter..jitter);

            if x.abs() > usable || z.abs() > usable {
                continue;
            }

            let y = heightmap::sample_height(&terrain.heightmap, x, z);
            let normal = heightmap::sample_normal(&terrain.heightmap, x, z);

            if normal.y < 0.75 || y > tree_line {
                continue;
            }

            let variant_idx = rng.r#gen_range(0..variants.len());
            let scale_xz = rng.r#gen_range(0.6..2.0);
            let scale_y = rng.r#gen_range(0.5..2.5);
            let yaw = rng.r#gen_range(0.0..std::f32::consts::TAU);
            let base_rotation = Quat::from_rotation_y(yaw);

            commands.spawn((
                Mesh3d(variants[variant_idx].clone()),
                MeshMaterial3d(grass_material.clone()),
                Transform::from_translation(Vec3::new(x, y, z))
                    .with_rotation(base_rotation)
                    .with_scale(Vec3::new(scale_xz, scale_y, scale_xz)),
                GrassTuft,
                WindSway {
                    phase: rng.r#gen_range(0.0..std::f32::consts::TAU),
                    base_rotation,
                    strength: 0.12,
                },
            ));
        }
    }
}

/// Sway all WindSway entities (grass + trees) with a rolling wave pattern.
pub fn animate_wind(time: Res<Time>, mut query: Query<(&mut Transform, &WindSway, &Visibility)>) {
    let t = time.elapsed_secs();

    for (mut transform, sway, vis) in &mut query {
        if *vis == Visibility::Hidden {
            continue;
        }

        let pos = transform.translation;

        let wave_x = (t * 2.0 + sway.phase + pos.x * 0.25 + pos.z * 0.15).sin();
        let wave_z = (t * 1.4 + sway.phase * 1.3 + pos.x * 0.15 - pos.z * 0.1).cos();

        let wind_rot = Quat::from_euler(
            EulerRot::XZY,
            wave_x * sway.strength,
            0.0,
            wave_z * sway.strength * 0.6,
        );

        transform.rotation = sway.base_rotation * wind_rot;
    }
}

fn cull_grass(
    character_query: Query<&Transform, With<CharacterMarker>>,
    mut grass_query: Query<
        (&Transform, &mut Visibility),
        (With<GrassTuft>, Without<CharacterMarker>),
    >,
) {
    let Some(char_tf) = character_query.iter().next() else {
        return;
    };
    let char_pos = char_tf.translation;

    for (grass_tf, mut vis) in &mut grass_query {
        let dx = grass_tf.translation.x - char_pos.x;
        let dz = grass_tf.translation.z - char_pos.z;
        let dist_sq = dx * dx + dz * dz;

        if dist_sq > GRASS_CULL_DISTANCE_SQ {
            *vis = Visibility::Hidden;
        } else {
            *vis = Visibility::Inherited;
        }
    }
}
