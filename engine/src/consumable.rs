//! Consumables: cards kept in the clipboard and used when the player wants.
//! Themes level up a poker hand type (planets, in the original); commands
//! change cards or create things (tarots).

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

/// An Omarchy or shell command: changes cards, makes money or creates things
/// (tarots, in the original).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    UpArrow,
    NpmTest,
    ThemeInstall,
    MakeJ,
    OmarchyMenu,
    CargoRelease,
    LnS,
    SystemctlEnable,
    Cpupower,
    Rsync,
    HyprctlReload,
    NpmVersionPatch,
    RmRf,
    CpF,
    DuSh,
    AcceptEula,
    Strip,
    MvDiamonds,
    MvClubs,
    MvHearts,
    YayS,
    MvSpades,
}

impl Command {
    pub const ALL: [Command; 22] = [
        Command::UpArrow,
        Command::NpmTest,
        Command::ThemeInstall,
        Command::MakeJ,
        Command::OmarchyMenu,
        Command::CargoRelease,
        Command::LnS,
        Command::SystemctlEnable,
        Command::Cpupower,
        Command::Rsync,
        Command::HyprctlReload,
        Command::NpmVersionPatch,
        Command::RmRf,
        Command::CpF,
        Command::DuSh,
        Command::AcceptEula,
        Command::Strip,
        Command::MvDiamonds,
        Command::MvClubs,
        Command::MvHearts,
        Command::YayS,
        Command::MvSpades,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Command::UpArrow => "Up Arrow",
            Command::NpmTest => "npm test",
            Command::ThemeInstall => "omarchy-theme-install",
            Command::MakeJ => "make -j",
            Command::OmarchyMenu => "omarchy-menu",
            Command::CargoRelease => "cargo build --release",
            Command::LnS => "ln -s",
            Command::SystemctlEnable => "systemctl enable",
            Command::Cpupower => "cpupower",
            Command::Rsync => "rsync ~/wallet",
            Command::HyprctlReload => "hyprctl reload",
            Command::NpmVersionPatch => "npm version patch",
            Command::RmRf => "rm -rf",
            Command::CpF => "cp -f",
            Command::DuSh => "du -sh ~/jokers",
            Command::AcceptEula => "Accept EULA",
            Command::Strip => "strip",
            Command::MvDiamonds => "mv * ♦/",
            Command::MvClubs => "mv * ♣/",
            Command::MvHearts => "mv * ♥/",
            Command::YayS => "yay -S",
            Command::MvSpades => "mv * ♠/",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Command::UpArrow => "Creates the last command or theme used (not Up Arrow)",
            Command::NpmTest => "Up to 2 selected cards become Flaky Test",
            Command::ThemeInstall => "Creates up to 2 random themes",
            Command::MakeJ => "Up to 2 selected cards become Multithreaded",
            Command::OmarchyMenu => "Creates up to 2 random commands",
            Command::CargoRelease => "Up to 2 selected cards become Optimized",
            Command::LnS => "1 selected card becomes a Symlink",
            Command::SystemctlEnable => "1 selected card becomes a Daemon",
            Command::Cpupower => "1 selected card becomes Overclocked",
            Command::Rsync => "Doubles your money (+$20 at most)",
            Command::HyprctlReload => {
                "1 in 4 chance to give a random joker Blur, Shadow or Animated"
            }
            Command::NpmVersionPatch => "Raises the rank of up to 2 selected cards by one",
            Command::RmRf => "Destroys up to 2 selected cards",
            Command::CpF => "Of 2 selected cards, the left one becomes a copy of the right one",
            Command::DuSh => "Gives the total sell value of your jokers ($50 at most)",
            Command::AcceptEula => "1 selected card becomes Enterprise",
            Command::Strip => "1 selected card becomes a Binary Blob",
            Command::MvDiamonds => "Up to 3 selected cards become diamonds",
            Command::MvClubs => "Up to 3 selected cards become clubs",
            Command::MvHearts => "Up to 3 selected cards become hearts",
            Command::YayS => "Creates a random joker (needs a free slot)",
            Command::MvSpades => "Up to 3 selected cards become spades",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Command::UpArrow => "History is a flat circle.",
            Command::NpmTest => "Works on the second run.",
            Command::ThemeInstall => "The gallery is bigger than you think.",
            Command::MakeJ => "-j$(nproc), obviously.",
            Command::OmarchyMenu => "Super+Alt+Space: everything's in there.",
            Command::CargoRelease => "Ten minutes to save ten milliseconds.",
            Command::LnS => "Two paths, one heart.",
            Command::SystemctlEnable => "Starts at boot. Forever.",
            Command::Cpupower => "Performance governor. Fans: on.",
            Command::Rsync => "Incremental backups of your money.",
            Command::HyprctlReload => "Nope. Nope. Nope. Oh, nice blur!",
            Command::NpmVersionPatch => "1.0.0 → 1.0.1, ship it.",
            Command::RmRf => "No undo. No regrets.",
            Command::CpF => "Overwrite without asking.",
            Command::DuSh => "Worth more than you think.",
            Command::AcceptEula => "Nobody reads it. Everybody signs it.",
            Command::Strip => "Debug symbols? Never heard of them.",
            Command::MvDiamonds => "Shiny new folder.",
            Command::MvClubs => "Members only.",
            Command::MvHearts => "Moving in with love.",
            Command::YayS => "Straight from the AUR. What could go wrong?",
            Command::MvSpades => "Dig a little deeper.",
        }
    }

    /// How many cards of the hand it needs selected (none for the others).
    pub fn targets(self) -> Option<(usize, usize)> {
        match self {
            Command::NpmTest
            | Command::MakeJ
            | Command::CargoRelease
            | Command::NpmVersionPatch
            | Command::RmRf => Some((1, 2)),
            Command::LnS
            | Command::SystemctlEnable
            | Command::Cpupower
            | Command::AcceptEula
            | Command::Strip => Some((1, 1)),
            Command::CpF => Some((2, 2)),
            Command::MvDiamonds | Command::MvClubs | Command::MvHearts | Command::MvSpades => {
                Some((1, 3))
            }
            _ => None,
        }
    }
}

