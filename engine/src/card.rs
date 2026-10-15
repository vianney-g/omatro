use crate::modifier::{Edition, Enhancement, Seal};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];

    pub fn is_red(self) -> bool {
        matches!(self, Suit::Hearts | Suit::Diamonds)
    }

    pub fn symbol(self) -> &'static str {
        match self {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        }
    }
}

/// A card. Rank goes from 2 to 14 (11 = jack, 12 = queen, 13 = king, 14 = ace).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub rank: u8,
    pub suit: Suit,
    /// Tells apart cards of the deck, which lasts the whole game.
    pub id: u32,
    #[serde(default)]
    pub enhancement: Option<Enhancement>,
    #[serde(default)]
    pub seal: Option<Seal>,
    #[serde(default)]
    pub edition: Option<Edition>,
    /// Chips gained for good (Incremental Build).
    #[serde(default)]
    pub bonus_chips: u64,
    /// Fuzzy Finder: suits of the same colour count as each other. Set on the
    /// cards in play only, never kept.
    #[serde(skip)]
    pub smeared: bool,
    /// Identicon: the card counts as a face card (in play only, like `smeared`).
    #[serde(skip)]
    pub any_face: bool,
}

pub const ACE: u8 = 14;

impl Card {
    pub fn new(rank: u8, suit: Suit) -> Self {
        assert!((2..=ACE).contains(&rank), "invalid rank: {rank}");
        Card {
            rank,
            suit,
            id: 0,
            enhancement: None,
            seal: None,
            edition: None,
            bonus_chips: 0,
            smeared: false,
            any_face: false,
        }
    }

    pub fn with_id(self, id: u32) -> Self {
        Card { id, ..self }
    }

    pub fn with_enhancement(self, enhancement: Enhancement) -> Self {
        Card {
            enhancement: Some(enhancement),
            ..self
        }
    }

    pub fn with_seal(self, seal: Seal) -> Self {
        Card {
            seal: Some(seal),
            ..self
        }
    }

    pub fn with_edition(self, edition: Edition) -> Self {
        Card {
            edition: Some(edition),
            ..self
        }
    }

    pub fn is(self, enhancement: Enhancement) -> bool {
        self.enhancement == Some(enhancement)
    }

    /// A Binary Blob has no rank and no suit.
    pub fn has_rank(self) -> bool {
        !self.is(Enhancement::BinaryBlob)
    }

    /// Whether the card counts as this suit: a Symlink counts as every suit.
    pub fn is_suit(self, suit: Suit) -> bool {
        self.has_rank()
            && (self.suit == suit
                || self.is(Enhancement::Symlink)
                || (self.smeared && self.suit.is_red() == suit.is_red()))
    }

    /// Chips of the rank alone: 2 to 10, 10 for a face card, 11 for an ace.
    pub fn rank_chips(self) -> u64 {
        match self.rank {
            2..=10 => self.rank as u64,
            11..=13 => 10,
            _ => 11,
        }
    }

    /// Chips the card is worth when it scores: its rank (50 for a Binary Blob),
    /// plus Optimized and the chips it gained.
    pub fn chips(self) -> u64 {
        let base = if self.has_rank() {
            self.rank_chips()
        } else {
            50
        };
        let optimized = if self.is(Enhancement::Optimized) {
            30
        } else {
            0
        };
        base + optimized + self.bonus_chips
    }

    pub fn is_face(self) -> bool {
        self.has_rank() && (self.any_face || (11..=13).contains(&self.rank))
    }
}

/// The 52-card deck, in order, with ids 0 to 51.
pub fn standard_deck() -> Vec<Card> {
    Suit::ALL
        .iter()
        .flat_map(|&suit| (2..=ACE).map(move |rank| Card::new(rank, suit)))
        .enumerate()
        .map(|(i, card)| card.with_id(i as u32))
        .collect()
}

/// Test cards written "AS KH 10D 2C" (rank, then suit H, D, C or S), with ids
/// 100, 101… (apart from the standard deck's).
#[cfg(test)]
pub(crate) fn cards(text: &str) -> Vec<Card> {
    text.split_whitespace()
        .enumerate()
        .map(|(i, word)| {
            let (rank, suit) = word.split_at(word.len() - 1);
            let rank = match rank {
                "J" => 11,
                "Q" => 12,
                "K" => 13,
                "A" => ACE,
                n => n.parse().unwrap(),
            };
            let suit = match suit {
                "H" => Suit::Hearts,
                "D" => Suit::Diamonds,
                "C" => Suit::Clubs,
                "S" => Suit::Spades,
                s => panic!("unknown suit: {s}"),
            };
            Card::new(rank, suit).with_id(100 + i as u32)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn deck_has_52_distinct_cards() {
        let deck = standard_deck();
        assert_eq!(deck.len(), 52);
        assert_eq!(deck.iter().collect::<HashSet<_>>().len(), 52);
        assert!(deck.iter().enumerate().all(|(i, c)| c.id == i as u32));
    }

    #[test]
    fn card_chips() {
        assert_eq!(
            cards("2H 10S JD KC AS")
                .iter()
                .map(|c| c.chips())
                .collect::<Vec<_>>(),
            [2, 10, 10, 10, 11]
        );
    }
}
