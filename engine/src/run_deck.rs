//! Starting decks (decks, in the original): the machine a game is played on,
//! with an advantage and sometimes a drawback.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Deck {
    ThinkPad,
    Framework,
    Basecamp,
    RaspberryPi,
    GamingRig,
    OmakaseInstall,
    ThemeHopper,
    BoardMember,
    MacRefugee,
    DualBoot,
    PowerUser,
    StudioDisplay,
    GithubProfile,
    HyprlandDwindle,
    LiveUsb,
    Raid1,
}

impl Deck {
    pub const ALL: [Deck; 16] = [
        Deck::ThinkPad,
        Deck::Framework,
        Deck::Basecamp,
        Deck::RaspberryPi,
        Deck::GamingRig,
        Deck::OmakaseInstall,
        Deck::ThemeHopper,
        Deck::BoardMember,
        Deck::MacRefugee,
        Deck::DualBoot,
        Deck::PowerUser,
        Deck::StudioDisplay,
        Deck::GithubProfile,
        Deck::HyprlandDwindle,
        Deck::LiveUsb,
        Deck::Raid1,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Deck::ThinkPad => "ThinkPad",
            Deck::Framework => "Framework Laptop",
            Deck::Basecamp => "Basecamp",
            Deck::RaspberryPi => "Raspberry Pi",
            Deck::GamingRig => "Gaming Rig",
            Deck::OmakaseInstall => "Omakase Install",
            Deck::ThemeHopper => "Theme Hopper",
            Deck::BoardMember => "Board Member",
            Deck::MacRefugee => "Mac Refugee",
            Deck::DualBoot => "Dual Boot",
            Deck::PowerUser => "Power User",
            Deck::StudioDisplay => "Studio Display",
            Deck::GithubProfile => "GitHub Profile",
            Deck::HyprlandDwindle => "Hyprland Dwindle",
            Deck::LiveUsb => "Live USB",
            Deck::Raid1 => "RAID 1",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Deck::ThinkPad => "+1 discard each blind",
            Deck::Framework => "+1 hand each blind",
            Deck::Basecamp => "Start with $10 more",
            Deck::RaspberryPi => {
                "+$2 per hand and +$1 per discard left at the end of each blind; no interest"
            }
            Deck::GamingRig => "+1 joker slot, −1 hand each blind",
            Deck::OmakaseInstall => "Start with Clipboard Manager and 2 Up Arrow",
            Deck::ThemeHopper => "Start with Theme Preview; −1 clipboard slot",
            Deck::BoardMember => "Oligarchy cards show up in the shop; start with Restructuring",
            Deck::MacRefugee => "Start with no face cards",
            Deck::DualBoot => "Start with 26 spades and 26 hearts",
            Deck::PowerUser => "Start with Shell Aliases, Theme Gallery and More Workspaces",
            Deck::StudioDisplay => "+2 hand size, −1 joker slot",
            Deck::GithubProfile => "Gain badges: 2x after each boss",
            Deck::HyprlandDwindle => {
                "Chips and mult are balanced when scoring; blinds are twice as big"
            }
            Deck::LiveUsb => "Every card's rank and suit is random",
            Deck::Raid1 => "Start with two copies of every card; +1 hand size",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Deck::ThinkPad => "The red nub sees everything.",
            Deck::Framework => "Swap a module, play another hand.",
            Deck::Basecamp => "Calm money, in the bank.",
            Deck::RaspberryPi => "Frugal by design.",
            Deck::GamingRig => "RGB sold separately.",
            Deck::OmakaseInstall => "The chef already set it up.",
            Deck::ThemeHopper => "New theme every Tuesday.",
            Deck::BoardMember => "Has a seat. Never attends.",
            Deck::MacRefugee => "Left everything behind. Even the faces.",
            Deck::DualBoot => "Half Linux, half regret.",
            Deck::PowerUser => "Dotfiles repo with 400 stars.",
            Deck::StudioDisplay => "5K pixels. One port.",
            Deck::GithubProfile => "Achievements unlocked. Twice.",
            Deck::HyprlandDwindle => "Perfectly balanced, as all splits should be.",
            Deck::LiveUsb => "Different every boot.",
            Deck::Raid1 => "Every disk, mirrored.",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decks_are_listed_once_with_short_quips() {
        for deck in Deck::ALL {
            assert_eq!(Deck::ALL.iter().filter(|&&d| d == deck).count(), 1);
            assert!(deck.quip().chars().count() <= 50, "{}", deck.quip());
        }
    }
}