/// A card that sits in the clipboard until used.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", content = "id", rename_all = "snake_case")]
pub enum Consumable {
    Theme(Theme),
    Command(Command),
}

impl Consumable {
    pub fn name(self) -> &'static str {
        match self {
            Consumable::Theme(t) => t.name(),
            Consumable::Command(c) => c.name(),
        }
    }

    pub fn description(self) -> String {
        match self {
            Consumable::Theme(t) => t.description(),
            Consumable::Command(c) => c.description().to_string(),
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Consumable::Theme(t) => t.quip(),
            Consumable::Command(c) => c.quip(),
        }
    }

    /// Shop price, before discounts.
    pub fn price(self) -> u32 {
        match self {
            Consumable::Theme(_) | Consumable::Command(_) => 3,
        }
    }

    /// Cards of the hand to select before using it.
    pub fn targets(self) -> Option<(usize, usize)> {
        match self {
            Consumable::Command(c) => c.targets(),
            Consumable::Theme(_) => None,
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
    fn commands_are_listed_once_with_short_quips() {
        for command in Command::ALL {
            assert_eq!(Command::ALL.iter().filter(|&&c| c == command).count(), 1);
            assert!(command.quip().chars().count() <= 50, "{}", command.quip());
        }
        assert_eq!(Command::CpF.targets(), Some((2, 2)));
        assert_eq!(Command::Rsync.targets(), None);
    }

    #[test]
    fn consumables_are_tagged_in_json() {
        let json = serde_json::to_string(&Consumable::Theme(Theme::RosePine)).unwrap();
        assert_eq!(json, r#"{"kind":"theme","id":"rose_pine"}"#);
    }
}
