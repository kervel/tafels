#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CharacterState {
    #[default]
    Idle,
    Walking,
    Running,
}
