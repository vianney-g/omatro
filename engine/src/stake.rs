//! Difficulty levels (stakes, in the original): each adds its rule to those
//! of the levels below it. Winning a game with a deck unlocks the next level
//! for that deck. Some levels put stickers on the jokers of the shop.

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Stake {
    #[default]
    DailyCommute,
    TrackDay,
    ClubRacing,
    Quattro,
    GroupB,
    LeMans,
    Endurance,
    Hypercar,
}

/// Chance for a shop joker to come with a sticker, when the level allows it.
pub const STICKER_ODDS: f64 = 0.3;

/// Blinds a Worn joker lasts.
pub const WORN_BLINDS: u32 = 5;

/// What a Leased joker costs to buy, and at the end of each blind.
pub const LEASE_PRICE: u32 = 1;
pub const LEASE_FEE: i64 = 3;

impl Stake {
    pub const ALL: [Stake; 8] = [
        Stake::DailyCommute,
        Stake::TrackDay,
        Stake::ClubRacing,
        Stake::Quattro,
        Stake::GroupB,
        Stake::LeMans,
        Stake::Endurance,
        Stake::Hypercar,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Stake::DailyCommute => "Daily Commute",
            Stake::TrackDay => "Track Day",
            Stake::ClubRacing => "Club Racing",
            Stake::Quattro => "Quattro",
            Stake::GroupB => "Group B",
            Stake::LeMans => "Le Mans",
            Stake::Endurance => "Endurance",
            Stake::Hypercar => "Hypercar",
        }
    }

    /// The rule this level adds (the levels below it apply too).
    pub fn description(self) -> &'static str {
        match self {
            Stake::DailyCommute => "The game as it is",
            Stake::TrackDay => "The small blind pays no reward",
            Stake::ClubRacing => "Targets grow faster from ante to ante",
            Stake::Quattro => "Shop jokers may be Welded",
            Stake::GroupB => "−1 discard each blind",
            Stake::LeMans => "Targets grow faster still",
            Stake::Endurance => "Shop jokers may be Worn",
            Stake::Hypercar => "Shop jokers may be Leased",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Stake::DailyCommute => "Speed limits respected. Mostly.",
            Stake::TrackDay => "Fun costs money. Lots of it.",
            Stake::ClubRacing => "The amateurs got faster.",
            Stake::Quattro => "Permanent all-wheel drive. Permanent jokers.",
            Stake::GroupB => "Too fast, no brakes. Banned in '86.",
            Stake::LeMans => "24 hours. Ask DHH how it went.",
            Stake::Endurance => "Tyres wear out. So do jokers.",
            Stake::Hypercar => "You don't own it. The sponsor does.",
        }
    }

    /// 0 for the first level.
    pub fn level(self) -> usize {
        self as usize
    }

    /// The level after this one.
    pub fn next(self) -> Option<Stake> {
        Stake::ALL.get(self.level() + 1).copied()
    }

    /// How fast targets grow: 1, 2 from Club Racing, 3 from Le Mans.
    pub fn scaling(self) -> usize {
        if self >= Stake::LeMans {
            3
        } else if self >= Stake::ClubRacing {
            2
        } else {
            1
        }
    }
}

/// A sticker on a joker, from the higher levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sticker {
    Welded,
    Worn,
    Leased,
}

impl Sticker {
    pub const ALL: [Sticker; 3] = [Sticker::Welded, Sticker::Worn, Sticker::Leased];

    pub fn name(self) -> &'static str {
        match self {
            Sticker::Welded => "Welded",
            Sticker::Worn => "Worn",
            Sticker::Leased => "Leased",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Sticker::Welded => "Can't be sold or destroyed",
            Sticker::Worn => "Stops working after 5 blinds",
            Sticker::Leased => "Costs $1 to buy, then $3 at the end of each blind",
        }
    }
}

/// The stickers of a joker.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Stickers {
    #[serde(default)]
    pub welded: bool,
    /// Blinds left before it stops working (none: not Worn).
    #[serde(default)]
    pub worn: Option<u32>,
    #[serde(default)]
    pub leased: bool,
}

impl Stickers {
    pub fn is_empty(&self) -> bool {
        *self == Stickers::default()
    }

    /// A Worn joker that used up its blinds: it does nothing.
    pub fn worn_out(&self) -> bool {
        self.worn == Some(0)
    }

    pub fn list(&self) -> Vec<Sticker> {
        let mut list = Vec::new();
        if self.welded {
            list.push(Sticker::Welded);
        }
        if self.worn.is_some() {
            list.push(Sticker::Worn);
        }
        if self.leased {
            list.push(Sticker::Leased);
        }
        list
    }
}

/// A difficulty level as the display shows it, with the rules of every level
/// up to it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StakeInfo {
    pub id: Stake,
    pub level: usize,
    pub name: &'static str,
    pub description: &'static str,
    pub quip: &'static str,
    /// What applies at this level: its own rule and those below it.
    pub rules: Vec<&'static str>,
}

impl From<Stake> for StakeInfo {
    fn from(stake: Stake) -> Self {
        StakeInfo {
            id: stake,
            level: stake.level(),
            name: stake.name(),
            description: stake.description(),
            quip: stake.quip(),
            rules: Stake::ALL[1..=stake.level().max(1)]
                .iter()
                .filter(|s| **s <= stake)
                .map(|s| s.description())
                .collect(),
        }
    }
}

/// A joker's sticker as the display shows it.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StickerInfo {
    pub id: Sticker,
    pub name: &'static str,
    pub description: &'static str,
    /// A Worn joker's blinds left.
    pub blinds_left: Option<u32>,
}

impl Stickers {
    pub fn info(&self) -> Vec<StickerInfo> {
        self.list()
            .into_iter()
            .map(|s| StickerInfo {
                id: s,
                name: s.name(),
                description: s.description(),
                blinds_left: if s == Sticker::Worn { self.worn } else { None },
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_follow_each_other_with_short_quips() {
        for (i, stake) in Stake::ALL.into_iter().enumerate() {
            assert_eq!(stake.level(), i);
            assert!(stake.quip().chars().count() <= 50, "{}", stake.quip());
        }
        assert_eq!(Stake::DailyCommute.next(), Some(Stake::TrackDay));
        assert_eq!(Stake::Hypercar.next(), None);
        assert_eq!(Stake::ALL.map(Stake::scaling), [1, 1, 2, 2, 2, 3, 3, 3]);
        assert!(StakeInfo::from(Stake::DailyCommute).rules.is_empty());
        assert_eq!(StakeInfo::from(Stake::ClubRacing).rules.len(), 2);
    }
}
