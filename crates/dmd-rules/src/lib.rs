use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RollMode {
    Normal,
    Advantage,
    Disadvantage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollRequest {
    pub label: String,
    pub dice: String,
    pub modifier: i32,
    pub mode: RollMode,
    pub hidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RollResult {
    pub raw_values: Vec<i32>,
    pub modifier: i32,
    pub total: i32,
}
