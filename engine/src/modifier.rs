//! What can be added to a card: an enhancement, a seal and an edition (the
//! edition also decorates jokers).

use crate::joker::Effect;
use serde::{Deserialize, Serialize};

/// What a card is upgraded to (one at most).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Enhancement {
    Optimized,
    Multithreaded,
    Symlink,
    Overclocked,
    Daemon,
    BinaryBlob,
    Enterprise,
    FlakyTest,
}

impl Enhancement {
    pub const ALL: [Enhancement; 8] = [
        Enhancement::Optimized,
        Enhancement::Multithreaded,
        Enhancement::Symlink,
        Enhancement::Overclocked,
        Enhancement::Daemon,
        Enhancement::BinaryBlob,
        Enhancement::Enterprise,
        Enhancement::FlakyTest,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Enhancement::Optimized => "Optimized",
            Enhancement::Multithreaded => "Multithreaded",
            Enhancement::Symlink => "Symlink",
            Enhancement::Overclocked => "Overclocked",
            Enhancement::Daemon => "Daemon",
            Enhancement::BinaryBlob => "Binary Blob",
            Enhancement::Enterprise => "Enterprise",
            Enhancement::FlakyTest => "Flaky Test",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Enhancement::Optimized => "+30 chips when scored",
            Enhancement::Multithreaded => "+4 mult when scored",
            Enhancement::Symlink => "Counts as every suit",
            Enhancement::Overclocked => "×2 mult when scored; 1 in 4 chance to break",
            Enhancement::Daemon => "×1.5 mult while held in hand",
            Enhancement::BinaryBlob => "+50 chips; no rank, no suit, always scores",
            Enhancement::Enterprise => "+$3 if held in hand at the end of the blind",
            Enhancement::FlakyTest => "1 in 5 chance of +20 mult, 1 in 15 chance of +$20",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Enhancement::Optimized => "Compiled with -O3.",
            Enhancement::Multithreaded => "Uses all the cores. Finally.",
            Enhancement::Symlink => "Points wherever you need it.",
            Enhancement::Overclocked => "Fast. Hot. Occasionally on fire.",
            Enhancement::Daemon => "Runs in the background. Never sleeps.",
            Enhancement::BinaryBlob => "Nobody knows what's inside. It works.",
            Enhancement::Enterprise => "Comes with a support contract.",
            Enhancement::FlakyTest => "Passes sometimes. Spectacularly.",
        }
    }
}

/// A seal on a card (one at most).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Seal {
    GoldStar,
    Retry,
    ThemeSync,
    ShellHistory,
}

impl Seal {
    pub const ALL: [Seal; 4] = [
        Seal::GoldStar,
        Seal::Retry,
        Seal::ThemeSync,
        Seal::ShellHistory,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Seal::GoldStar => "Gold Star",
            Seal::Retry => "Retry",
            Seal::ThemeSync => "Theme Sync",
            Seal::ShellHistory => "Shell History",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Seal::GoldStar => "+$3 when scored",
            Seal::Retry => "Scores one more time",
            Seal::ThemeSync => {
                "If held in hand at the end of the blind, creates the theme of the last hand played"
            }
            Seal::ShellHistory => "When discarded, creates a command",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Seal::GoldStar => "A GitHub star, but it pays.",
            Seal::Retry => "Exit code 1. Trying again.",
            Seal::ThemeSync => "Syncs your theme. Eventually.",
            Seal::ShellHistory => "Every discard, saved for later.",
        }
    }
}

/// Hyprland decorations on a joker or a card.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Edition {
    Blur,
    Shadow,
    Animated,
    Transparent,
}

impl Edition {
    pub const ALL: [Edition; 4] = [
        Edition::Blur,
        Edition::Shadow,
        Edition::Animated,
        Edition::Transparent,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Edition::Blur => "Blur",
            Edition::Shadow => "Shadow",
            Edition::Animated => "Animated",
            Edition::Transparent => "Transparent",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Edition::Blur => "+50 chips",
            Edition::Shadow => "+10 mult",
            Edition::Animated => "×1.5 mult",
            Edition::Transparent => "+1 joker slot, or +1 hand size on a card",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Edition::Blur => "Looks expensive. Costs GPU.",
            Edition::Shadow => "Depth, for your flat design.",
            Edition::Animated => "Wobbly windows are back.",
            Edition::Transparent => "See-through. Takes no room.",
        }
    }

    /// Added to the price of a joker.
    pub fn price(self) -> u32 {
        match self {
            Edition::Blur => 2,
            Edition::Shadow => 3,
            Edition::Animated | Edition::Transparent => 5,
        }
    }

    /// Chance for a shop joker to come with this edition.
    pub fn odds(self) -> f64 {
        match self {
            Edition::Blur => 0.02,
            Edition::Shadow => 0.014,
            Edition::Animated | Edition::Transparent => 0.003,
        }
    }

    /// Effect when the joker scores: Blur and Shadow before its own effect,
    /// Animated after it.
    pub fn effect(self) -> Option<Effect> {
        match self {
            Edition::Blur => Some(Effect::Chips(50)),
            Edition::Shadow => Some(Effect::Mult(10.0)),
            Edition::Animated => Some(Effect::XMult(1.5)),
            Edition::Transparent => None,
        }
    }
}
