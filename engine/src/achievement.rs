//! Achievements: feats kept in the profile from one game to the next. Some
//! unlock a hidden item, which games never offer before.

use crate::collection::Item;
use crate::joker::Joker;
use crate::run_deck::Deck;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Achievement {
    HelloWorld,
    ItsNotAPhase,
    DuplicateDetected,
    CoreDump,
    OneLiner,
    BigO,
    HeatDeath,
    BareMetal,
    SavingsAccount,
    EverythingMustGo,
    SunkCost,
    LongTermSupport,
    Rtfm,
    Collector,
    NothingLeftToInstall,
}

/// What a counting achievement counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Count {
    GamesLost,
    JokersSold,
    Found,
}

/// Most jokers Bare Metal allows at once.
pub const BARE_METAL_JOKERS: usize = 2;

impl Achievement {
    pub const ALL: [Achievement; 15] = [
        Achievement::HelloWorld,
        Achievement::ItsNotAPhase,
        Achievement::DuplicateDetected,
        Achievement::CoreDump,
        Achievement::OneLiner,
        Achievement::BigO,
        Achievement::HeatDeath,
        Achievement::BareMetal,
        Achievement::SavingsAccount,
        Achievement::EverythingMustGo,
        Achievement::SunkCost,
        Achievement::LongTermSupport,
        Achievement::Rtfm,
        Achievement::Collector,
        Achievement::NothingLeftToInstall,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Achievement::HelloWorld => "Hello, World",
            Achievement::ItsNotAPhase => "It's Not a Phase",
            Achievement::DuplicateDetected => "Duplicate Detected",
            Achievement::CoreDump => "Core Dump",
            Achievement::OneLiner => "One-Liner",
            Achievement::BigO => "Big O",
            Achievement::HeatDeath => "Heat Death",
            Achievement::BareMetal => "Bare Metal",
            Achievement::SavingsAccount => "Savings Account",
            Achievement::EverythingMustGo => "Everything Must Go",
            Achievement::SunkCost => "Sunk Cost",
            Achievement::LongTermSupport => "Long-Term Support",
            Achievement::Rtfm => "RTFM",
            Achievement::Collector => "Collector",
            Achievement::NothingLeftToInstall => "Nothing Left to Install",
        }
    }

    /// What it takes.
    pub fn description(self) -> &'static str {
        match self {
            Achievement::HelloWorld => "Win a blind",
            Achievement::ItsNotAPhase => "Win a game",
            Achievement::DuplicateDetected => "Play a Fork Bomb: five identical cards",
            Achievement::CoreDump => "Lose 5 games",
            Achievement::OneLiner => "Beat a boss blind in one hand, without discarding",
            Achievement::BigO => "Score 1,000,000 in one hand",
            Achievement::HeatDeath => "Score 100,000,000 in one hand",
            Achievement::BareMetal => "Win a game without ever holding more than 2 jokers",
            Achievement::SavingsAccount => "Have $200 at once",
            Achievement::EverythingMustGo => "Sell 50 jokers",
            Achievement::SunkCost => "Reroll 10 times in one shop",
            Achievement::LongTermSupport => "Reach ante 12",
            Achievement::Rtfm => "Open the collection",
            Achievement::Collector => "Find 100 things for the collection",
            Achievement::NothingLeftToInstall => "Find everything in the collection",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Achievement::HelloWorld => "Every journey starts with a printf.",
            Achievement::ItsNotAPhase => "Yes, mom, I use Arch now.",
            Achievement::DuplicateDetected => "Same card, five times. Totally legit.",
            Achievement::CoreDump => "Something went wrong. Five times.",
            Achievement::OneLiner => "Could have been a shell script.",
            Achievement::BigO => "O(1) mult. O(n!) chips.",
            Achievement::HeatDeath => "The universe ends. Your score doesn't.",
            Achievement::BareMetal => "No VMs. No containers. No jokers.",
            Achievement::SavingsAccount => "Cloud exit savings, finally visible.",
            Achievement::EverythingMustGo => "Even Fork had a price.",
            Achievement::SunkCost => "One more reroll. Just one.",
            Achievement::LongTermSupport => "Rolling release, rolling ante.",
            Achievement::Rtfm => "You actually read the manual.",
            Achievement::Collector => "Gotta pacman -S them all.",
            Achievement::NothingLeftToInstall => "Nothing left to install.",
        }
    }

    /// The hidden item it unlocks: games never offer it before.
    pub fn unlocks(self) -> Option<Item> {
        match self {
            Achievement::ItsNotAPhase => Some(Item::Deck(Deck::Raid1)),
            Achievement::DuplicateDetected => Some(Item::Joker(Joker::HashCollision)),
            Achievement::CoreDump => Some(Item::Joker(Joker::Postmortem)),
            Achievement::HeatDeath => Some(Item::Joker(Joker::IntegerOverflow)),
            Achievement::BareMetal => Some(Item::Joker(Joker::SingleBinary)),
            _ => None,
        }
    }

    /// The achievement that unlocks a hidden item.
    pub fn unlocking(item: Item) -> Option<Achievement> {
        Achievement::ALL.into_iter().find(|a| a.unlocks() == Some(item))
    }

    /// What a counting achievement counts, and how far (none: the whole
    /// collection).
    pub fn goal(self) -> Option<(Count, Option<u64>)> {
        match self {
            Achievement::CoreDump => Some((Count::GamesLost, Some(5))),
            Achievement::EverythingMustGo => Some((Count::JokersSold, Some(50))),
            Achievement::Collector => Some((Count::Found, Some(100))),
            Achievement::NothingLeftToInstall => Some((Count::Found, None)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn achievements_are_listed_once_with_short_quips() {
        for a in Achievement::ALL {
            assert_eq!(Achievement::ALL.iter().filter(|&&b| b == a).count(), 1);
            assert!(a.quip().chars().count() <= 50, "{}", a.quip());
        }
        for item in Achievement::ALL.iter().filter_map(|a| a.unlocks()) {
            assert!(Item::all().contains(&item));
        }
    }
}
