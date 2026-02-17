use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
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
            Difficulty::Easy => (2..=5, 12.0),
            Difficulty::Medium => (2..=9, 10.0),
            Difficulty::Hard => (2..=12, 7.0),
        }
    }

    /// Returns the round time limit in seconds.
    pub fn round_time(&self) -> f32 {
        match self {
            Difficulty::Easy => 180.0,    // 3 minutes
            Difficulty::Medium => 150.0,  // 2.5 minutes
            Difficulty::Hard => 120.0,    // 2 minutes
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
            Difficulty::Easy => "Tables 2-5, 3 min round",
            Difficulty::Medium => "Tables 2-9, 2.5 min round",
            Difficulty::Hard => "Tables 2-12, 2 min round",
        }
    }
}
