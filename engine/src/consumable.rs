//! Consumables: cards kept in the clipboard and used when the player wants.
//! For now, themes, which level up a poker hand type (planets, in the original).

use crate::hand::HandKind;
use serde::{Deserialize, Serialize};

/// Clipboard slots.
pub const CONSUMABLE_SLOTS: usize = 2;

/// An Omarchy theme: applying it levels up one hand type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Theme {
    MatteBlack,
    RosePine,
    Ristretto,
    OsakaJade,
    Nord,
    Everforest,
    Kanagawa,
    Catppuccin,
    TokyoNight,
    Hackerman,
    Ethereal,
    FlexokiLight,
}

impl Theme {
    pub const ALL: [Theme; 12] = [
        Theme::MatteBlack,
        Theme::RosePine,
        Theme::Ristretto,
        Theme::OsakaJade,
        Theme::Nord,
        Theme::Everforest,
        Theme::Kanagawa,
        Theme::Catppuccin,
        Theme::TokyoNight,
        Theme::Hackerman,
        Theme::Ethereal,
        Theme::FlexokiLight,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Theme::MatteBlack => "Matte Black",
            Theme::RosePine => "Rosé Pine",
            Theme::Ristretto => "Ristretto",
            Theme::OsakaJade => "Osaka Jade",
            Theme::Nord => "Nord",
            Theme::Everforest => "Everforest",
            Theme::Kanagawa => "Kanagawa",
            Theme::Catppuccin => "Catppuccin",
            Theme::TokyoNight => "Tokyo Night",
            Theme::Hackerman => "Hackerman",
            Theme::Ethereal => "Ethereal",
            Theme::FlexokiLight => "Flexoki Light",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Theme::MatteBlack => "Nothing matches. Nothing needs to.",
            Theme::RosePine => "A glass for you, a glass for me.",
            Theme::Ristretto => "Two shots, twice.",
            Theme::OsakaJade => "Three stones, all jade.",
            Theme::Nord => "Due north, in a straight line.",
            Theme::Everforest => "Green as far as you can see.",
            Theme::Kanagawa => "The great wave fills the house.",
            Theme::Catppuccin => "Latte, Frappé, Macchiato, Mocha.",
            Theme::TokyoNight => "Neon, in perfect sequence.",
            Theme::Hackerman => "I'm in. Five times.",
            Theme::Ethereal => "Above the clouds. No cloud bill.",
            Theme::FlexokiLight => "Blinding. Exponentially.",
        }
    }

    /// The hand type it levels up.
    pub fn hand(self) -> HandKind {
        HandKind::ALL[self as usize]
    }

    pub fn description(self) -> String {
        format!("Levels up {}", self.hand().name())
    }
}

/// A card that sits in the clipboard until used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum Consumable {
    Theme(Theme),
}

impl Consumable {
    pub fn name(self) -> &'static str {
        match self {
            Consumable::Theme(t) => t.name(),
        }
    }

    pub fn description(self) -> String {
        match self {
            Consumable::Theme(t) => t.description(),
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Consumable::Theme(t) => t.quip(),
        }
    }

    /// Shop price, before discounts.
    pub fn price(self) -> u32 {
        match self {
            Consumable::Theme(_) => 3,
        }
    }

    pub fn sell_value(self) -> u32 {
        (self.price() / 2).max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_theme_levels_up_its_own_hand() {
        let hands: Vec<HandKind> = Theme::ALL.iter().map(|t| t.hand()).collect();
        assert_eq!(hands, HandKind::ALL);
        assert_eq!(Theme::Nord.description(), "Levels up Straight");
        assert!(Theme::ALL.iter().all(|t| t.quip().chars().count() <= 50));
    }

    #[test]
    fn consumables_are_tagged_in_json() {
        let json = serde_json::to_string(&Consumable::Theme(Theme::RosePine)).unwrap();
        assert_eq!(json, r#"{"kind":"theme","id":"rose_pine"}"#);
    }
}
