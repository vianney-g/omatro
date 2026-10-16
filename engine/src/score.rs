//! Scoring a hand, step by step so the display can animate it.

use crate::card::Card;
use crate::hand::{HandCounts, HandKind, HandLevels, Rules, evaluate_with};
use crate::joker::{Board, Effect, HandContext, Joker, OwnedJoker};
use crate::modifier::{Edition, Enhancement, Seal};
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
    /// A consumable in the clipboard (index into it): Theme Showcase.
    Clipboard { index: usize },
    /// The deck itself (Hyprland Dwindle).
    Deck,
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
    /// What happened afterwards ("Free Trial ran out.").
    pub notes: Vec<String>,
    /// Flaky Test cards that triggered (Flaky CI grows by as many).
    #[serde(skip)]
    pub lucky_hits: u32,
    /// Ids of the Overclocked cards that broke.
    #[serde(skip)]
    pub broken: Vec<u32>,
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
    /// How hands are made (MVP, Fast-Forward).
    pub rules: Rules,
    /// Big Bang Release: every played card scores.
    pub splash: bool,
    /// What the jokers see of the game: the deck, money, hands left…
    pub board: &'a Board<'a>,
    /// Ids of the cards a boss keeps from scoring.
    pub debuffed: &'a [u32],
    /// The Thermal Throttle: base chips and mult are halved.
    pub halve_base: bool,
    /// The Windows Update: this joker does nothing this hand.
    pub disabled_joker: Option<usize>,
    /// Theme Showcase: clipboard indices of the themes of the hand type played.
    pub showcase: &'a [usize],
    /// Hyprland Dwindle: chips and mult become their average.
    pub balance: bool,
}

