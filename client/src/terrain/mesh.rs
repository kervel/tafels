use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::prelude::*;

use super::heightmap::HeightmapData;

fn terrain_color(height_ratio: f32, slope: f32) -> [f32; 4] {
    // slope = normal.y: 1.0 = flat, 0.0 = vertical cliff
    let grass = [0.22, 0.45, 0.12, 1.0];
    let dark_grass = [0.15, 0.32, 0.08, 1.0];
    let dirt = [0.42, 0.32, 0.18, 1.0];
    let rock = [0.48, 0.46, 0.42, 1.0];
    let snow = [0.88, 0.90, 0.92, 1.0];

    // Rocky on steep slopes
    if slope < 0.7 {
        return lerp_color(&rock, &dirt, (slope - 0.5).max(0.0) / 0.2);
    }

    // Height-based coloring for flat areas
    if height_ratio > 0.85 {
        let t = (height_ratio - 0.85) / 0.15;
        return lerp_color(&rock, &snow, t);
    }
    if height_ratio > 0.6 {
        let t = (height_ratio - 0.6) / 0.25;
        return lerp_color(&dirt, &rock, t);
    }
    if height_ratio > 0.35 {
        let t = (height_ratio - 0.35) / 0.25;
        return lerp_color(&dark_grass, &dirt, t);
    }

    // Low flat areas: green grass with some variation
    let t = height_ratio / 0.35;
    lerp_color(&grass, &dark_grass, t)
}

fn lerp_color(a: &[f32; 4], b: &[f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        1.0,
    ]
}

pub fn generate_terrain_mesh(heightmap: &HeightmapData) -> Mesh {
    let w = heightmap.width as usize;
    let d = heightmap.depth as usize;
    let half = heightmap.world_size / 2.0;
    let step_x = heightmap.world_size / (w - 1) as f32;
    let step_z = heightmap.world_size / (d - 1) as f32;

    let vertex_count = w * d;
    let mut positions = Vec::with_capacity(vertex_count);
    let mut normals = Vec::with_capacity(vertex_count);
    let mut uvs = Vec::with_capacity(vertex_count);

    for z in 0..d {
        for x in 0..w {
            let px = x as f32 * step_x - half;
            let pz = z as f32 * step_z - half;
            let py = heightmap.heights[z * w + x];
            positions.push([px, py, pz]);
            uvs.push([x as f32 * 0.5, z as f32 * 0.5]);
        }
    }

    // Calculate normals from heightmap gradients
    let mut normal_vecs = Vec::with_capacity(vertex_count);
    for z in 0..d {
        for x in 0..w {
            let h_left = if x > 0 {
                heightmap.heights[z * w + x - 1]
            } else {
                heightmap.heights[z * w + x]
            };
            let h_right = if x < w - 1 {
                heightmap.heights[z * w + x + 1]
            } else {
                heightmap.heights[z * w + x]
            };
            let h_down = if z > 0 {
                heightmap.heights[(z - 1) * w + x]
            } else {
                heightmap.heights[z * w + x]
            };
            let h_up = if z < d - 1 {
                heightmap.heights[(z + 1) * w + x]
            } else {
                heightmap.heights[z * w + x]
            };

            let normal = Vec3::new(h_left - h_right, 2.0 * step_x, h_down - h_up).normalize();
            normal_vecs.push(normal);
            normals.push([normal.x, normal.y, normal.z]);
        }
    }

    // Vertex colors based on height and slope
    let max_h = heightmap.height_scale;
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(vertex_count);
    for z in 0..d {
        for x in 0..w {
            let idx = z * w + x;
            let h = heightmap.heights[idx];
            let height_ratio = (h / max_h).clamp(0.0, 1.0);
            let slope = normal_vecs[idx].y;
            colors.push(terrain_color(height_ratio, slope));
        }
    }

    // Generate triangle indices
    let quad_count = (w - 1) * (d - 1);
    let mut indices = Vec::with_capacity(quad_count * 6);

    for z in 0..(d - 1) {
        for x in 0..(w - 1) {
            let tl = (z * w + x) as u32;
            let tr = tl + 1;
            let bl = ((z + 1) * w + x) as u32;
            let br = bl + 1;

            indices.push(tl);
            indices.push(bl);
            indices.push(tr);

            indices.push(tr);
            indices.push(bl);
            indices.push(br);
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
