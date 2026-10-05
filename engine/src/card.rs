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

/// Une carte. Le rang va de 2 à 14 (11 = valet, 12 = dame, 13 = roi, 14 = as).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Card {
    pub rank: u8,
    pub suit: Suit,
}

pub const ACE: u8 = 14;

impl Card {
    pub fn new(rank: u8, suit: Suit) -> Self {
        assert!((2..=ACE).contains(&rank), "rang invalide : {rank}");
        Card { rank, suit }
    }

    /// Jetons rapportés par la carte quand elle marque.
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

/// Le jeu de 52 cartes, dans l'ordre.
pub fn standard_deck() -> Vec<Card> {
    Suit::ALL
        .iter()
        .flat_map(|&suit| (2..=ACE).map(move |rank| Card::new(rank, suit)))
        .collect()
}

/// Cartes de test écrites « AS KH 10D 2C » (rang puis couleur H, D, C ou S).
#[cfg(test)]
pub(crate) fn cards(text: &str) -> Vec<Card> {
    text.split_whitespace()
        .map(|word| {
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
                s => panic!("couleur inconnue : {s}"),
            };
            Card::new(rank, suit)
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
