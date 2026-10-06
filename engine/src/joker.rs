//! Les jokers : des modificateurs déclenchés pendant le calcul du score.

use crate::card::{Card, Suit};
use crate::hand::HandKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Joker {
    Baobab,
    Calebasse,
    Marche,
    Pagne,
    Griot,
    Djembe,
    Zemidjan,
    Harmattan,
    Masque,
    Tontine,
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
    pub const ALL: [Joker; 10] = [
        Joker::Baobab,
        Joker::Calebasse,
        Joker::Marche,
        Joker::Pagne,
        Joker::Griot,
        Joker::Djembe,
        Joker::Zemidjan,
        Joker::Harmattan,
        Joker::Masque,
        Joker::Tontine,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Joker::Baobab => "Baobab",
            Joker::Calebasse => "Calebasse",
            Joker::Marche => "Marché",
            Joker::Pagne => "Pagne",
            Joker::Griot => "Griot",
            Joker::Djembe => "Djembé",
            Joker::Zemidjan => "Zémidjan",
            Joker::Harmattan => "Harmattan",
            Joker::Masque => "Masque",
            Joker::Tontine => "Tontine",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Joker::Baobab => "+30 jetons",
            Joker::Calebasse => "+5 mult",
            Joker::Marche => "+2 mult pour chaque carreau qui marque",
            Joker::Pagne => "+15 jetons pour chaque cœur qui marque",
            Joker::Griot => "+20 jetons pour chaque figure qui marque",
            Joker::Djembe => "+2 mult pour chaque carte qui marque",
            Joker::Zemidjan => "+12 mult si la main est une suite",
            Joker::Harmattan => "×2 mult si la main jouée compte 3 cartes ou moins",
            Joker::Masque => "×3 mult si la main est un brelan",
            Joker::Tontine => "+1 $ par défausse restante à la fin de la blind",
        }
    }

    /// Prix en boutique ; il se revend la moitié.
    pub fn price(self) -> u32 {
        match self {
            Joker::Baobab | Joker::Calebasse => 4,
            Joker::Marche | Joker::Pagne | Joker::Griot | Joker::Zemidjan | Joker::Tontine => 5,
            Joker::Djembe => 6,
            Joker::Harmattan => 7,
            Joker::Masque => 8,
        }
    }

    pub fn sell_value(self) -> u32 {
        (self.price() / 2).max(1)
    }

    /// Déclenché pour chaque carte qui marque, juste après ses jetons.
    pub fn on_card_scored(self, card: Card) -> Option<Effect> {
        match self {
            Joker::Marche if card.suit == Suit::Diamonds => Some(Effect::Mult(2.0)),
            Joker::Pagne if card.suit == Suit::Hearts => Some(Effect::Chips(15)),
            Joker::Griot if card.is_face() => Some(Effect::Chips(20)),
            Joker::Djembe => Some(Effect::Mult(2.0)),
            _ => None,
        }
    }

    /// Déclenché une fois par main, après les cartes, de gauche à droite.
    pub fn on_hand(self, played: &[Card], kind: HandKind) -> Option<Effect> {
        match self {
            Joker::Baobab => Some(Effect::Chips(30)),
            Joker::Calebasse => Some(Effect::Mult(5.0)),
            Joker::Zemidjan if matches!(kind, HandKind::Straight | HandKind::StraightFlush) => {
                Some(Effect::Mult(12.0))
            }
            Joker::Harmattan if played.len() <= 3 => Some(Effect::XMult(2.0)),
            Joker::Masque if kind == HandKind::ThreeOfAKind => Some(Effect::XMult(3.0)),
            _ => None,
        }
    }

    /// Argent rapporté en fin de blind.
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
