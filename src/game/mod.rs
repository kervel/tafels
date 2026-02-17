pub mod difficulty;
pub mod exercise;
pub mod panels;
pub mod scoring;

use bevy::prelude::*;

use difficulty::Difficulty;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(GameSession::default())
            .add_plugins(exercise::ExercisePlugin)
            .add_plugins(panels::PanelPlugin)
            .add_plugins(scoring::ScoringPlugin)
            .add_systems(OnEnter(GameState::Playing), init_game_session);
    }
}

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GameState {
    #[default]
    Menu,
    Playing,
    GameOver,
}

#[derive(Resource)]
pub struct GameSession {
    pub difficulty: Difficulty,
    pub total_exercises: u32,
    pub current_index: u32,
    pub coins: i32,
    pub correct_count: u32,
    pub wrong_count: u32,
    pub timeout_count: u32,
    pub start_time: f64,
}

impl Default for GameSession {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Easy,
            total_exercises: 20,
            current_index: 0,
            coins: 10,
            correct_count: 0,
            wrong_count: 0,
            timeout_count: 0,
            start_time: 0.0,
        }
    }
}

fn init_game_session(mut session: ResMut<GameSession>, time: Res<Time>) {
    session.current_index = 0;
    session.coins = 10;
    session.correct_count = 0;
    session.wrong_count = 0;
    session.timeout_count = 0;
    session.start_time = time.elapsed_secs_f64();
}
