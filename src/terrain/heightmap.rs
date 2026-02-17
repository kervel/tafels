use bevy::prelude::*;
use noise::{Fbm, MultiFractal, NoiseFn, Perlin};

pub struct HeightmapData {
    pub heights: Vec<f32>,
    pub width: u32,
    pub depth: u32,
    pub world_size: f32,
    pub height_scale: f32,
}

pub fn generate_heightmap(
    width: u32,
    depth: u32,
    world_size: f32,
    height_scale: f32,
) -> HeightmapData {
    let fbm = Fbm::<Perlin>::new(42)
        .set_octaves(6)
        .set_frequency(0.8)
        .set_lacunarity(2.2)
        .set_persistence(0.45);

    let mut heights = Vec::with_capacity((width * depth) as usize);

    for z in 0..depth {
        for x in 0..width {
            let nx = x as f64 / width as f64;
            let nz = z as f64 / depth as f64;

            let mut h = fbm.get([nx * 3.0, nz * 3.0]) as f32;

            // Add larger-scale hills for alpine foothills feel
            let broad = fbm.get([nx * 0.8, nz * 0.8]) as f32;
            h = h * 0.35 + broad * 0.65;

            // Normalize to 0..1 range (fbm output is roughly -1..1)
            h = (h + 1.0) * 0.5;

            // Push more terrain toward lower elevations (more valleys, fewer peaks)
            h = h * h * (3.0 - 2.0 * h); // smoothstep: keeps 0 and 1 but creates more low areas
            h = h.powf(1.3); // bias toward lower heights

            heights.push(h * height_scale);
        }
    }

    HeightmapData {
        heights,
        width,
        depth,
        world_size,
        height_scale,
    }
}

/// Sample terrain height at world position (x, z) using bilinear interpolation.
pub fn sample_height(data: &HeightmapData, x: f32, z: f32) -> f32 {
    let half = data.world_size / 2.0;
    let u = ((x + half) / data.world_size).clamp(0.0, 1.0);
    let v = ((z + half) / data.world_size).clamp(0.0, 1.0);

    let fx = u * (data.width - 1) as f32;
    let fz = v * (data.depth - 1) as f32;

    let ix = fx.floor() as u32;
    let iz = fz.floor() as u32;
    let ix1 = (ix + 1).min(data.width - 1);
    let iz1 = (iz + 1).min(data.depth - 1);

    let tx = fx.fract();
    let tz = fz.fract();

    let h00 = data.heights[(iz * data.width + ix) as usize];
    let h10 = data.heights[(iz * data.width + ix1) as usize];
    let h01 = data.heights[(iz1 * data.width + ix) as usize];
    let h11 = data.heights[(iz1 * data.width + ix1) as usize];

    let h0 = h00 * (1.0 - tx) + h10 * tx;
    let h1 = h01 * (1.0 - tx) + h11 * tx;

    h0 * (1.0 - tz) + h1 * tz
}

/// Sample terrain surface normal at world position.
pub fn sample_normal(data: &HeightmapData, x: f32, z: f32) -> Vec3 {
    let eps = data.world_size / data.width as f32;
    let hx0 = sample_height(data, x - eps, z);
    let hx1 = sample_height(data, x + eps, z);
    let hz0 = sample_height(data, x, z - eps);
    let hz1 = sample_height(data, x, z + eps);

    Vec3::new(hx0 - hx1, 2.0 * eps, hz0 - hz1).normalize()
}
