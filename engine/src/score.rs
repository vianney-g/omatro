//! Scoring a hand, step by step so the display can animate it.

use crate::card::Card;
use crate::hand::{HandCounts, HandKind, HandLevels, evaluate_with};
use crate::joker::{Edition, Effect, HandContext, Joker, OwnedJoker};
use crate::rng::Rng;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    /// A played card (index into `Breakdown::cards`).
    Card { index: usize },
    /// A card held in hand (index into the hand left after playing).
    Held { index: usize },
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
    pub hand_name: &'static str,
    pub scoring: Vec<usize>,
    pub base_chips: u64,
    pub base_mult: f64,
    pub steps: Vec<Step>,
    pub chips: u64,
    pub mult: f64,
    /// A float, like the blind targets: endless mode outgrows any integer.
    pub total: f64,
    /// Money earned while scoring.
    pub money: u32,
    /// What happened to the jokers afterwards ("Free Trial ran out.").
    pub notes: Vec<String>,
}

/// Everything a hand is scored with.
pub struct Table<'a> {
    pub played: &'a [Card],
    /// Cards left in hand.
    pub held: &'a [Card],
    pub jokers: &'a [OwnedJoker],
    pub levels: &'a HandLevels,
    /// Hand types played so far, this hand included.
    pub plays: &'a HandCounts,
    /// Flushes and straights of 4 cards (MVP).
    pub four_fingers: bool,
}

/// The joker whose ability the one at `index` uses: itself, or for a Fork, the
/// first joker on its right that is not a Fork (none if there is nothing to copy).
pub fn copy_target(jokers: &[OwnedJoker], index: usize) -> Option<usize> {
    let target = (index..jokers.len()).find(|&i| jokers[i].joker != Joker::Fork)?;
    (target == index || jokers[target].joker.copyable()).then_some(target)
}

struct Running {
    chips: u64,
    mult: f64,
    money: u32,
    steps: Vec<Step>,
}

impl Running {
    fn push(&mut self, source: Source, effect: Effect) {
        effect.apply(&mut self.chips, &mut self.mult);
        if let Effect::Money(m) = effect {
            self.money += m;
        }
        self.steps.push(Step {
            source,
            effect,
            chips: self.chips,
            mult: self.mult,
        });
    }
}

