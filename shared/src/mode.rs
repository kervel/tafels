/// What kind of exercises a round contains. Add a variant here, a generator in
/// `exercise.rs`, and a description in `difficulty.rs` to add a new game mode.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
pub enum GameMode {
    #[default]
    Tables,
    Arithmetic,
}

impl GameMode {
    pub const ALL: [GameMode; 2] = [GameMode::Tables, GameMode::Arithmetic];

    pub fn label(&self) -> &'static str {
        match self {
            GameMode::Tables => "Tables",
            GameMode::Arithmetic => "Arithmetic",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            GameMode::Tables => "Multiplication & division",
            GameMode::Arithmetic => "Addition & subtraction over 10",
        }
    }
}
