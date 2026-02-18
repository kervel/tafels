use bevy::prelude::*;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_lighting);
    }
}

fn setup_lighting(mut commands: Commands) {
    // Directional sun - moderate late afternoon
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.9, 0.75),
            illuminance: 40_000.0,
            shadows_enabled: !cfg!(target_arch = "wasm32"),
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -0.35, // low-ish angle
            std::f32::consts::FRAC_PI_6,
            0.0,
        )),
    ));
}
