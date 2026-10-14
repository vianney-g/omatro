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
}

/// A card. Rank goes from 2 to 14 (11 = jack, 12 = queen, 13 = king, 14 = ace).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub rank: u8,
    pub suit: Suit,
    /// Tells apart cards of the deck, which lasts the whole game.
    pub id: u32,
}

pub const ACE: u8 = 14;

impl Card {
    pub fn new(rank: u8, suit: Suit) -> Self {
        assert!((2..=ACE).contains(&rank), "invalid rank: {rank}");
        Card { rank, suit, id: 0 }
    }

    pub fn with_id(self, id: u32) -> Self {
        Card { id, ..self }
    }

    /// Chips the card is worth when it scores.
    pub fn chips(self) -> u64 {
        match self.rank {
            2..=10 => self.rank as u64,
            11..=13 => 10,
            _ => 11,
        }
    }

    pub fn is_face(self) -> bool {
        (11..=13).contains(&self.rank)
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
