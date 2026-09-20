use std::ops::RangeInclusive;

use crate::mode::GameMode;

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

    pub fn description(&self, mode: GameMode) -> &'static str {
        match (mode, self) {
            (GameMode::Tables, Difficulty::Easy) => "Tables 2-9, 4 min, 20s per question",
            (GameMode::Tables, Difficulty::Medium) => "Tables 2-9, 2.5 min, 12s per question",
            (GameMode::Tables, Difficulty::Hard) => "Tables 2-9, 2 min, 7s per question",
            (GameMode::Arithmetic, Difficulty::Easy) => "Up to 20, 4 min, 20s per question",
            (GameMode::Arithmetic, Difficulty::Medium) => "Up to 100, 2.5 min, 12s per question",
            (GameMode::Arithmetic, Difficulty::Hard) => "Two-digit up to 100, 2 min, 7s per question",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mode::GameMode;

    #[test]
    fn description_depends_on_mode() {
        let d = Difficulty::Easy;
        assert!(d.description(GameMode::Tables).contains("Tables"));
        assert!(d.description(GameMode::Arithmetic).contains("20"));
        assert_ne!(d.description(GameMode::Tables), d.description(GameMode::Arithmetic));
    }

    #[test]
    fn game_mode_all_lists_every_variant_with_labels() {
        assert_eq!(GameMode::ALL, [GameMode::Tables, GameMode::Arithmetic]);
        assert_eq!(GameMode::Tables.label(), "Tables");
        assert_eq!(GameMode::Arithmetic.label(), "Arithmetic");
        assert_eq!(GameMode::default(), GameMode::Tables);
    }
}
