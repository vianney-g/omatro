//! Jokers: modifiers triggered while a hand is scored.

use crate::card::{Card, Suit};
use crate::hand::HandKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Joker {
    Baobab,
    Calabash,
    Market,
    Wrapper,
    Griot,
    Djembe,
    Zemidjan,
    Harmattan,
    Mask,
    Tontine,
}

/// Effect of a trigger on the running score.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Chips(u64),
    Mult(f64),
    XMult(f64),
}

impl Effect {
    pub fn apply(self, chips: &mut u64, mult: &mut f64) {
        match self {
            Effect::Chips(c) => *chips += c,
            Effect::Mult(m) => *mult += m,
            Effect::XMult(x) => *mult *= x,
        }
    }
}

impl Joker {
    pub const ALL: [Joker; 10] = [
        Joker::Baobab,
        Joker::Calabash,
        Joker::Market,
        Joker::Wrapper,
        Joker::Griot,
        Joker::Djembe,
        Joker::Zemidjan,
        Joker::Harmattan,
        Joker::Mask,
        Joker::Tontine,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Joker::Baobab => "Baobab",
            Joker::Calabash => "Calabash",
            Joker::Market => "Market",
            Joker::Wrapper => "Wrapper",
            Joker::Griot => "Griot",
            Joker::Djembe => "Djembe",
            Joker::Zemidjan => "Zemidjan",
            Joker::Harmattan => "Harmattan",
            Joker::Mask => "Mask",
            Joker::Tontine => "Tontine",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Joker::Baobab => "+30 chips",
            Joker::Calabash => "+5 mult",
            Joker::Market => "+2 mult for each scoring diamond",
            Joker::Wrapper => "+15 chips for each scoring heart",
            Joker::Griot => "+20 chips for each scoring face card",
            Joker::Djembe => "+2 mult for each scoring card",
            Joker::Zemidjan => "+12 mult if the hand is a straight",
            Joker::Harmattan => "×2 mult if the played hand has 3 cards or fewer",
            Joker::Mask => "×3 mult if the hand is a three of a kind",
            Joker::Tontine => "+$1 per discard left at the end of the blind",
        }
    }

    /// Shop price; jokers sell for half.
    pub fn price(self) -> u32 {
        match self {
            Joker::Baobab | Joker::Calabash => 4,
            Joker::Market | Joker::Wrapper | Joker::Griot | Joker::Zemidjan | Joker::Tontine => 5,
            Joker::Djembe => 6,
            Joker::Harmattan => 7,
            Joker::Mask => 8,
        }
    }

    pub fn sell_value(self) -> u32 {
        (self.price() / 2).max(1)
    }

    /// Triggered for each scoring card, right after its chips.
    pub fn on_card_scored(self, card: Card) -> Option<Effect> {
        match self {
            Joker::Market if card.suit == Suit::Diamonds => Some(Effect::Mult(2.0)),
            Joker::Wrapper if card.suit == Suit::Hearts => Some(Effect::Chips(15)),
            Joker::Griot if card.is_face() => Some(Effect::Chips(20)),
            Joker::Djembe => Some(Effect::Mult(2.0)),
            _ => None,
        }
    }

    /// Triggered once per hand, after the cards, left to right.
    pub fn on_hand(self, played: &[Card], kind: HandKind) -> Option<Effect> {
        match self {
            Joker::Baobab => Some(Effect::Chips(30)),
            Joker::Calabash => Some(Effect::Mult(5.0)),
            Joker::Zemidjan if matches!(kind, HandKind::Straight | HandKind::StraightFlush) => {
                Some(Effect::Mult(12.0))
            }
            Joker::Harmattan if played.len() <= 3 => Some(Effect::XMult(2.0)),
            Joker::Mask if kind == HandKind::ThreeOfAKind => Some(Effect::XMult(3.0)),
            _ => None,
        }
    }

    /// Money earned at the end of a blind.
    pub fn on_blind_won(self, discards_left: u32) -> u32 {
        match self {
            Joker::Tontine => discards_left,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_joker_is_listed_once() {
        for joker in Joker::ALL {
            assert_eq!(Joker::ALL.iter().filter(|&&j| j == joker).count(), 1);
            assert!(joker.sell_value() >= 1 && joker.sell_value() < joker.price());
        }
    }
}
