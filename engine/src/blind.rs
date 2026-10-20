//! Blinds: each ante has a small, a big and a boss blind, with growing targets.

use serde::{Deserialize, Serialize};

/// Beating the boss blind of this ante wins the game; play may go on in
/// endless mode.
pub const WINNING_ANTE: u32 = 8;

/// Base score to reach in antes 1 to 8 (small blind), for each target
/// scaling: the higher difficulty levels grow faster (`Stake::scaling`).
const ANTE_BASES: [[f64; WINNING_ANTE as usize]; 3] = [
    [
        300.0, 800.0, 2_000.0, 5_000.0, 11_000.0, 20_000.0, 35_000.0, 50_000.0,
    ],
    [
        300.0, 900.0, 2_600.0, 8_000.0, 20_000.0, 36_000.0, 60_000.0, 100_000.0,
    ],
    [
        300.0, 1_000.0, 3_200.0, 9_000.0, 25_000.0, 60_000.0, 110_000.0, 200_000.0,
    ],
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BlindKind {
    Small,
    Big,
    Boss,
}

impl BlindKind {
    pub const ALL: [BlindKind; 3] = [BlindKind::Small, BlindKind::Big, BlindKind::Boss];

    pub fn name(self) -> &'static str {
        match self {
            BlindKind::Small => "Small Blind",
            BlindKind::Big => "Big Blind",
            BlindKind::Boss => "Boss Blind",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            BlindKind::Small => "A warm-up. Like a fresh Arch install.",
            BlindKind::Big => "Same thing, now with more dependencies.",
            BlindKind::Boss => "It has opinions. Strong ones.",
        }
    }

    /// Target multiplier over the ante's base score.
    pub fn multiplier(self) -> f64 {
        match self {
            BlindKind::Small => 1.0,
            BlindKind::Big => 1.5,
            BlindKind::Boss => 2.0,
        }
    }

    /// Money for beating the blind.
    pub fn reward(self) -> u32 {
        match self {
            BlindKind::Small => 3,
            BlindKind::Big => 4,
            BlindKind::Boss => 5,
        }
    }

    /// Only the small and big blinds can be skipped.
    pub fn skippable(self) -> bool {
        self != BlindKind::Boss
    }

    /// The blind after this one in the same ante.
    pub fn next(self) -> Option<BlindKind> {
        match self {
            BlindKind::Small => Some(BlindKind::Big),
            BlindKind::Big => Some(BlindKind::Boss),
            BlindKind::Boss => None,
        }
    }
}

/// Base score of an ante (from 1), at the first scaling.
pub fn ante_base(ante: u32) -> f64 {
    ante_base_scaled(ante, 1)
}

/// Base score of an ante (from 1) at a target scaling (1 to 3). Past the
/// winning ante, it grows ever faster: `a × (1.6 + (0.75 c)^d)^c` with `c`
/// antes past the 8th and `d = 1 + 0.2 c`, rounded down to two significant
/// digits.
pub fn ante_base_scaled(ante: u32, scaling: usize) -> f64 {
    let ante = ante.max(1);
    let bases = &ANTE_BASES[scaling.clamp(1, ANTE_BASES.len()) - 1];
    if ante <= WINNING_ANTE {
        return bases[ante as usize - 1];
    }
    let a = bases[WINNING_ANTE as usize - 1];
    let c = (ante - WINNING_ANTE) as f64;
    let d = 1.0 + 0.2 * c;
    let amount = (a * (1.6 + (0.75 * c).powf(d)).powf(c)).floor();
    if !amount.is_finite() {
        return amount;
    }
    let unit = 10f64.powf(amount.log10().floor() - 1.0);
    amount - amount % unit
}

/// Score needed to beat a blind.
pub fn target(ante: u32, kind: BlindKind) -> f64 {
    ante_base(ante) * kind.multiplier()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_follow_the_ante_table() {
        assert_eq!(target(1, BlindKind::Small), 300.0);
        assert_eq!(target(1, BlindKind::Big), 450.0);
        assert_eq!(target(1, BlindKind::Boss), 600.0);
        assert_eq!(target(8, BlindKind::Boss), 100_000.0);
    }

    #[test]
    fn higher_scalings_grow_faster() {
        assert_eq!(ante_base_scaled(1, 3), 300.0);
        assert_eq!(ante_base_scaled(8, 2), 100_000.0);
        assert_eq!(ante_base_scaled(8, 3), 200_000.0);
        assert!((2..20).all(|a| ante_base_scaled(a, 3) > ante_base_scaled(a, 1)));
    }

    #[test]
    fn endless_targets_explode() {
        assert_eq!(ante_base(9), 110_000.0);
        assert_eq!(ante_base(10), 560_000.0);
        assert_eq!(ante_base(11), 7_200_000.0);
        assert_eq!(ante_base(12), 300_000_000.0);
        assert!((9..38).all(|a| ante_base(a + 1) > ante_base(a)));
        // Like in the original, the target overflows to infinity at ante 39.
        assert!(ante_base(38).is_finite() && ante_base(39).is_infinite());
    }

    #[test]
    fn blinds_follow_each_other() {
        assert_eq!(BlindKind::Small.next(), Some(BlindKind::Big));
        assert_eq!(BlindKind::Boss.next(), None);
        assert!(!BlindKind::Boss.skippable());
    }
}
