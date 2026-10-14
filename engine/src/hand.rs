//! Poker hand recognition (1 to 5 played cards).

use crate::card::{ACE, Card};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
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
}

impl HandKind {
    /// Base chips and multiplier of the hand.
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Evaluated {
    pub kind: HandKind,
    /// Indices (into the played cards, ascending) of the scoring cards.
    pub scoring: Vec<usize>,
}

pub fn evaluate(cards: &[Card]) -> Evaluated {
    assert!((1..=5).contains(&cards.len()), "play 1 to 5 cards");
    let all: Vec<usize> = (0..cards.len()).collect();

    // Groups of cards of the same rank, largest then highest first.
    let mut groups: Vec<Vec<usize>> = Vec::new();
    for (i, card) in cards.iter().enumerate() {
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

    let flush = cards.len() == 5 && cards.iter().all(|c| c.suit == cards[0].suit);
    let straight = cards.len() == 5 && groups.len() == 5 && {
        let mut ranks: Vec<u8> = cards.iter().map(|c| c.rank).collect();
        ranks.sort();
        ranks[4] - ranks[0] == 4 || ranks == [2, 3, 4, 5, ACE]
    };

    let hand = |kind, mut scoring: Vec<usize>| {
        scoring.sort();
        Evaluated { kind, scoring }
    };
    let sizes: Vec<usize> = groups.iter().map(Vec::len).collect();
    match sizes.as_slice() {
        _ if straight && flush => hand(HandKind::StraightFlush, all),
        [4, ..] => hand(HandKind::FourOfAKind, groups[0].clone()),
        [3, 2] => hand(HandKind::FullHouse, all),
        _ if flush => hand(HandKind::Flush, all),
        _ if straight => hand(HandKind::Straight, all),
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
}
