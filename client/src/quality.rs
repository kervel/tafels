use std::collections::VecDeque;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

pub struct QualityPlugin;

impl Plugin for QualityPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(QualitySettings::default())
            .add_systems(Update, (update_quality_settings, apply_resolution_scale));
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityLevel {
    High,
    Low,
}

#[derive(Resource)]
pub struct QualitySettings {
    pub level: QualityLevel,
    /// Ring buffer of FPS samples (one per frame, last ~5 seconds).
    fps_samples: VecDeque<f32>,
    /// Timer for periodic evaluation (every 1 second).
    eval_timer: f32,
    /// Consecutive evaluations below the low threshold.
    consecutive_low: u32,
    /// Consecutive evaluations above the high threshold.
    consecutive_high: u32,
}

impl Default for QualitySettings {
    fn default() -> Self {
        Self {
            level: QualityLevel::High,
            fps_samples: VecDeque::with_capacity(300), // ~5s at 60fps
            eval_timer: 0.0,
            consecutive_low: 0,
            consecutive_high: 0,
        }
    }
}

const FPS_LOW_THRESHOLD: f32 = 25.0;
const FPS_HIGH_THRESHOLD: f32 = 45.0;
const EVAL_INTERVAL: f32 = 1.0;
const CONSECUTIVE_REQUIRED: u32 = 2;
/// Frames with delta above this are ignored (app switch / tab hidden).
const MAX_FRAME_DELTA: f32 = 0.5;
/// Keep at most this many seconds of samples.
const SAMPLE_WINDOW: f32 = 5.0;

fn update_quality_settings(time: Res<Time>, mut settings: ResMut<QualitySettings>) {
    let dt = time.delta_secs();

    // Skip frames with very large deltas (app switch, tab hidden).
    if dt > MAX_FRAME_DELTA || dt <= 0.0 {
        return;
    }

    let fps = 1.0 / dt;
    settings.fps_samples.push_back(fps);

    // Trim old samples: keep roughly SAMPLE_WINDOW seconds worth.
    // Approximate by capping the buffer size based on current FPS.
    let max_samples = (fps * SAMPLE_WINDOW) as usize;
    let max_samples = max_samples.clamp(30, 600);
    while settings.fps_samples.len() > max_samples {
        settings.fps_samples.pop_front();
    }

    settings.eval_timer += dt;
    if settings.eval_timer < EVAL_INTERVAL {
        return;
    }
    settings.eval_timer = 0.0;

    if settings.fps_samples.len() < 10 {
        return;
    }

    let avg_fps: f32 =
        settings.fps_samples.iter().sum::<f32>() / settings.fps_samples.len() as f32;

    if avg_fps < FPS_LOW_THRESHOLD {
        settings.consecutive_low += 1;
        settings.consecutive_high = 0;
    } else if avg_fps > FPS_HIGH_THRESHOLD {
        settings.consecutive_high += 1;
        settings.consecutive_low = 0;
    } else {
        settings.consecutive_low = 0;
        settings.consecutive_high = 0;
    }

    if settings.consecutive_low >= CONSECUTIVE_REQUIRED && settings.level == QualityLevel::High {
        info!("Adaptive quality: switching to Low (avg FPS: {avg_fps:.0})");
        settings.level = QualityLevel::Low;
        settings.consecutive_low = 0;
    } else if settings.consecutive_high >= CONSECUTIVE_REQUIRED
        && settings.level == QualityLevel::Low
    {
        info!("Adaptive quality: switching to High (avg FPS: {avg_fps:.0})");
        settings.level = QualityLevel::High;
        settings.consecutive_high = 0;
    }
}

/// On Low quality, reduce render resolution by bumping the scale factor override down.
/// This makes the GPU render fewer physical pixels for the same window size.
fn apply_resolution_scale(
    quality: Res<QualitySettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    mut last_level: Local<Option<QualityLevel>>,
) {
    if *last_level == Some(quality.level) {
        return;
    }
    *last_level = Some(quality.level);
    let Ok(mut window) = windows.single_mut() else {
        return;
    };
    // A lower scale_factor_override means Bevy thinks each logical pixel is fewer
    // physical pixels, so the render target shrinks while the window stays the same size.
    let scale = match quality.level {
        QualityLevel::High => None,      // native OS scale
        QualityLevel::Low => Some(0.5),  // render at ~25% pixel count on most displays
    };
    window.resolution.set_scale_factor_override(scale);
    info!("Render scale override: {:?}", scale);
}
