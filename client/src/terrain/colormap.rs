use bevy::image::Image;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::heightmap::HeightmapData;

/// Generate a terrain color texture with noise-based variation.
/// Uses height, slope, AND spatial noise to create natural-looking patches
/// of different ground cover (grass, dirt, rock, etc.)
pub fn generate_terrain_texture(heightmap: &HeightmapData) -> Image {
    let w = heightmap.width;
    let d = heightmap.depth;
    let step = heightmap.world_size / (w - 1) as f32;

    let mut data = Vec::with_capacity((w * d * 4) as usize);

    for z in 0..d {
        for x in 0..w {
            let idx = (z * w + x) as usize;
            let h = heightmap.heights[idx];
            let height_ratio = (h / heightmap.height_scale).clamp(0.0, 1.0);

            let slope = compute_slope(heightmap, x, z, step);

            // Spatial noise for ground variation (using simple hash-based noise)
            let nx = x as f32 / w as f32;
            let nz = z as f32 / d as f32;
            let patch_noise = pseudo_noise(nx * 8.0, nz * 8.0);
            let detail_noise = pseudo_noise(nx * 25.0 + 7.3, nz * 25.0 + 3.1);
            let micro_noise = pseudo_noise(nx * 60.0 + 13.7, nz * 60.0 + 17.9);

            let (r, g, b) =
                terrain_color(height_ratio, slope, patch_noise, detail_noise, micro_noise);

            data.push((r * 255.0) as u8);
            data.push((g * 255.0) as u8);
            data.push((b * 255.0) as u8);
            data.push(255u8);
        }
    }

    Image::new(
        Extent3d {
            width: w,
            height: d,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    )
}

/// Simple pseudo-noise function (no external crate needed for texture gen)
fn pseudo_noise(x: f32, y: f32) -> f32 {
    let ix = x.floor() as i32;
    let iy = y.floor() as i32;
    let fx = x - x.floor();
    let fy = y - y.floor();

    // Smoothstep
    let sx = fx * fx * (3.0 - 2.0 * fx);
    let sy = fy * fy * (3.0 - 2.0 * fy);

    let h00 = hash2d(ix, iy);
    let h10 = hash2d(ix + 1, iy);
    let h01 = hash2d(ix, iy + 1);
    let h11 = hash2d(ix + 1, iy + 1);

    let a = h00 + (h10 - h00) * sx;
    let b = h01 + (h11 - h01) * sx;
    a + (b - a) * sy
}

fn hash2d(x: i32, y: i32) -> f32 {
    let n = (x
        .wrapping_mul(374761393)
        .wrapping_add(y.wrapping_mul(668265263))) as u32;
    let n = n ^ (n >> 13);
    let n = n.wrapping_mul(1274126177);
    let n = n ^ (n >> 16);
    (n & 0xFFFF) as f32 / 65535.0
}

fn compute_slope(heightmap: &HeightmapData, x: u32, z: u32, step: f32) -> f32 {
    let w = heightmap.width;
    let d = heightmap.depth;
    let idx = |x: u32, z: u32| (z * w + x) as usize;

    let h_center = heightmap.heights[idx(x, z)];
    let h_left = if x > 0 {
        heightmap.heights[idx(x - 1, z)]
    } else {
        h_center
    };
    let h_right = if x < w - 1 {
        heightmap.heights[idx(x + 1, z)]
    } else {
        h_center
    };
    let h_down = if z > 0 {
        heightmap.heights[idx(x, z - 1)]
    } else {
        h_center
    };
    let h_up = if z < d - 1 {
        heightmap.heights[idx(x, z + 1)]
    } else {
        h_center
    };

    Vec3::new(h_left - h_right, 2.0 * step, h_down - h_up)
        .normalize()
        .y
}

fn terrain_color(h: f32, slope: f32, patch: f32, detail: f32, micro: f32) -> (f32, f32, f32) {
    // Rich color palette
    let grass_lush = (0.25, 0.52, 0.10);
    let grass_yellow = (0.40, 0.50, 0.12);
    let grass_dark = (0.16, 0.38, 0.08);
    let meadow = (0.30, 0.48, 0.15);
    let dirt_light = (0.48, 0.38, 0.24);
    let dirt_dark = (0.38, 0.28, 0.16);
    let rock_warm = (0.42, 0.38, 0.32);
    let rock_gray = (0.50, 0.48, 0.44);
    let snow = (0.85, 0.87, 0.90);

    // Very steep → rock (with some color variation)
    if slope < 0.55 {
        let rock = lerp3(&rock_gray, &rock_warm, detail);
        return add_micro(&rock, micro);
    }
    if slope < 0.72 {
        let t = (slope - 0.55) / 0.17;
        let rock = lerp3(&rock_gray, &rock_warm, detail);
        let ground = height_ground_color(
            h,
            patch,
            &grass_lush,
            &grass_dark,
            &meadow,
            &grass_yellow,
            &dirt_light,
            &dirt_dark,
        );
        let base = lerp3(&rock, &ground, t);
        return add_micro(&base, micro);
    }

    // Moderate slopes get dirt mixed in
    let ground = height_ground_color(
        h,
        patch,
        &grass_lush,
        &grass_dark,
        &meadow,
        &grass_yellow,
        &dirt_light,
        &dirt_dark,
    );
    let base = if slope < 0.85 {
        let t = (slope - 0.72) / 0.13;
        let dirty = lerp3(&dirt_light, &ground, 0.5);
        lerp3(&dirty, &ground, t)
    } else {
        ground
    };

    // High altitude override
    if h > 0.90 {
        let t = (h - 0.90) / 0.10;
        let high = lerp3(&rock_gray, &snow, t);
        return add_micro(&high, micro);
    }
    if h > 0.78 {
        let t = (h - 0.78) / 0.12;
        let high = lerp3(&base, &rock_gray, t);
        return add_micro(&high, micro);
    }

    add_micro(&base, micro)
}

/// Ground color based on height with noise-driven patches
fn height_ground_color(
    h: f32,
    patch: f32,
    lush: &(f32, f32, f32),
    dark: &(f32, f32, f32),
    meadow: &(f32, f32, f32),
    yellow: &(f32, f32, f32),
    dirt_l: &(f32, f32, f32),
    dirt_d: &(f32, f32, f32),
) -> (f32, f32, f32) {
    // Use patch noise to create distinct zones of grass/meadow/dirt
    // regardless of height - this is the key to varied terrain

    if h > 0.65 {
        // Higher areas: mix of yellowed grass and dirt
        let t = (h - 0.65) / 0.15;
        if patch < 0.35 {
            lerp3(yellow, dirt_l, t)
        } else if patch < 0.65 {
            lerp3(dark, dirt_d, t)
        } else {
            lerp3(meadow, dirt_l, t * 0.7)
        }
    } else if h > 0.35 {
        // Mid areas: varied grass types with occasional dirt patches
        if patch < 0.15 {
            // Bare dirt patches
            lerp3(dirt_l, dirt_d, h)
        } else if patch < 0.40 {
            // Dark dense grass
            *dark
        } else if patch < 0.55 {
            // Yellow-green meadow grass
            lerp3(yellow, meadow, (h - 0.35) / 0.30)
        } else if patch < 0.80 {
            // Lush green grass
            *lush
        } else {
            // Mixed meadow
            *meadow
        }
    } else {
        // Low valley: lush greens with some variation
        if patch < 0.10 {
            lerp3(dirt_l, dirt_d, 0.5) // occasional dirt
        } else if patch < 0.35 {
            *dark
        } else if patch < 0.70 {
            *lush
        } else {
            *meadow
        }
    }
}

fn lerp3(a: &(f32, f32, f32), b: &(f32, f32, f32), t: f32) -> (f32, f32, f32) {
    (
        a.0 + (b.0 - a.0) * t,
        a.1 + (b.1 - a.1) * t,
        a.2 + (b.2 - a.2) * t,
    )
}

fn add_micro(color: &(f32, f32, f32), noise: f32) -> (f32, f32, f32) {
    // Subtle brightness variation from micro noise
    let v = 0.92 + 0.16 * noise;
    (
        (color.0 * v).clamp(0.0, 1.0),
        (color.1 * v).clamp(0.0, 1.0),
        (color.2 * v).clamp(0.0, 1.0),
    )
}
