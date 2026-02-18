pub mod colormap;
pub mod heightmap;
pub mod mesh;

use bevy::prelude::*;
use heightmap::HeightmapData;

pub struct TerrainPlugin;

impl Plugin for TerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_terrain);
    }
}

#[derive(Resource)]
pub struct TerrainResource {
    pub heightmap: HeightmapData,
    pub world_size: f32,
}

pub fn setup_terrain(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let resolution = 256;
    let world_size = 500.0;
    let height_scale = 60.0;

    let heightmap = heightmap::generate_heightmap(resolution, resolution, world_size, height_scale);
    let terrain_mesh = mesh::generate_terrain_mesh(&heightmap);
    let color_texture = colormap::generate_terrain_texture(&heightmap);

    let texture_handle = images.add(color_texture);

    commands.spawn((
        Mesh3d(meshes.add(terrain_mesh)),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: Some(texture_handle),
            perceptual_roughness: 0.85,
            ..default()
        })),
        Transform::from_translation(Vec3::ZERO),
    ));

    commands.insert_resource(TerrainResource {
        heightmap,
        world_size,
    });
}
