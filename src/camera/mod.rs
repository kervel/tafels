pub mod orbit;

use bevy::prelude::*;
use bevy::core_pipeline::Skybox;
use bevy::post_process::bloom::Bloom;
use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    orbit::camera_mouse_input,
                    orbit::camera_scroll_zoom,
                    orbit::camera_follow,
                )
                    .chain(),
            );
    }
}

fn spawn_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    terrain: Option<Res<crate::terrain::TerrainResource>>,
) {
    let center_height = terrain
        .as_ref()
        .map(|t| crate::terrain::heightmap::sample_height(&t.heightmap, 0.0, 0.0))
        .unwrap_or(30.0);

    let pitch = 0.30_f32;
    let distance = 28.0_f32;
    let start_y = center_height + 1.5 + distance * pitch.sin();
    let start_z = distance * pitch.cos();

    // Generate a simple gradient skybox cubemap
    let skybox_handle = images.add(generate_sky_cubemap());

    commands.spawn((
        Camera3d::default(),
        Bloom {
            intensity: 0.5,
            low_frequency_boost: 0.6,
            low_frequency_boost_curvature: 0.5,
            ..Bloom::NATURAL
        },
        Transform::from_translation(Vec3::new(0.0, start_y, start_z))
            .looking_at(Vec3::new(0.0, center_height + 1.5, 0.0), Vec3::Y),
        Skybox {
            image: skybox_handle,
            brightness: 500.0,
            rotation: Quat::IDENTITY,
        },
        AmbientLight {
            color: Color::srgb(0.5, 0.55, 0.7),
            brightness: 300.0,
            affects_lightmapped_meshes: true,
        },
        DistanceFog {
            color: Color::srgba(0.50, 0.55, 0.65, 1.0),
            directional_light_color: Color::srgba(1.0, 0.85, 0.65, 0.5),
            directional_light_exponent: 15.0,
            falloff: FogFalloff::Exponential { density: 0.012 },
        },
        orbit::OrbitCamera {
            distance,
            yaw: 0.0,
            pitch,
            target: Entity::PLACEHOLDER,
            auto_follow: true,
            return_timer: 0.0,
            last_move_yaw: 0.0,
        },
    ));
}

/// Generate a simple gradient sky cubemap (6 faces, 64x64 each)
fn generate_sky_cubemap() -> Image {
    let size = 64;
    let mut data = Vec::with_capacity(size * size * 4 * 6);

    // Sky colors - dusky twilight for neon contrast
    let zenith = [0.08_f32, 0.12, 0.35]; // dark indigo
    let horizon = [0.25_f32, 0.28, 0.40]; // muted blue-grey
    let sun_side = [0.40_f32, 0.30, 0.25]; // warm sunset remnant

    for face in 0..6 {
        for y in 0..size {
            for x in 0..size {
                // Normalized coords [-1, 1]
                let u = (x as f32 + 0.5) / size as f32 * 2.0 - 1.0;
                let v = (y as f32 + 0.5) / size as f32 * 2.0 - 1.0;

                // Direction vector for this face
                let dir = match face {
                    0 => Vec3::new(1.0, -v, -u),  // +X
                    1 => Vec3::new(-1.0, -v, u),   // -X
                    2 => Vec3::new(u, 1.0, v),     // +Y (top)
                    3 => Vec3::new(u, -1.0, -v),   // -Y (bottom)
                    4 => Vec3::new(u, -v, 1.0),    // +Z
                    _ => Vec3::new(-u, -v, -1.0),  // -Z
                };
                let dir = dir.normalize();

                // Vertical angle: 1 = straight up, -1 = straight down
                let up_factor = dir.y.clamp(-1.0, 1.0);

                let (r, g, b) = if up_factor > 0.0 {
                    // Sky: blend from horizon to zenith
                    let t = up_factor.powf(0.6);
                    (
                        horizon[0] + (zenith[0] - horizon[0]) * t,
                        horizon[1] + (zenith[1] - horizon[1]) * t,
                        horizon[2] + (zenith[2] - horizon[2]) * t,
                    )
                } else {
                    // Below horizon: subtle ground reflection tint
                    let t = (-up_factor).powf(0.5).min(1.0);
                    (
                        horizon[0] + (sun_side[0] - horizon[0]) * t * 0.3,
                        horizon[1] + (sun_side[1] - horizon[1]) * t * 0.3,
                        horizon[2] + (sun_side[2] - horizon[2]) * t * 0.3,
                    )
                };

                data.push((r * 255.0) as u8);
                data.push((g * 255.0) as u8);
                data.push((b * 255.0) as u8);
                data.push(255u8);
            }
        }
    }

    let mut image = Image::new(
        Extent3d {
            width: size as u32,
            height: size as u32,
            depth_or_array_layers: 6,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        default(),
    );

    // Mark this as a Cube texture so the skybox shader reads it correctly
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });

    image
}