/// The joker whose ability the one at `index` uses: itself, or for a Fork, the
/// joker on its right, for Upstream, the leftmost joker, following copies of
/// copies (none if there is nothing to copy, or the copies go round in circles).
pub fn copy_target(jokers: &[OwnedJoker], index: usize) -> Option<usize> {
    let mut target = index;
    for _ in 0..=jokers.len() {
        target = match jokers[target].joker {
            Joker::Fork => target + 1,
            Joker::Upstream => 0,
            _ => return (target == index || jokers[target].joker.copyable()).then_some(target),
        };
        if target >= jokers.len() {
            return None;
        }
    }
    None
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
    let mut evaluated = evaluate_with(played, table.rules);
    if table.splash {
        evaluated.scoring = (0..played.len()).collect();
    }
    let board = table.board;
    let (mut base_chips, mut base_mult) = table.levels.values(evaluated.kind);
    if table.halve_base {
        base_chips = (base_chips as f64 / 2.0).round() as u64;
        base_mult = (base_mult / 2.0).round().max(1.0);
    }
    let mut run = Running {
        chips: base_chips,
        mult: base_mult,
        money: 0,
        steps: Vec::new(),
    };
    let targets: Vec<Option<usize>> = (0..jokers.len())
        .map(|i| copy_target(jokers, i).filter(|_| table.disabled_joker != Some(i)))
        .collect();
    // Each joker with the joker whose ability it uses.
    let abilities = || {
        targets
            .iter()
            .enumerate()
            .filter_map(|(j, t)| t.map(|t| (j, &jokers[t])))
    };

    // Scoring cards, left to right, each scoring again once per retrigger
    // (Retry seal, jokers).
    let mut lucky_hits = 0;
    let mut broken = Vec::new();
    let first_face = evaluated.scoring.iter().copied().find(|&i| {
        played[i].is_face() && !table.debuffed.contains(&played[i].id)
    });
    let final_hand = board.hands_left == 1;
    for (position, &index) in evaluated.scoring.iter().enumerate() {
        let card = played[index];
        let source = Source::Card { index };
        if table.debuffed.contains(&card.id) {
            run.push(source, Effect::Debuffed);
            continue;
        }
        let mut retriggers: Vec<Source> = Vec::new();
        if card.seal == Some(Seal::Retry) {
            retriggers.push(source);
        }
        for (j, owned) in abilities() {
            let times = owned.joker.retriggers(card, position == 0, final_hand) as usize;
            retriggers.extend(std::iter::repeat_n(Source::Joker { index: j }, times));
        }
        for round in 0..=retriggers.len() {
            if round > 0 {
                run.push(retriggers[round - 1], Effect::Again);
            }
            run.push(source, Effect::Chips(card.chips()));
            if card.is(Enhancement::Multithreaded) {
                run.push(source, Effect::Mult(4.0));
            }
            if card.is(Enhancement::Overclocked) {
                run.push(source, Effect::XMult(2.0));
            }
            if card.is(Enhancement::FlakyTest) {
                if rng.chance(5) {
                    run.push(source, Effect::Mult(20.0));
                    lucky_hits += 1;
                }
                if rng.chance(15) {
                    run.push(source, Effect::Money(20));
                    lucky_hits += 1;
                }
            }
            if let Some(effect) = card.edition.and_then(Edition::effect) {
                run.push(source, effect);
            }
            if card.seal == Some(Seal::GoldStar) {
                run.push(source, Effect::Money(3));
            }
            for (j, owned) in abilities() {
                for effect in
                    owned
                        .joker
                        .on_card_scored(card, first_face == Some(index), board, rng)
                {
                    run.push(Source::Joker { index: j }, effect);
                }
            }
        }
        if card.is(Enhancement::Overclocked) && rng.chance(4) {
            broken.push(card.id);
        }
    }

    // Cards held in hand, once more per Cron Job; the lowest is the rightmost
    // of the lowest rank.
    let cron_jobs: Vec<usize> = abilities()
        .filter(|(_, owned)| owned.joker == Joker::CronJob)
        .map(|(j, _)| j)
        .collect();
    let lowest = held
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, c)| c.has_rank())
        .min_by_key(|(_, c)| c.rank)
        .map(|(i, _)| i);
    for (index, &card) in held.iter().enumerate() {
        if table.debuffed.contains(&card.id) {
            continue;
        }
        let mut effects = Vec::new();
        if card.is(Enhancement::Daemon) {
            effects.push(Effect::XMult(1.5));
        }
        for (_, owned) in abilities() {
            effects.extend(owned.joker.on_held(card, Some(index) == lowest, rng));
        }
        if effects.is_empty() {
            continue;
        }
        for round in 0..=cron_jobs.len() {
            if round > 0 {
                run.push(
                    Source::Joker {
                        index: cron_jobs[round - 1],
                    },
                    Effect::Again,
                );
            }
            for &effect in &effects {
                run.push(Source::Held { index }, effect);
            }
        }
    }

    // Jokers, left to right, with their editions.
    let context = HandContext {
        played,
        held,
        evaluated: &evaluated,
        times_played: table.plays.get(evaluated.kind),
        lucky_hits,
        board,
    };
    for (j, owned) in jokers.iter().enumerate() {
        if table.disabled_joker == Some(j) {
            continue;
        }
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

    // Theme Showcase: ×1.5 per theme of this hand type in the clipboard.
    for &index in table.showcase {
        run.push(Source::Clipboard { index }, Effect::XMult(1.5));
    }

    // Hyprland Dwindle: chips and mult both become their average.
    if table.balance {
        let average = ((run.chips as f64 + run.mult) / 2.0).round();
        run.chips = average as u64;
        run.mult = average;
        run.steps.push(Step {
            source: Source::Deck,
            effect: Effect::Balance,
            chips: run.chips,
            mult: run.mult,
        });
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
        lucky_hits,
        broken,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::{Suit, cards};

    struct Setup {
        held: Vec<Card>,
        deck: Vec<Card>,
        jokers: Vec<OwnedJoker>,
        levels: HandLevels,
        four_fingers: bool,
        splash: bool,
        board: Board<'static>,
    }

    fn setup(jokers: &[Joker]) -> Setup {
        Setup {
            held: Vec::new(),
            deck: Vec::new(),
            jokers: jokers.iter().map(|&j| OwnedJoker::new(j, None)).collect(),
            levels: HandLevels::default(),
            four_fingers: false,
            splash: false,
            board: Board::default(),
        }
    }

    impl Setup {
        fn play(&self, text: &str) -> Breakdown {
            self.play_cards(cards(text))
        }

        fn play_cards(&self, played: Vec<Card>) -> Breakdown {
            let mut plays = HandCounts::default();
            let rules = Rules {
                four_fingers: self.four_fingers,
                shortcut: false,
            };
            plays.add(evaluate_with(&played, rules).kind);
            let board = Board {
                deck: &self.deck,
                jokers: &self.jokers,
                ..self.board
            };
            let table = Table {
                played: &played,
                held: &self.held,
                jokers: &self.jokers,
                levels: &self.levels,
                plays: &plays,
                rules,
                splash: self.splash,
                board: &board,
                debuffed: &[],
                halve_base: false,
                disabled_joker: None,
                showcase: &[],
                balance: false,
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
    fn enhancements_seals_and_editions_score() {
        let s = setup(&[]);
        let mut played = cards("KH KD");
        played[0] = played[0]
            .with_enhancement(Enhancement::Optimized)
            .with_seal(Seal::Retry)
            .with_edition(Edition::Shadow);
        played[1] = played[1].with_enhancement(Enhancement::Multithreaded);
        let b = s.play_cards(played);
        // Pair 10 × 2; first king twice: 40 chips +10 mult each; second: 10 chips +4 mult.
        assert_eq!((b.chips, b.mult), (100, 26.0));
        assert_eq!(
            b.steps.iter().filter(|s| s.effect == Effect::Again).count(),
            1
        );
    }

    #[test]
    fn daemons_in_hand_and_cron_jobs() {
        let mut s = setup(&[Joker::CronJob]);
        s.held = vec![cards("5S")[0].with_enhancement(Enhancement::Daemon)];
        // High card ace: (5 + 11) × 1 × 1.5 × 1.5
        assert_eq!(s.play("AH").total, (16.0 * 2.25f64).floor());
    }

    #[test]
    fn overclocked_cards_may_break() {
        let s = setup(&[]);
        let breaks = (0..40)
            .filter(|&seed| {
                let played = vec![cards("KH")[0].with_enhancement(Enhancement::Overclocked)];
                let table = Table {
                    played: &played,
                    held: &[],
                    jokers: &s.jokers,
                    levels: &s.levels,
                    plays: &HandCounts::default(),
                    rules: Rules::default(),
                    splash: false,
                    board: &Board::default(),
                    debuffed: &[],
                    halve_base: false,
                    disabled_joker: None,
                    showcase: &[],
                    balance: false,
                };
                !score(&table, &mut Rng::new(seed)).broken.is_empty()
            })
            .count();
        assert!((3..=20).contains(&breaks), "{breaks}");
    }

    #[test]
    fn deck_jokers_count_enhanced_cards() {
        let mut s = setup(&[Joker::Systemd, Joker::BlobStorage]);
        s.deck = cards("2H 3H 4H")
            .into_iter()
            .map(|c| c.with_enhancement(Enhancement::Daemon))
            .chain([cards("5H")[0].with_enhancement(Enhancement::BinaryBlob)])
            .collect();
        // High card ace: (5 + 11 + 25) × 1 × 1.6
        assert_eq!(s.play("AH").total, (41.0 * 1.6f64).floor());
    }

    #[test]
    fn four_fingers_score_four_card_flushes() {
        let mut s = setup(&[]);
        s.four_fingers = true;
        let b = s.play("2H 9H KH 4H 6S");
        assert_eq!((b.hand, b.scoring.len()), (HandKind::Flush, 4));
    }

    #[test]
    fn suit_and_rank_jokers() {
        // Pair of aces: 10 + 11 + 50 + 11 chips, 2 + 7 mult.
        let b = setup(&[Joker::DeepDive, Joker::MeetupPizza]).play("AS AC");
        assert_eq!((b.chips, b.mult), (82, 9.0));
        assert_eq!(setup(&[Joker::GemInstall]).play("AD").money, 1);
        assert_eq!(setup(&[Joker::EmojiReaction]).play("KH KD").mult, 12.0);
        let b = setup(&[Joker::Error404]).play("10H 10D");
        assert_eq!((b.chips, b.mult), (50, 10.0));
        // Only the first king doubles.
        assert_eq!(setup(&[Joker::ProfilePicture]).play("KH KD").mult, 4.0);
        // Both threes score twice: 10 + 4 × 3.
        assert_eq!(setup(&[Joker::BitTwiddling]).play("3H 3D").chips, 22);
    }

    #[test]
    fn every_played_card_scores_with_big_bang_release() {
        let mut s = setup(&[]);
        s.splash = true;
        assert_eq!(s.play("KH 4D KS").chips, 34);
    }

    #[test]
    fn upstream_copies_the_leftmost_joker() {
        assert_eq!(setup(&[Joker::Omakase, Joker::Upstream]).play("KH KD").mult, 12.0);
        assert_eq!(setup(&[Joker::Upstream]).play("KH KD").mult, 2.0);
        // Fork copies Upstream, which copies Omakase.
        let b = setup(&[Joker::Omakase, Joker::Fork, Joker::Upstream]).play("KH KD");
        assert_eq!(b.mult, 17.0);
        // Copies that go round in circles copy nothing.
        let b = setup(&[Joker::Fork, Joker::Upstream, Joker::Omakase]).play("KH KD");
        assert_eq!(b.mult, 7.0);
    }

    #[test]
    fn suit_mixes_and_held_suits() {
        assert_eq!(setup(&[Joker::SplitBrain]).play("2C 2H").mult, 4.0);
        assert_eq!(setup(&[Joker::SplitBrain]).play("2C 2C").mult, 2.0);
        assert_eq!(setup(&[Joker::RainbowPrompt]).play("2C 3H 4S 5D 6C").mult, 12.0);
        let mut s = setup(&[Joker::PitchBlack]);
        s.held = cards("KS 2C");
        assert_eq!(s.play("AS").mult, 3.0);
        s.held = cards("KS 2H");
        assert_eq!(s.play("AS").mult, 1.0);
    }

    #[test]
    fn jokers_that_read_the_board() {
        let b = setup(&[Joker::PluginManager, Joker::Omakase]).play("KH KD");
        assert_eq!(b.mult, 13.0);
        let mut s = setup(&[Joker::OwnedServers, Joker::Bootstrapped]);
        s.board.money = 10;
        assert_eq!((s.play("KH KD").chips, s.play("KH KD").mult), (50, 6.0));
        let mut s = setup(&[Joker::FridayDeploy, Joker::LastCommit]);
        s.board.hands_left = 2;
        assert_eq!((s.play("KH KD").chips, s.play("KH KD").mult), (30, 2.0));
        s.board.hands_left = 1;
        assert_eq!((s.play("KH KD").chips, s.play("KH KD").mult), (50, 6.0));
        let mut s = setup(&[Joker::EmptyWorkspace, Joker::InboxZero]);
        s.board.discards_left = 2;
        assert_eq!((s.play("KH KD").chips, s.play("KH KD").mult), (90, 2.0));
        s.board.discards_left = 0;
        assert_eq!((s.play("KH KD").chips, s.play("KH KD").mult), (30, 17.0));
        let mut s = setup(&[Joker::MinimalWaybar]);
        s.board.joker_slots = 5;
        assert_eq!(s.play("KH KD").mult, 10.0);
        let mut s = setup(&[Joker::WarmStart]);
        assert_eq!(s.play("KH KD").mult, 2.0);
        s.board.blind_hands = &[HandKind::Pair];
        assert_eq!(s.play("KH KD").mult, 6.0);
    }

    #[test]
    fn jokers_aim_at_the_blind_targets() {
        let mut s = setup(&[Joker::RollingRelease, Joker::TodoMd, Joker::DailyDriver]);
        s.board.targets.suit = Suit::Hearts;
        s.board.targets.hand = HandKind::Pair;
        s.board.targets.card = (13, Suit::Diamonds);
        let b = s.play("KH KD");
        assert_eq!((b.mult, b.money), (6.0, 4));
    }
}
