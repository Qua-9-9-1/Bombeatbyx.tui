use crate::game::spawns::GameRng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecondItem {
    Shield,
    Detonator,
    Dash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BonusType {
    BombQty,
    BombRange,
    Shield,
    Pierce,
    ExtraLife,
    Detonator,
    Dash,
}

impl BonusType {
    /// Relative drop weights: common upgrades first, rare items last.
    const WEIGHTS: &'static [(BonusType, u32)] = &[
        (BonusType::BombQty, 28),
        (BonusType::BombRange, 28),
        (BonusType::Shield, 12),
        (BonusType::Dash, 10),
        (BonusType::Detonator, 9),
        (BonusType::Pierce, 8),
        (BonusType::ExtraLife, 5),
    ];

    pub fn roll(rng: &mut GameRng) -> BonusType {
        let total: u32 = Self::WEIGHTS.iter().map(|(_, w)| w).sum();
        let mut pick = rng.next_u32() % total;
        for &(bonus, weight) in Self::WEIGHTS {
            if pick < weight {
                return bonus;
            }
            pick -= weight;
        }
        BonusType::BombQty
    }
}
