use std::ops::RangeInclusive;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Default, serde::Serialize, serde::Deserialize,
)]
pub enum Difficulty {
    #[default]
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Returns (table range, exercise timer seconds) for this difficulty.
    pub fn config(&self) -> (RangeInclusive<u32>, f32) {
        match self {
            Difficulty::Easy => (2..=9, 20.0),
            Difficulty::Medium => (2..=9, 12.0),
            Difficulty::Hard => (2..=9, 7.0),
        }
    }

    /// Returns the round time limit in seconds.
    pub fn round_time(&self) -> f32 {
        match self {
            Difficulty::Easy => 240.0,
            Difficulty::Medium => 150.0,
            Difficulty::Hard => 120.0,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Medium => "Medium",
            Difficulty::Hard => "Hard",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Difficulty::Easy => "Tables 2-9, 4 min, 20s per question",
            Difficulty::Medium => "Tables 2-9, 2.5 min, 12s per question",
            Difficulty::Hard => "Tables 2-9, 2 min, 7s per question",
        }
    }
}
