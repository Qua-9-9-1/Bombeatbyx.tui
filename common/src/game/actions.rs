use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,
    PlaceBomb,
    TriggerSpell,
}
