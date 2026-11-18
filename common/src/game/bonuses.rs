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

    /// Emoji shown on the map and in the HUD. Must never look like a placed
    /// bomb (💣) or a flame (💥/🔥), and must be a single wide codepoint
    /// (no variation selector) so every terminal draws it on two cells.
    pub fn icon(self) -> &'static str {
        match self {
            BonusType::BombQty => "🎒",
            BonusType::BombRange => "📏",
            BonusType::Shield => "🧿",
            BonusType::Pierce => "🔪",
            BonusType::ExtraLife => "💖",
            BonusType::Detonator => "📡",
            BonusType::Dash => "👟",
        }
    }

    /// Two-character fallback for the ASCII display mode.
    pub fn ascii_icon(self) -> &'static str {
        match self {
            BonusType::BombQty => "B+",
            BonusType::BombRange => "R+",
            BonusType::Shield => "S+",
            BonusType::Pierce => "P+",
            BonusType::ExtraLife => "L+",
            BonusType::Detonator => "D+",
            BonusType::Dash => "F+",
        }
    }

    pub fn from_icon(icon: &str) -> Option<BonusType> {
        Self::WEIGHTS
            .iter()
            .map(|&(bonus, _)| bonus)
            .find(|bonus| bonus.icon() == icon)
    }
}

#[cfg(test)]
mod tests {
    use super::BonusType;

    #[test]
    fn icons_are_unique_round_trip_and_never_look_like_bombs_or_fire() {
        let all: Vec<BonusType> = BonusType::WEIGHTS.iter().map(|&(b, _)| b).collect();
        for (i, bonus) in all.iter().enumerate() {
            let icon = bonus.icon();
            assert!(!["💣", "💥", "🔥", "✨", "🧨"].contains(&icon), "{icon}");
            assert!(!icon.contains('\u{FE0F}'), "{icon} needs a variation selector");
            assert_eq!(BonusType::from_icon(icon), Some(*bonus));
            assert!(all[i + 1..].iter().all(|other| other.icon() != icon));
            assert!(all[i + 1..].iter().all(|other| other.ascii_icon() != bonus.ascii_icon()));
        }
    }
}

impl SecondItem {
    pub fn bonus(self) -> BonusType {
        match self {
            SecondItem::Shield => BonusType::Shield,
            SecondItem::Detonator => BonusType::Detonator,
            SecondItem::Dash => BonusType::Dash,
        }
    }
}