pub fn score(table: &Table, rng: &mut Rng) -> Breakdown {
    let Table {
        played,
        held,
        jokers,
        ..
    } = *table;
    let evaluated = evaluate_with(played, table.four_fingers);
    let (base_chips, base_mult) = table.levels.values(evaluated.kind);
    let mut run = Running {
        chips: base_chips,
        mult: base_mult,
        money: 0,
        steps: Vec::new(),
    };
    let targets: Vec<Option<usize>> = (0..jokers.len()).map(|i| copy_target(jokers, i)).collect();
    // Each joker with the joker whose ability it uses.
    let abilities = || {
        targets
            .iter()
            .enumerate()
            .filter_map(|(j, t)| t.map(|t| (j, &jokers[t])))
    };

    // Scoring cards, left to right, each scoring again once per retrigger.
    for (position, &index) in evaluated.scoring.iter().enumerate() {
        let card = played[index];
        let retriggers: Vec<usize> = abilities()
            .flat_map(|(j, owned)| {
                let times = owned.joker.retriggers(card, position == 0) as usize;
                std::iter::repeat_n(j, times)
            })
            .collect();
        for round in 0..=retriggers.len() {
            if round > 0 {
                run.push(
                    Source::Joker {
                        index: retriggers[round - 1],
                    },
                    Effect::Again,
                );
            }
            run.push(Source::Card { index }, Effect::Chips(card.chips()));
            for (j, owned) in abilities() {
                for effect in owned.joker.on_card_scored(card, rng) {
                    run.push(Source::Joker { index: j }, effect);
                }
            }
        }
    }

    // Cards held in hand; the lowest is the rightmost of the lowest rank.
    let lowest = held
        .iter()
        .enumerate()
        .rev()
        .min_by_key(|(_, c)| c.rank)
        .map(|(i, _)| i);
    for (index, &card) in held.iter().enumerate() {
        for (_, owned) in abilities() {
            for effect in owned.joker.on_held(card, Some(index) == lowest) {
                run.push(Source::Held { index }, effect);
            }
        }
    }

    // Jokers, left to right, with their editions.
    let context = HandContext {
        played,
        evaluated: &evaluated,
        times_played: table.plays.get(evaluated.kind),
    };
    for (j, owned) in jokers.iter().enumerate() {
        let source = Source::Joker { index: j };
        let edition = owned
            .edition
            .and_then(|e| e.effect().map(|effect| (e, effect)));
        if let Some((e, effect)) = edition
            && e != Edition::Animated
        {
            run.push(source, effect);
        }
        if let Some(t) = targets[j] {
            for effect in jokers[t].joker.on_hand(jokers[t].counter, &context, rng) {
                run.push(source, effect);
            }
        }
        if let Some((Edition::Animated, effect)) = edition {
            run.push(source, effect);
        }
    }

    Breakdown {
        cards: played.to_vec(),
        hand: evaluated.kind,
        hand_name: evaluated.kind.name(),
        scoring: evaluated.scoring,
        base_chips,
        base_mult,
        steps: run.steps,
        chips: run.chips,
        mult: run.mult,
        total: (run.chips as f64 * run.mult).floor(),
        money: run.money,
        notes: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::cards;

    struct Setup {
        held: Vec<Card>,
        jokers: Vec<OwnedJoker>,
        levels: HandLevels,
        four_fingers: bool,
    }

    fn setup(jokers: &[Joker]) -> Setup {
        Setup {
            held: Vec::new(),
            jokers: jokers.iter().map(|&j| OwnedJoker::new(j, None)).collect(),
            levels: HandLevels::default(),
            four_fingers: false,
        }
    }

    impl Setup {
        fn play(&self, text: &str) -> Breakdown {
            let played = cards(text);
            let mut plays = HandCounts::default();
            plays.add(evaluate_with(&played, self.four_fingers).kind);
            let table = Table {
                played: &played,
                held: &self.held,
                jokers: &self.jokers,
                levels: &self.levels,
                plays: &plays,
                four_fingers: self.four_fingers,
            };
            score(&table, &mut Rng::new(7))
        }
    }

    fn total(text: &str, jokers: &[Joker]) -> f64 {
        setup(jokers).play(text).total
    }

    #[test]
    fn pair_without_jokers() {
        // Pair: 10 chips × 2, plus 10 + 10 for the kings.
        let b = setup(&[]).play("KH 4D KS");
        assert_eq!(b.hand, HandKind::Pair);
        assert_eq!((b.chips, b.mult, b.total), (30, 2.0, 60.0));
        assert_eq!(b.steps.len(), 2);
    }

    #[test]
    fn hand_levels_raise_the_base() {
        let mut s = setup(&[]);
        s.levels.raise(HandKind::Pair, 1);
        // Pair at level 2: 25 chips × 3, plus 10 + 10 for the kings.
        let b = s.play("KH 4D KS");
        assert_eq!((b.base_chips, b.base_mult, b.total), (25, 3.0, 135.0));
    }

    #[test]
    fn hand_jokers_apply_in_order() {
        // (30 + 30) × 2 × 2
        let b = setup(&[Joker::Dotfiles, Joker::NoBuild]).play("KH 4D KS");
        assert_eq!(b.total, 240.0);
        let last = b.steps.last().unwrap();
        assert_eq!(last.source, Source::Joker { index: 1 });
        assert_eq!(last.effect, Effect::XMult(2.0));
    }

    #[test]
    fn card_jokers_trigger_per_scoring_card() {
        // Three queens, two of them diamonds: 30 + 3×10 + 3×20 chips, 3 + 2×2 mult.
        let b = setup(&[Joker::RubyGem, Joker::RailsWorld]).play("QD QH 2D QD");
        assert_eq!((b.chips, b.mult), (120, 7.0));
        assert_eq!(b.total, 840.0);
    }

    #[test]
    fn hand_kind_jokers() {
        // Three of a kind: (30 + 3×5) × 3, then ×3 for Hotwire.
        assert_eq!(total("5H 5D 5S", &[Joker::Hotwire]), 405.0);
        // Straight: (30 + 2+3+4+5+6) × (4 + 12).
        assert_eq!(total("2H 3D 4S 5C 6H", &[Joker::TilingManager]), 800.0);
        assert!(setup(&[Joker::TilingManager]).play("2H 2D 4S").steps.len() == 2);
    }

    #[test]
    fn no_build_ignores_big_hands() {
        let b = setup(&[Joker::NoBuild]).play("2H 9H KH 4H 6H");
        assert!(
            b.steps
                .iter()
                .all(|s| s.source != Source::Joker { index: 0 })
        );
    }

    #[test]
    fn contained_hands_trigger_jokers() {
        // A full house contains a pair: (40 + 3×10 + 2×4 + 50) × (4 + 8).
        let b = setup(&[Joker::PairProgramming, Joker::RubberDuck]).play("QH 4D QS 4C QC");
        assert_eq!((b.chips, b.mult), (128, 12.0));
    }

    #[test]
    fn retriggers_score_the_card_again() {
        // King pair: the first king scores 3 times with Bang Bang: 10 + 4×10 chips.
        let b = setup(&[Joker::BangBang]).play("KH 4D KS");
        assert_eq!(b.chips, 50);
        let agains = b.steps.iter().filter(|s| s.effect == Effect::Again).count();
        assert_eq!(agains, 2);
    }

    #[test]
    fn held_cards_trigger_jokers() {
        let mut s = setup(&[Joker::LowBattery, Joker::BenevolentDictator]);
        s.held = cards("KS KH 3D 7C");
        // Pair of twos, held cards in hand order: (10 + 2 + 2) × ((2 × 1.5 × 1.5) + 2×3)
        let b = s.play("2H 2D");
        assert_eq!(b.total, 14.0 * 10.5);
        assert!(
            b.steps
                .iter()
                .any(|s| s.source == Source::Held { index: 2 })
        );
    }

    #[test]
    fn fork_copies_the_joker_on_its_right() {
        assert_eq!(
            total("KH 4D KS", &[Joker::Fork, Joker::NoBuild]),
            30.0 * 8.0
        );
        assert_eq!(
            total("KH 4D KS", &[Joker::NoBuild, Joker::Fork]),
            30.0 * 4.0
        );
        // Nothing copyable on the right: no effect.
        assert_eq!(total("KH 4D KS", &[Joker::Fork, Joker::Ultrawide]), 60.0);
        assert_eq!(
            total("KH 4D KS", &[Joker::Fork, Joker::Fork, Joker::Omakase]),
            30.0 * 17.0
        );
    }

    #[test]
    fn editions_frame_the_joker_effect() {
        let mut s = setup(&[Joker::Omakase]);
        s.jokers[0].edition = Some(Edition::Animated);
        // (2 + 5) × 1.5
        assert_eq!(s.play("KH 4D KS").mult, 10.5);
        s.jokers[0].edition = Some(Edition::Blur);
        assert_eq!(s.play("KH 4D KS").chips, 80);
    }

    #[test]
    fn muscle_memory_counts_this_hand() {
        let b = setup(&[Joker::MuscleMemory]).play("KH 4D KS");
        assert_eq!(b.mult, 3.0);
    }

    #[test]
    fn four_fingers_score_four_card_flushes() {
        let mut s = setup(&[]);
        s.four_fingers = true;
        let b = s.play("2H 9H KH 4H 6S");
        assert_eq!((b.hand, b.scoring.len()), (HandKind::Flush, 4));
    }
}
