pub use tafels_shared::heightmap::{HeightmapData, generate_heightmap, sample_height};

use bevy::prelude::*;

/// Sample terrain surface normal at world position.
/// This wraps the shared version but returns a Bevy Vec3.
pub fn sample_normal(data: &HeightmapData, x: f32, z: f32) -> Vec3 {
    let n = tafels_shared::heightmap::sample_normal(data, x, z);
    Vec3::new(n.x, n.y, n.z)
}
