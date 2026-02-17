use std::ops::RangeInclusive;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Difficulty {
    #[default]
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Returns (table range, timer seconds) for this difficulty.
    pub fn config(&self) -> (RangeInclusive<u32>, f32) {
        match self {
            Difficulty::Easy => (2..=5, 12.0),
            Difficulty::Medium => (2..=9, 10.0),
            Difficulty::Hard => (2..=12, 7.0),
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
            Difficulty::Easy => "Tables 2-5, 12 seconds",
            Difficulty::Medium => "Tables 2-9, 10 seconds",
            Difficulty::Hard => "Tables 2-12, 7 seconds",
        }
    }
}
