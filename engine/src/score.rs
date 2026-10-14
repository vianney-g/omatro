//! Scoring a hand, step by step so the display can animate it.

use crate::card::Card;
use crate::hand::{HandKind, evaluate};
use crate::joker::{Effect, Joker};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    /// A played card (index into `Breakdown::cards`).
    Card { index: usize },
    /// A joker (index into the joker row).
    Joker { index: usize },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Step {
    pub source: Source,
    pub effect: Effect,
    /// Chips and mult after this step.
    pub chips: u64,
    pub mult: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Breakdown {
    pub cards: Vec<Card>,
    pub hand: HandKind,
    pub scoring: Vec<usize>,
    pub base_chips: u64,
    pub base_mult: f64,
    pub steps: Vec<Step>,
    pub chips: u64,
    pub mult: f64,
    pub total: u64,
}

pub fn score(played: &[Card], jokers: &[Joker]) -> Breakdown {
    let evaluated = evaluate(played);
    let (base_chips, base_mult) = evaluated.kind.base();
    let (mut chips, mut mult) = (base_chips, base_mult);
    let mut steps = Vec::new();
    let mut push = |source, effect: Effect, chips: &mut u64, mult: &mut f64| {
        effect.apply(chips, mult);
        steps.push(Step {
            source,
            effect,
            chips: *chips,
            mult: *mult,
        });
    };

    for &index in &evaluated.scoring {
        let card = played[index];
        push(
            Source::Card { index },
            Effect::Chips(card.chips()),
            &mut chips,
            &mut mult,
        );
        for (j, joker) in jokers.iter().enumerate() {
            if let Some(effect) = joker.on_card_scored(card) {
                push(Source::Joker { index: j }, effect, &mut chips, &mut mult);
            }
        }
    }
    for (j, joker) in jokers.iter().enumerate() {
        if let Some(effect) = joker.on_hand(played, evaluated.kind) {
            push(Source::Joker { index: j }, effect, &mut chips, &mut mult);
        }
    }

    Breakdown {
        cards: played.to_vec(),
        hand: evaluated.kind,
        scoring: evaluated.scoring,
        base_chips,
        base_mult,
        steps,
        chips,
        mult,
        total: (chips as f64 * mult).floor() as u64,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::cards;

    #[test]
    fn pair_without_jokers() {
        // Pair: 10 chips × 2, plus 10 + 10 for the kings.
        let b = score(&cards("KH 4D KS"), &[]);
        assert_eq!(b.hand, HandKind::Pair);
        assert_eq!((b.chips, b.mult, b.total), (30, 2.0, 60));
        assert_eq!(b.steps.len(), 2);
    }

    #[test]
    fn hand_jokers_apply_in_order() {
        // (30 + 30) × 2 × 2
        let b = score(&cards("KH 4D KS"), &[Joker::Dotfiles, Joker::NoBuild]);
        assert_eq!(b.total, 240);
        let last = b.steps.last().unwrap();
        assert_eq!(last.source, Source::Joker { index: 1 });
        assert_eq!(last.effect, Effect::XMult(2.0));
    }

    #[test]
    fn card_jokers_trigger_per_scoring_card() {
        // Three queens, two of them diamonds: 30 + 3×10 + 3×20 chips, 3 + 2×2 mult.
        let b = score(&cards("QD QH 2D QD"), &[Joker::RubyGem, Joker::RailsWorld]);
        assert_eq!((b.chips, b.mult), (120, 7.0));
        assert_eq!(b.total, 840);
    }

    #[test]
    fn hand_kind_jokers() {
        // Three of a kind: (30 + 3×5) × 3, then ×3 for Hotwire.
        assert_eq!(score(&cards("5H 5D 5S"), &[Joker::Hotwire]).total, 405);
        // Straight: (30 + 2+3+4+5+6) × (4 + 12).
        assert_eq!(
            score(&cards("2H 3D 4S 5C 6H"), &[Joker::TilingManager]).total,
            800
        );
        assert!(
            score(&cards("2H 2D 4S"), &[Joker::TilingManager])
                .steps
                .len()
                == 2
        );
    }

    #[test]
    fn no_build_ignores_big_hands() {
        let b = score(&cards("2H 9H KH 4H 6H"), &[Joker::NoBuild]);
        assert!(
            b.steps
                .iter()
                .all(|s| s.source != Source::Joker { index: 0 })
        );
    }
}
