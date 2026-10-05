//! Les jokers : des modificateurs déclenchés pendant le calcul du score.

use crate::card::{Card, Suit};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Joker {
    Baobab,
    Marche,
    Griot,
    Harmattan,
}

/// Effet d'un déclencheur sur le score en cours.
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
    pub const ALL: [Joker; 4] = [Joker::Baobab, Joker::Marche, Joker::Griot, Joker::Harmattan];

    pub fn name(self) -> &'static str {
        match self {
            Joker::Baobab => "Baobab",
            Joker::Marche => "Marché",
            Joker::Griot => "Griot",
            Joker::Harmattan => "Harmattan",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Joker::Baobab => "+30 jetons",
            Joker::Marche => "+2 mult pour chaque carreau qui marque",
            Joker::Griot => "+20 jetons pour chaque figure qui marque",
            Joker::Harmattan => "×2 mult si la main jouée compte 3 cartes ou moins",
        }
    }

    /// Déclenché pour chaque carte qui marque, juste après ses jetons.
    pub fn on_card_scored(self, card: Card) -> Option<Effect> {
        match self {
            Joker::Marche if card.suit == Suit::Diamonds => Some(Effect::Mult(2.0)),
            Joker::Griot if card.is_face() => Some(Effect::Chips(20)),
            _ => None,
        }
    }

    /// Déclenché une fois par main, après les cartes, de gauche à droite.
    pub fn on_hand(self, played: &[Card]) -> Option<Effect> {
        match self {
            Joker::Baobab => Some(Effect::Chips(30)),
            Joker::Harmattan if played.len() <= 3 => Some(Effect::XMult(2.0)),
            _ => None,
        }
    }
}
