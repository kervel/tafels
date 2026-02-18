pub mod beacon;
pub mod coins;
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
            .insert_resource(ActiveExercises::default())
            .insert_resource(MultiplayerRoundState::default())
            .insert_resource(Leaderboard::default())
            .insert_resource(ForceSinglePlayer(false))
            .add_plugins(exercise::ExercisePlugin)
            .add_plugins(panels::PanelPlugin)
            .add_plugins(scoring::ScoringPlugin)
            .add_plugins(coins::CoinPlugin)
            .add_plugins(beacon::BeaconPlugin)
            .add_systems(OnEnter(GameState::Playing), init_game_session);
    }
}

#[derive(Resource, Default, Debug, Clone, PartialEq)]
pub enum MultiplayerRoundState {
    #[default]
    None,
    Lobby,
    Countdown(f32),
    Playing,
    RoundOver,
}

#[derive(Debug, Clone)]
pub struct LeaderboardEntry {
    pub player_id: u32,
    pub name: String,
    pub coins: i32,
}

#[derive(Resource, Default)]
pub struct Leaderboard {
    pub entries: Vec<LeaderboardEntry>,
}

#[derive(Resource)]
pub struct ActiveExercises {
    pub total_engaged: u32,
    pub total_spawned: u32,
    pub target_concurrent: u32,
    pub next_exercise_id: u32,
    pub cooldown_timer: f32,
}

impl Default for ActiveExercises {
    fn default() -> Self {
        Self {
            total_engaged: 0,
            total_spawned: 0,
            target_concurrent: 6,
            next_exercise_id: 0,
            cooldown_timer: 0.0,
        }
    }
}

#[derive(Resource)]
pub struct ForceSinglePlayer(pub bool);

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
    pub combo: u32,
    pub max_combo: u32,
    pub start_time: f64,
    pub round_time_remaining: f32,
    pub round_time_limit: f32,
    pub player_name: String,
    pub prev_coins: i32,
}

impl Default for GameSession {
    fn default() -> Self {
        let difficulty = Difficulty::Easy;
        Self {
            difficulty,
            total_exercises: 20,
            current_index: 0,
            coins: 10,
            correct_count: 0,
            wrong_count: 0,
            timeout_count: 0,
            combo: 0,
            max_combo: 0,
            start_time: 0.0,
            round_time_remaining: difficulty.round_time(),
            round_time_limit: difficulty.round_time(),
            player_name: String::new(),
            prev_coins: 10,
        }
    }
}

fn init_game_session(
    mut session: ResMut<GameSession>,
    mut active_exercises: ResMut<ActiveExercises>,
    time: Res<Time>,
) {
    session.current_index = 0;
    session.coins = 10;
    session.prev_coins = 10;
    session.correct_count = 0;
    session.wrong_count = 0;
    session.timeout_count = 0;
    session.combo = 0;
    session.max_combo = 0;
    session.start_time = time.elapsed_secs_f64();
    session.round_time_remaining = session.difficulty.round_time();
    session.round_time_limit = session.difficulty.round_time();

    *active_exercises = ActiveExercises::default();
}
