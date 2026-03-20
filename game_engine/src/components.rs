#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Velocity {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AiState {
    pub target: Option<Position>,
    pub state: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LodLevel {
    Active,
    Simulated,
    Background,
}