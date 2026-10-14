//! Jokers: modifiers triggered while a hand is scored.

use crate::card::{Card, Suit};
use crate::hand::HandKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Joker {
    Dotfiles,
    Omakase,
    RubyGem,
    FanMail,
    RailsWorld,
    SuperKey,
    TilingManager,
    NoBuild,
    Hotwire,
    CloudExit,
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
        Joker::Dotfiles,
        Joker::Omakase,
        Joker::RubyGem,
        Joker::FanMail,
        Joker::RailsWorld,
        Joker::SuperKey,
        Joker::TilingManager,
        Joker::NoBuild,
        Joker::Hotwire,
        Joker::CloudExit,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Joker::Dotfiles => "Dotfiles",
            Joker::Omakase => "Omakase",
            Joker::RubyGem => "Ruby Gem",
            Joker::FanMail => "Fan Mail",
            Joker::RailsWorld => "Rails World",
            Joker::SuperKey => "Super Key",
            Joker::TilingManager => "Tiling Manager",
            Joker::NoBuild => "No Build",
            Joker::Hotwire => "Hotwire",
            Joker::CloudExit => "Cloud Exit",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Joker::Dotfiles => "+30 chips",
            Joker::Omakase => "+5 mult",
            Joker::RubyGem => "+2 mult for each scoring diamond",
            Joker::FanMail => "+15 chips for each scoring heart",
            Joker::RailsWorld => "+20 chips for each scoring face card",
            Joker::SuperKey => "+2 mult for each scoring card",
            Joker::TilingManager => "+12 mult if the hand is a straight",
            Joker::NoBuild => "×2 mult if the played hand has 3 cards or fewer",
            Joker::Hotwire => "×3 mult if the hand is a three of a kind",
            Joker::CloudExit => "+$1 per discard left at the end of the blind",
        }
    }

    /// A one-line joke shown under the effect: affectionate jabs at Omarchy,
    /// Arch, Hyprland and DHH, never at the player.
    pub fn quip(self) -> &'static str {
        match self {
            Joker::Dotfiles => "It works on my machine. Now it works on yours.",
            Joker::Omakase => "You don't pick the menu. The menu picks you.",
            Joker::RubyGem => "Optimized for programmer happiness.",
            Joker::FanMail => "HEY, someone loves your pull request.",
            Joker::RailsWorld => "Everyone came for the keynote. And the opinions.",
            Joker::SuperKey => "Every good shortcut starts here.",
            Joker::TilingManager => "Floating windows are a lifestyle choice.",
            Joker::NoBuild => "Webpack? Never heard of her.",
            Joker::Hotwire => "Three frameworks walk into a monolith.",
            Joker::CloudExit => "Why rent what you can own?",
        }
    }

    /// Shop price; jokers sell for half.
    pub fn price(self) -> u32 {
        match self {
            Joker::Dotfiles | Joker::Omakase => 4,
            Joker::RubyGem
            | Joker::FanMail
            | Joker::RailsWorld
            | Joker::TilingManager
            | Joker::CloudExit => 5,
            Joker::SuperKey => 6,
            Joker::NoBuild => 7,
            Joker::Hotwire => 8,
        }
    }

    pub fn sell_value(self) -> u32 {
        (self.price() / 2).max(1)
    }

    /// Triggered for each scoring card, right after its chips.
    pub fn on_card_scored(self, card: Card) -> Option<Effect> {
        match self {
            Joker::RubyGem if card.suit == Suit::Diamonds => Some(Effect::Mult(2.0)),
            Joker::FanMail if card.suit == Suit::Hearts => Some(Effect::Chips(15)),
            Joker::RailsWorld if card.is_face() => Some(Effect::Chips(20)),
            Joker::SuperKey => Some(Effect::Mult(2.0)),
            _ => None,
        }
    }

    /// Triggered once per hand, after the cards, left to right.
    pub fn on_hand(self, played: &[Card], kind: HandKind) -> Option<Effect> {
        match self {
            Joker::Dotfiles => Some(Effect::Chips(30)),
            Joker::Omakase => Some(Effect::Mult(5.0)),
            Joker::TilingManager
                if matches!(kind, HandKind::Straight | HandKind::StraightFlush) =>
            {
                Some(Effect::Mult(12.0))
            }
            Joker::NoBuild if played.len() <= 3 => Some(Effect::XMult(2.0)),
            Joker::Hotwire if kind == HandKind::ThreeOfAKind => Some(Effect::XMult(3.0)),
            _ => None,
        }
    }

    /// Money earned at the end of a blind.
    pub fn on_blind_won(self, discards_left: u32) -> u32 {
        match self {
            Joker::CloudExit => discards_left,
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
            // Longer quips overflow the joker card in the interface.
            let quip = joker.quip();
            assert!(
                !quip.is_empty() && quip.len() <= 50 && !quip.contains('|'),
                "{quip}"
            );
        }
    }

    /// The human-readable joker reference, `docs/jokers.md`.
    fn reference() -> String {
        let mut jokers = Joker::ALL.to_vec();
        jokers.sort_by_key(|j| j.price());
        let mut doc = String::from(
            "# Jokers\n\n\
             <!-- Generated from engine/src/joker.rs by `make jokers-doc`: do not edit by hand. -->\n\n\
             Jokers are bought in the shop between blinds (5 slots at most) and sell for\n\
             half their price. While a hand is scored, card jokers trigger for each scoring\n\
             card right after its chips; hand jokers then trigger once, left to right.\n\n\
             | Id | Name | Price | Sells for | Effect | Quip |\n\
             |---|---|---|---|---|---|\n",
        );
        for joker in jokers {
            let id = serde_json::to_value(joker).unwrap();
            doc += &format!(
                "| `{}` | {} | ${} | ${} | {} | *{}* |\n",
                id.as_str().unwrap(),
                joker.name(),
                joker.price(),
                joker.sell_value(),
                joker.description(),
                joker.quip()
            );
        }
        doc
    }

    /// Fails when `docs/jokers.md` no longer matches the code; `make jokers-doc`
    /// rewrites it.
    #[test]
    fn joker_reference_is_up_to_date() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/jokers.md");
        if std::env::var_os("UPDATE_JOKERS_DOC").is_some() {
            std::fs::write(path, reference()).unwrap();
        }
        let current = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            current == reference(),
            "docs/jokers.md is out of date: run `make jokers-doc`"
        );
    }
}
