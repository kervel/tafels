use bevy::prelude::*;
use rand::prelude::*;

use super::VegetationType;
use crate::terrain::heightmap::{self, HeightmapData};

pub struct VegetationInstance {
    pub position: Vec3,
    pub rotation_y: f32,
    pub scale: f32,
    pub vegetation_type: VegetationType,
}

/// Use grid-jitter placement for natural-looking distribution.
/// Divide the terrain into cells and place one item per cell with random offset.
pub fn generate_vegetation_positions(
    heightmap: &HeightmapData,
    density: f32,
    seed: u64,
) -> Vec<VegetationInstance> {
    let mut rng = StdRng::seed_from_u64(seed);
    let half = heightmap.world_size / 2.0;
    let tree_line_height = heightmap.height_scale * 0.75;

    // Grid-jitter: divide terrain into cells, place 0 or 1 item per cell
    let cell_size = 18.0;
    let usable = half * 0.88;
    let grid_count = (2.0 * usable / cell_size) as i32;

    let mut instances = Vec::new();

    for gz in 0..grid_count {
        for gx in 0..grid_count {
            // Random chance to skip this cell (controls density)
            if rng.r#gen::<f32>() > density {
                continue;
            }

            // Cell center + random jitter within cell
            let cx = -usable + (gx as f32 + 0.5) * cell_size;
            let cz = -usable + (gz as f32 + 0.5) * cell_size;
            let jitter = cell_size * 0.45;
            let x = cx + rng.r#gen_range(-jitter..jitter);
            let z = cz + rng.r#gen_range(-jitter..jitter);

            // Skip if outside usable area
            if x.abs() > usable || z.abs() > usable {
                continue;
            }

            let y = heightmap::sample_height(heightmap, x, z);
            let normal = heightmap::sample_normal(heightmap, x, z);
            let height_ratio = y / heightmap.height_scale;

            let roll: f32 = rng.r#gen();

            // Scale notes for Poly Pizza models:
            //   polypizza_pine:  3.6m native height → scale 3-6 for 11-22m trees
            //   pp_spruce2:     10.2m native height → scale 1.2-2.5 for 12-25m trees
            //   pp_conifer2:     9.0m native height → scale 1.3-2.5 for 12-22m trees
            //   rock_07:         ~1m native height  → scale 2-8 for varied rocks
            //   dirt_patch:      ~1m native          → scale 3-5

            let (vegetation_type, scale) = if normal.y < 0.7 {
                // Steep slopes -> rocks only
                if roll < 0.70 {
                    (VegetationType::RockOutcrop, rng.r#gen_range(3.0..8.0))
                } else {
                    (VegetationType::DirtPatch, rng.r#gen_range(3.0..5.0))
                }
            } else if y > tree_line_height {
                // Above tree line -> rocks, no trees
                if roll < 0.65 {
                    (VegetationType::RockOutcrop, rng.r#gen_range(2.0..6.0))
                } else if roll < 0.85 {
                    (VegetationType::Shrub, rng.r#gen_range(1.0..3.0))
                } else {
                    (VegetationType::DirtPatch, rng.r#gen_range(3.0..5.0))
                }
            } else if height_ratio > 0.45 {
                // Mid altitude -> mix of 3 tree types + rocks
                if roll < 0.25 {
                    // Pine (polypizza_pine, 3.6m native)
                    (VegetationType::PineSapling, rng.r#gen_range(3.5..6.0))
                } else if roll < 0.48 {
                    // Spruce (pp_spruce2, 10.2m native)
                    (VegetationType::FirSapling, rng.r#gen_range(1.2..2.2))
                } else if roll < 0.62 {
                    // Conifer (pp_conifer2, 9.0m native)
                    (VegetationType::ProceduralConifer, rng.r#gen_range(1.3..2.2))
                } else if roll < 0.82 {
                    (VegetationType::RockOutcrop, rng.r#gen_range(2.0..6.0))
                } else {
                    (VegetationType::DirtPatch, rng.r#gen_range(3.0..5.0))
                }
            } else {
                // Low valleys -> dense trees, fewer rocks
                if roll < 0.30 {
                    (VegetationType::PineSapling, rng.r#gen_range(4.0..7.0))
                } else if roll < 0.55 {
                    (VegetationType::FirSapling, rng.r#gen_range(1.5..2.5))
                } else if roll < 0.72 {
                    (VegetationType::ProceduralConifer, rng.r#gen_range(1.5..2.5))
                } else if roll < 0.85 {
                    (VegetationType::RockOutcrop, rng.r#gen_range(1.5..4.0))
                } else {
                    (VegetationType::DirtPatch, rng.r#gen_range(3.0..5.0))
                }
            };

            instances.push(VegetationInstance {
                position: Vec3::new(x, y, z),
                rotation_y: rng.r#gen_range(0.0..std::f32::consts::TAU),
                scale,
                vegetation_type,
            });
        }
    }

    instances
}
