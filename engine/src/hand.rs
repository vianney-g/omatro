//! Poker hand recognition (1 to 5 played cards) and hand levels.

use crate::card::{ACE, Card, Suit};
use serde::{Deserialize, Serialize};

/// Hand types, weakest first. The last three need duplicate cards: they stay
/// secret until played.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandKind {
    HighCard,
    Pair,
    TwoPair,
    ThreeOfAKind,
    Straight,
    Flush,
    FullHouse,
    FourOfAKind,
    StraightFlush,
    FiveOfAKind,
    /// A full house of a single suit.
    MajesticMonolith,
    /// Five identical cards.
    ForkBomb,
}

impl HandKind {
    pub const ALL: [HandKind; 12] = [
        HandKind::HighCard,
        HandKind::Pair,
        HandKind::TwoPair,
        HandKind::ThreeOfAKind,
        HandKind::Straight,
        HandKind::Flush,
        HandKind::FullHouse,
        HandKind::FourOfAKind,
        HandKind::StraightFlush,
        HandKind::FiveOfAKind,
        HandKind::MajesticMonolith,
        HandKind::ForkBomb,
    ];

    pub fn name(self) -> &'static str {
        match self {
            HandKind::HighCard => "High Card",
            HandKind::Pair => "Pair",
            HandKind::TwoPair => "Two Pair",
            HandKind::ThreeOfAKind => "Three of a Kind",
            HandKind::Straight => "Straight",
            HandKind::Flush => "Flush",
            HandKind::FullHouse => "Full House",
            HandKind::FourOfAKind => "Four of a Kind",
            HandKind::StraightFlush => "Straight Flush",
            HandKind::FiveOfAKind => "Five of a Kind",
            HandKind::MajesticMonolith => "Majestic Monolith",
            HandKind::ForkBomb => "Fork Bomb",
        }
    }

    /// Only the secret hands get a quip.
    pub fn quip(self) -> &'static str {
        match self {
            HandKind::FiveOfAKind => "Statistically suspicious.",
            HandKind::MajesticMonolith => "One big beautiful app. No microservices.",
            HandKind::ForkBomb => ":(){ :|:& };:",
            _ => "",
        }
    }

    pub fn secret(self) -> bool {
        self >= HandKind::FiveOfAKind
    }

    /// Chips and mult at level 1.
    pub fn base(self) -> (u64, f64) {
        match self {
            HandKind::HighCard => (5, 1.0),
            HandKind::Pair => (10, 2.0),
            HandKind::TwoPair => (20, 2.0),
            HandKind::ThreeOfAKind => (30, 3.0),
            HandKind::Straight => (30, 4.0),
            HandKind::Flush => (35, 4.0),
            HandKind::FullHouse => (40, 4.0),
            HandKind::FourOfAKind => (60, 7.0),
            HandKind::StraightFlush => (100, 8.0),
            HandKind::FiveOfAKind => (120, 12.0),
            HandKind::MajesticMonolith => (140, 14.0),
            HandKind::ForkBomb => (160, 16.0),
        }
    }

    /// Chips and mult added by each level past the first.
    pub fn per_level(self) -> (u64, f64) {
        match self {
            HandKind::HighCard => (10, 1.0),
            HandKind::Pair => (15, 1.0),
            HandKind::TwoPair => (20, 1.0),
            HandKind::ThreeOfAKind => (20, 2.0),
            HandKind::Straight => (30, 3.0),
            HandKind::Flush => (15, 2.0),
            HandKind::FullHouse => (25, 2.0),
            HandKind::FourOfAKind => (30, 3.0),
            HandKind::StraightFlush => (40, 4.0),
            HandKind::FiveOfAKind => (35, 3.0),
            HandKind::MajesticMonolith => (40, 4.0),
            HandKind::ForkBomb => (50, 3.0),
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

/// The level of each hand type, raised during the game.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandLevels([u32; HandKind::ALL.len()]);

impl Default for HandLevels {
    fn default() -> Self {
        HandLevels([1; HandKind::ALL.len()])
    }
}

impl HandLevels {
    pub fn level(&self, kind: HandKind) -> u32 {
        self.0[kind.index()]
    }

    pub fn raise(&mut self, kind: HandKind, levels: u32) {
        self.0[kind.index()] += levels;
    }

    /// One level less, never below the first (The Regression).
    pub fn lower(&mut self, kind: HandKind) {
        let level = &mut self.0[kind.index()];
        *level = (*level - 1).max(1);
    }

    /// Starting chips and mult of a hand at its current level.
    pub fn values(&self, kind: HandKind) -> (u64, f64) {
        let (chips, mult) = kind.base();
        let (step_chips, step_mult) = kind.per_level();
        let extra = self.level(kind) - 1;
        (
            chips + step_chips * extra as u64,
            mult + step_mult * extra as f64,
        )
    }
}

/// How many times each hand type was played this game.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HandCounts([u32; HandKind::ALL.len()]);

impl HandCounts {
    pub fn get(&self, kind: HandKind) -> u32 {
        self.0[kind.index()]
    }

    pub fn add(&mut self, kind: HandKind) {
        self.0[kind.index()] += 1;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Evaluated {
    pub kind: HandKind,
    /// Indices (into the played cards, ascending) of the scoring cards.
    pub scoring: Vec<usize>,
    /// Every hand type the cards contain: a full house contains a pair, a three
    /// of a kind and a two pair.
    pub contains: Vec<HandKind>,
}

impl Evaluated {
    pub fn contains(&self, kind: HandKind) -> bool {
        self.contains.contains(&kind)
    }
}

pub fn evaluate(cards: &[Card]) -> Evaluated {
    evaluate_with(cards, Rules::default())
}

/// Jokers that bend how hands are made.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Rules {
    /// MVP: flushes and straights need only 4 cards.
    pub four_fingers: bool,
    /// Fast-Forward: straights may skip one rank between two cards.
    pub shortcut: bool,
}

/// Cards of a straight: a run of `need` ranks, each one or (with `shortcut`)
/// two below the previous, the ace also counting low; the highest run wins.
fn straight_run(cards: &[Card], need: u8, shortcut: bool) -> Vec<usize> {
    // Rank 1 stands for the ace played low; Binary Blobs have no rank.
    let is_rank = |card: &Card, rank: u8| {
        card.has_rank() && (card.rank == rank || (rank == 1 && card.rank == ACE))
    };
    let present = |rank: u8| cards.iter().any(|c| is_rank(c, rank));
    let max_step = if shortcut { 2 } else { 1 };
    // The ranks of a run of `left` cards going down from `top`.
    fn run(top: u8, left: u8, max_step: u8, present: &dyn Fn(u8) -> bool) -> Option<Vec<u8>> {
        if !present(top) {
            return None;
        }
        if left == 1 {
            return Some(vec![top]);
        }
        (1..=max_step).filter(|&s| top > s).find_map(|step| {
            let mut ranks = run(top - step, left - 1, max_step, present)?;
            ranks.push(top);
            Some(ranks)
        })
    }
    for top in (1..=ACE).rev() {
        if let Some(ranks) = run(top, need, max_step, &present) {
            return (0..cards.len())
                .filter(|&i| ranks.iter().any(|&r| is_rank(&cards[i], r)))
                .collect();
        }
    }
    Vec::new()
}

/// Binary Blobs take no part in the hand but always score; Symlinks count as
/// every suit.
pub fn evaluate_with(cards: &[Card], rules: Rules) -> Evaluated {
    assert!((1..=5).contains(&cards.len()), "play 1 to 5 cards");
    let all: Vec<usize> = (0..cards.len()).collect();
    let need = if rules.four_fingers { 4 } else { 5 };

    let blobs: Vec<usize> = all
        .iter()
        .copied()
        .filter(|&i| !cards[i].has_rank())
        .collect();

    // Groups of cards of the same rank, largest then highest first.
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for (i, card) in cards.iter().enumerate().filter(|(_, c)| c.has_rank()) {
        match groups.iter_mut().find(|g| cards[g[0]].rank == card.rank) {
            Some(group) => group.push(i),
            None => groups.push(vec![i]),
        }
    }
    groups.sort_by(|a, b| {
        b.len()
            .cmp(&a.len())
            .then(cards[b[0]].rank.cmp(&cards[a[0]].rank))
    });
    let sizes: Vec<usize> = groups.iter().map(Vec::len).collect();

    let flush_cards: Vec<usize> = Suit::ALL
        .iter()
        .map(|&suit| {
            all.iter()
                .copied()
                .filter(|&i| cards[i].is_suit(suit))
                .collect::<Vec<_>>()
        })
        .find(|suited| suited.len() >= need as usize)
        .unwrap_or_default();
    let straight_cards = straight_run(cards, need, rules.shortcut);
    let (flush, straight) = (!flush_cards.is_empty(), !straight_cards.is_empty());

    let pairs = sizes.iter().filter(|&&s| s >= 2).count();
    let largest = sizes.first().copied().unwrap_or(0);
    let mut contains = vec![HandKind::HighCard];
    let mut include = |kind, yes: bool| {
        if yes {
            contains.push(kind);
        }
    };
    include(HandKind::Pair, largest >= 2);
    include(HandKind::TwoPair, pairs >= 2);
    include(HandKind::ThreeOfAKind, largest >= 3);
    include(HandKind::FourOfAKind, largest >= 4);
    include(HandKind::FullHouse, largest >= 3 && pairs >= 2);
    include(HandKind::Straight, straight);
    include(HandKind::Flush, flush);
    include(HandKind::StraightFlush, straight && flush);

    let hand = |kind, scoring: Vec<usize>| {
        let mut scoring = [scoring, blobs.clone()].concat();
        scoring.sort();
        scoring.dedup();
        let mut contains = contains.clone();
        if !contains.contains(&kind) {
            contains.push(kind);
        }
        Evaluated {
            kind,
            scoring,
            contains,
        }
    };
    match sizes.as_slice() {
        [5] if flush => hand(HandKind::ForkBomb, all),
        [3, 2] if flush => hand(HandKind::MajesticMonolith, all),
        [5] => hand(HandKind::FiveOfAKind, all),
        [] => hand(HandKind::HighCard, Vec::new()),
        _ if straight && flush => hand(
            HandKind::StraightFlush,
            [straight_cards, flush_cards].concat(),
        ),
        [4, ..] => hand(HandKind::FourOfAKind, groups[0].clone()),
        [3, 2] => hand(HandKind::FullHouse, all),
        _ if flush => hand(HandKind::Flush, flush_cards),
        _ if straight => hand(HandKind::Straight, straight_cards),
        [3, ..] => hand(HandKind::ThreeOfAKind, groups[0].clone()),
        [2, 2, ..] => hand(
            HandKind::TwoPair,
            [groups[0].clone(), groups[1].clone()].concat(),
        ),
        [2, ..] => hand(HandKind::Pair, groups[0].clone()),
        _ => hand(HandKind::HighCard, vec![groups[0][0]]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::cards;

    fn check(text: &str, kind: HandKind, scoring: &[usize]) {
        let result = evaluate(&cards(text));
        assert_eq!(result.kind, kind, "{text}");
        assert_eq!(result.scoring, scoring, "{text}");
    }

    #[test]
    fn high_card_scores_only_the_highest() {
        check("3H KD 7S", HandKind::HighCard, &[1]);
    }

    #[test]
    fn pairs_and_sets() {
        check("5H 9D 5S", HandKind::Pair, &[0, 2]);
        check("5H 9D 5S 9C 2H", HandKind::TwoPair, &[0, 1, 2, 3]);
        check("QH QD 3S QC", HandKind::ThreeOfAKind, &[0, 1, 3]);
        check("QH QD QS QC 2H", HandKind::FourOfAKind, &[0, 1, 2, 3]);
        check("QH 4D QS 4C QC", HandKind::FullHouse, &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn straights_and_flushes() {
        check("9H 7D 8S JC 10H", HandKind::Straight, &[0, 1, 2, 3, 4]);
        check("AH 2D 3S 4C 5H", HandKind::Straight, &[0, 1, 2, 3, 4]);
        check("2H 9H KH 4H 6H", HandKind::Flush, &[0, 1, 2, 3, 4]);
        check("9S 7S 8S JS 10S", HandKind::StraightFlush, &[0, 1, 2, 3, 4]);
    }

    #[test]
    fn no_wraparound_straight_and_no_short_flush() {
        check("QH KD AS 2C 3H", HandKind::HighCard, &[2]);
        check("2H 9H KH 4H", HandKind::HighCard, &[2]);
    }

    #[test]
    fn hands_contain_smaller_ones() {
        let full = evaluate(&cards("QH 4D QS 4C QC"));
        use HandKind::*;
        for kind in [HighCard, Pair, TwoPair, ThreeOfAKind, FullHouse] {
            assert!(full.contains(kind), "{kind:?}");
        }
        assert!(!full.contains(FourOfAKind) && !full.contains(Flush));
        let four = evaluate(&cards("7H 7D 7S 7C 2H"));
        assert!(four.contains(ThreeOfAKind) && !four.contains(TwoPair));
    }

    #[test]
    fn shortcuts_skip_a_rank_in_straights() {
        let rules = Rules {
            four_fingers: false,
            shortcut: true,
        };
        let short = |text| evaluate_with(&cards(text), rules);
        assert_eq!(short("2H 4D 6S 8C 10H").kind, HandKind::Straight);
        assert_eq!(short("AH 3D 4S 6C 7H").kind, HandKind::Straight);
        assert_eq!(short("2H 5D 6S 7C 8H").kind, HandKind::HighCard);
        assert_eq!(evaluate(&cards("2H 4D 6S 8C 10H")).kind, HandKind::HighCard);
    }

    #[test]
    fn four_fingers_make_short_flushes_and_straights() {
        let rules = Rules {
            four_fingers: true,
            shortcut: false,
        };
        let four = |text| evaluate_with(&cards(text), rules);
        let flush = four("2H 9H KH 4H 6S");
        assert_eq!(
            (flush.kind, flush.scoring.clone()),
            (HandKind::Flush, vec![0, 1, 2, 3])
        );
        let straight = four("5C 6D 7S 8H KC");
        assert_eq!(
            (straight.kind, straight.scoring.clone()),
            (HandKind::Straight, vec![0, 1, 2, 3])
        );
        assert_eq!(four("AH 2H 3H 4H 9C").kind, HandKind::StraightFlush);
        assert_eq!(four("AH 2H 3H 4H 9C").scoring, [0, 1, 2, 3]);
        assert_eq!(evaluate(&cards("2H 9H KH 4H 6S")).kind, HandKind::HighCard);
    }

    #[test]
    fn binary_blobs_always_score_and_symlinks_suit_everything() {
        use crate::modifier::Enhancement;
        let mut played = cards("KH KD 2S");
        played[2] = played[2].with_enhancement(Enhancement::BinaryBlob);
        let result = evaluate(&played);
        assert_eq!(
            (result.kind, result.scoring),
            (HandKind::Pair, vec![0, 1, 2])
        );
        let blobs: Vec<Card> = cards("KH 2D")
            .into_iter()
            .map(|c| c.with_enhancement(Enhancement::BinaryBlob))
            .collect();
        assert_eq!(evaluate(&blobs).scoring, [0, 1]);
        let mut flush = cards("2H 9H KH 4H 6S");
        flush[4] = flush[4].with_enhancement(Enhancement::Symlink);
        assert_eq!(evaluate(&flush).kind, HandKind::Flush);
    }

    #[test]
    fn secret_hands_need_duplicates() {
        check("7H 7D 7S 7C 7H", HandKind::FiveOfAKind, &[0, 1, 2, 3, 4]);
        check(
            "QH 4H QH 4H QH",
            HandKind::MajesticMonolith,
            &[0, 1, 2, 3, 4],
        );
        check("7S 7S 7S 7S 7S", HandKind::ForkBomb, &[0, 1, 2, 3, 4]);
        assert!(
            HandKind::ALL
                .iter()
                .all(|k| k.secret() == !k.quip().is_empty())
        );
    }

    #[test]
    fn levels_add_chips_and_mult() {
        let mut levels = HandLevels::default();
        assert_eq!(levels.values(HandKind::Pair), (10, 2.0));
        levels.raise(HandKind::Pair, 2);
        assert_eq!(levels.level(HandKind::Pair), 3);
        assert_eq!(levels.values(HandKind::Pair), (40, 4.0));
        assert_eq!(levels.values(HandKind::Flush), (35, 4.0));
    }
}
