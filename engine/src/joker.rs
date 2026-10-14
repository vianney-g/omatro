//! Jokers: what each one does, how rare it is, the editions that decorate it,
//! and the state an owned joker keeps between hands.

use crate::card::{ACE, Card, Suit};
use crate::hand::{Evaluated, HandKind};
pub use crate::modifier::Edition;
use crate::modifier::Enhancement;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

/// How often a joker shows up in the shop, named after Arch's repositories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    Core,
    Extra,
    Aur,
    Legendary,
}

impl Rarity {
    pub const ALL: [Rarity; 4] = [Rarity::Core, Rarity::Extra, Rarity::Aur, Rarity::Legendary];

    pub fn name(self) -> &'static str {
        match self {
            Rarity::Core => "core",
            Rarity::Extra => "extra",
            Rarity::Aur => "AUR",
            Rarity::Legendary => "Legendary",
        }
    }

    /// Chance for a shop slot to hold a joker of this rarity.
    pub fn odds(self) -> f64 {
        match self {
            Rarity::Core => 0.70,
            Rarity::Extra => 0.25,
            Rarity::Aur => 0.05,
            Rarity::Legendary => 0.0,
        }
    }
}

/// Effect of a trigger on the running score.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    Chips(u64),
    Mult(f64),
    XMult(f64),
    /// Money, earned right away.
    Money(u32),
    /// The card scores once more.
    Again,
}

impl Effect {
    pub fn apply(self, chips: &mut u64, mult: &mut f64) {
        match self {
            Effect::Chips(c) => *chips += c,
            Effect::Mult(m) => *mult += m,
            Effect::XMult(x) => *mult *= x,
            Effect::Money(_) | Effect::Again => {}
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    PairProgramming,
    DualMonitors,
    ThreeWayMerge,
    ColorScheme,
    RubberDuck,
    SplitKeyboard,
    TripleBoot,
    Pipeline,
    DarkMode,
    CodeReview,
    QuadCore,
    GitBisect,
    Monorepo,
    OpenSourceLove,
    RabbitHole,
    LinuxUserGroup,
    Heartbleed,
    TwoSpaces,
    OffByOne,
    Recursion,
    ArchWiki,
    LowBattery,
    BenevolentDictator,
    Uptime,
    GreenBuild,
    FreeTrial,
    MuscleMemory,
    CiRunner,
    OrphanCleanup,
    Sponsorware,
    VintageThinkpad,
    DataHoarder,
    ConsultingGig,
    Segfault,
    BetaRelease,
    StableRelease,
    BangBang,
    WatchMode,
    Fork,
    Mvp,
    Ultrawide,
    UndoHistory,
    ThemeCollector,
    Unixporn,
    FreeAsInBeer,
    AutoUpdate,
    ConfigReload,
    ClosedSource,
    EnterpriseSales,
    GarbageCollector,
    Systemd,
    BlobStorage,
    SupportContract,
    FlakyCi,
    RmaDepartment,
    SeniorEngineer,
    IncrementalBuild,
    CodeSigning,
    CronJob,
    AliasCollection,
    ShellFu,
    VimGolf,
    FreeTier,
    Man8,
}

/// What a joker sees of the hand being scored.
pub struct HandContext<'a> {
    pub played: &'a [Card],
    pub evaluated: &'a Evaluated,
    /// Times this hand type was played this game, this one included.
    pub times_played: u32,
    /// The whole deck.
    pub deck: &'a [Card],
    /// Flaky Test cards that triggered in this hand.
    pub lucky_hits: u32,
}

fn count(deck: &[Card], enhancement: Enhancement) -> usize {
    deck.iter().filter(|c| c.is(enhancement)).count()
}

/// Cards of the deck with an enhancement (Senior Engineer).
pub fn enhanced_cards(deck: &[Card]) -> usize {
    deck.iter().filter(|c| c.enhancement.is_some()).count()
}

/// Enhanced cards Senior Engineer needs.
pub const SENIOR_THRESHOLD: usize = 16;

fn is_odd(card: Card) -> bool {
    card.has_rank() && (card.rank == ACE || (card.rank <= 9 && card.rank % 2 == 1))
}

fn is_even(card: Card) -> bool {
    card.has_rank() && card.rank <= 10 && card.rank % 2 == 0
}

impl Joker {
    pub const ALL: [Joker; 74] = [
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
        Joker::PairProgramming,
        Joker::DualMonitors,
        Joker::ThreeWayMerge,
        Joker::ColorScheme,
        Joker::RubberDuck,
        Joker::SplitKeyboard,
        Joker::TripleBoot,
        Joker::Pipeline,
        Joker::DarkMode,
        Joker::CodeReview,
        Joker::QuadCore,
        Joker::GitBisect,
        Joker::Monorepo,
        Joker::OpenSourceLove,
        Joker::RabbitHole,
        Joker::LinuxUserGroup,
        Joker::Heartbleed,
        Joker::TwoSpaces,
        Joker::OffByOne,
        Joker::Recursion,
        Joker::ArchWiki,
        Joker::LowBattery,
        Joker::BenevolentDictator,
        Joker::Uptime,
        Joker::GreenBuild,
        Joker::FreeTrial,
        Joker::MuscleMemory,
        Joker::CiRunner,
        Joker::OrphanCleanup,
        Joker::Sponsorware,
        Joker::VintageThinkpad,
        Joker::DataHoarder,
        Joker::ConsultingGig,
        Joker::Segfault,
        Joker::BetaRelease,
        Joker::StableRelease,
        Joker::BangBang,
        Joker::WatchMode,
        Joker::Fork,
        Joker::Mvp,
        Joker::Ultrawide,
        Joker::UndoHistory,
        Joker::ThemeCollector,
        Joker::Unixporn,
        Joker::FreeAsInBeer,
        Joker::AutoUpdate,
        Joker::ConfigReload,
        Joker::ClosedSource,
        Joker::EnterpriseSales,
        Joker::GarbageCollector,
        Joker::Systemd,
        Joker::BlobStorage,
        Joker::SupportContract,
        Joker::FlakyCi,
        Joker::RmaDepartment,
        Joker::SeniorEngineer,
        Joker::IncrementalBuild,
        Joker::CodeSigning,
        Joker::CronJob,
        Joker::AliasCollection,
        Joker::ShellFu,
        Joker::VimGolf,
        Joker::FreeTier,
        Joker::Man8,
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
            Joker::PairProgramming => "Pair Programming",
            Joker::DualMonitors => "Dual Monitors",
            Joker::ThreeWayMerge => "Three-Way Merge",
            Joker::ColorScheme => "Color Scheme",
            Joker::RubberDuck => "Rubber Duck",
            Joker::SplitKeyboard => "Split Keyboard",
            Joker::TripleBoot => "Triple Boot",
            Joker::Pipeline => "Pipeline",
            Joker::DarkMode => "Dark Mode",
            Joker::CodeReview => "Code Review",
            Joker::QuadCore => "Quad Core",
            Joker::GitBisect => "Git Bisect",
            Joker::Monorepo => "Monorepo",
            Joker::OpenSourceLove => "Open Source Love",
            Joker::RabbitHole => "Rabbit Hole",
            Joker::LinuxUserGroup => "Linux User Group",
            Joker::Heartbleed => "Heartbleed",
            Joker::TwoSpaces => "Two Spaces",
            Joker::OffByOne => "Off By One",
            Joker::Recursion => "Recursion",
            Joker::ArchWiki => "Arch Wiki",
            Joker::LowBattery => "Low Battery",
            Joker::BenevolentDictator => "Benevolent Dictator",
            Joker::Uptime => "Uptime",
            Joker::GreenBuild => "Green Build",
            Joker::FreeTrial => "Free Trial",
            Joker::MuscleMemory => "Muscle Memory",
            Joker::CiRunner => "CI Runner",
            Joker::OrphanCleanup => "Orphan Cleanup",
            Joker::Sponsorware => "Sponsorware",
            Joker::VintageThinkpad => "Vintage ThinkPad",
            Joker::DataHoarder => "Data Hoarder",
            Joker::ConsultingGig => "Consulting Gig",
            Joker::Segfault => "Segfault",
            Joker::BetaRelease => "Beta Release",
            Joker::StableRelease => "Stable Release",
            Joker::BangBang => "Bang Bang",
            Joker::WatchMode => "Watch Mode",
            Joker::Fork => "Fork",
            Joker::Mvp => "MVP",
            Joker::Ultrawide => "Ultrawide",
            Joker::UndoHistory => "Undo History",
            Joker::ThemeCollector => "Theme Collector",
            Joker::Unixporn => "r/unixporn",
            Joker::FreeAsInBeer => "Free as in Beer",
            Joker::AutoUpdate => "Auto Update",
            Joker::ConfigReload => "Config Reload",
            Joker::ClosedSource => "Closed Source",
            Joker::EnterpriseSales => "Enterprise Sales",
            Joker::GarbageCollector => "Garbage Collector",
            Joker::Systemd => "systemd",
            Joker::BlobStorage => "Blob Storage",
            Joker::SupportContract => "Support Contract",
            Joker::FlakyCi => "Flaky CI",
            Joker::RmaDepartment => "RMA Department",
            Joker::SeniorEngineer => "Senior Engineer",
            Joker::IncrementalBuild => "Incremental Build",
            Joker::CodeSigning => "Code Signing",
            Joker::CronJob => "Cron Job",
            Joker::AliasCollection => "Alias Collection",
            Joker::ShellFu => "Shell Fu",
            Joker::VimGolf => "Vim Golf",
            Joker::FreeTier => "Free Tier",
            Joker::Man8 => "man 8",
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
            Joker::PairProgramming => "+8 mult if the hand contains a Pair",
            Joker::DualMonitors => "+10 mult if the hand contains a Two Pair",
            Joker::ThreeWayMerge => "+12 mult if the hand contains a Three of a Kind",
            Joker::ColorScheme => "+10 mult if the hand contains a Flush",
            Joker::RubberDuck => "+50 chips if the hand contains a Pair",
            Joker::SplitKeyboard => "+80 chips if the hand contains a Two Pair",
            Joker::TripleBoot => "+100 chips if the hand contains a Three of a Kind",
            Joker::Pipeline => "+100 chips if the hand contains a Straight",
            Joker::DarkMode => "+80 chips if the hand contains a Flush",
            Joker::CodeReview => "×2 mult if the hand contains a Pair",
            Joker::QuadCore => "×4 mult if the hand contains a Four of a Kind",
            Joker::GitBisect => "×3 mult if the hand contains a Straight",
            Joker::Monorepo => "×2 mult if the hand contains a Flush",
            Joker::OpenSourceLove => "+3 mult for each scoring heart",
            Joker::RabbitHole => "+3 mult for each scoring spade",
            Joker::LinuxUserGroup => "+3 mult for each scoring club",
            Joker::Heartbleed => "1 in 2 chance of ×1.5 mult for each scoring heart",
            Joker::TwoSpaces => "+4 mult for each scoring even card (10, 8, 6, 4, 2)",
            Joker::OffByOne => "+31 chips for each scoring odd card (A, 9, 7, 5, 3)",
            Joker::Recursion => "+8 mult for each scoring A, 2, 3, 5 or 8",
            Joker::ArchWiki => "+20 chips and +4 mult for each scoring ace",
            Joker::LowBattery => "Adds twice the rank of the lowest card held in hand to mult",
            Joker::BenevolentDictator => "×1.5 mult for each king held in hand",
            Joker::Uptime => "+1 mult per hand in a row without a scoring face card",
            Joker::GreenBuild => "+1 mult per hand played, −1 mult per discard",
            Joker::FreeTrial => "+100 chips, −5 chips per hand played",
            Joker::MuscleMemory => "Adds the number of times the hand type was played to mult",
            Joker::CiRunner => "Gains +15 chips when the hand contains a Straight",
            Joker::OrphanCleanup => "Gains ×0.25 mult per card sold; resets after a boss",
            Joker::Sponsorware => "+$4 at the end of each blind",
            Joker::VintageThinkpad => "Sell value rises by $3 at the end of each blind",
            Joker::DataHoarder => "+$2 per discard left, if no discard was used this blind",
            Joker::ConsultingGig => "1 in 2 chance of +$2 for each scoring face card",
            Joker::Segfault => "+0 to +23 mult, at random",
            Joker::BetaRelease => "+15 mult; 1 in 6 chance to break at the end of each blind",
            Joker::StableRelease => "×3 mult; 1 in 1000 chance to break at the end of each blind",
            Joker::BangBang => "Retriggers the first scoring card 2 more times",
            Joker::WatchMode => "Retriggers every scoring face card",
            Joker::Fork => "Copies the ability of the joker on its right",
            Joker::Mvp => "Flushes and Straights can be made with 4 cards",
            Joker::Ultrawide => "+1 hand size",
            Joker::UndoHistory => "+1 discard each blind",
            Joker::ThemeCollector => "Gains ×0.1 mult per theme applied",
            Joker::Unixporn => "+$1 at the end of each blind per unique theme applied",
            Joker::FreeAsInBeer => "Themes in the shop cost $0",
            Joker::AutoUpdate => "1 in 4 chance to level up the played hand type",
            Joker::ConfigReload => "Levels up the hand type of the first discard of each blind",
            Joker::ClosedSource => "Adds a Binary Blob card to the deck when a blind starts",
            Joker::EnterpriseSales => "Scoring face cards become Enterprise cards",
            Joker::GarbageCollector => {
                "Gains ×0.1 mult per enhanced card played, and removes the enhancement"
            }
            Joker::Systemd => "×0.2 mult per Daemon card in the deck",
            Joker::BlobStorage => "+25 chips per Binary Blob card in the deck",
            Joker::SupportContract => "Enterprise cards give +$4 when scored",
            Joker::FlakyCi => "Gains ×0.25 mult each time a Flaky Test card triggers",
            Joker::RmaDepartment => "Gains ×0.75 mult per Overclocked card that breaks",
            Joker::SeniorEngineer => "×3 mult if the deck holds at least 16 enhanced cards",
            Joker::IncrementalBuild => "Every played card permanently gains +5 chips",
            Joker::CodeSigning => {
                "When a blind starts, adds a random card with a random seal to the hand"
            }
            Joker::CronJob => "Retriggers the effects of cards held in hand",
            Joker::AliasCollection => "Creates a command when a blind starts (if there is room)",
            Joker::ShellFu => "+1 mult per command used this game",
            Joker::VimGolf => "Creates a command if the hand contains an ace and a Straight",
            Joker::FreeTier => "Creates a command when a hand is played with $4 or less",
            Joker::Man8 => "1 in 4 chance per scoring 8 to create a command",
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
            Joker::PairProgramming => "Two keyboards, one bug.",
            Joker::DualMonitors => "Twice the screens, twice the tabs.",
            Joker::ThreeWayMerge => "Ours, theirs, and somehow a third.",
            Joker::ColorScheme => "Every window, the same theme.",
            Joker::RubberDuck => "Explains your bug back to you.",
            Joker::SplitKeyboard => "Two halves, both ergonomic.",
            Joker::TripleBoot => "Arch, Arch, and also Arch.",
            Joker::Pipeline => "cat | grep | sort | uniq -c",
            Joker::DarkMode => "Light mode is a bug report.",
            Joker::CodeReview => "Looks good to me. Ship it.",
            Joker::QuadCore => "All cores busy compiling Rust.",
            Joker::GitBisect => "Good, bad, good, bad… found it.",
            Joker::Monorepo => "Everything in one place. Even you.",
            Joker::OpenSourceLove => "Made with ♥ and too much coffee.",
            Joker::RabbitHole => "Just one more config tweak.",
            Joker::LinuxUserGroup => "Pizza, slides, and a broken projector.",
            Joker::Heartbleed => "Your heart leaks, 64 KB at a time.",
            Joker::TwoSpaces => "Not tabs. Never tabs.",
            Joker::OffByOne => "Arrays start at one. Fight me.",
            Joker::Recursion => "To understand recursion, see Recursion.",
            Joker::ArchWiki => "The answer was on the Arch Wiki all along.",
            Joker::LowBattery => "Still at 2%. Still going.",
            Joker::BenevolentDictator => "For life. Or until the fork.",
            Joker::Uptime => "847 days without a reboot.",
            Joker::GreenBuild => "All tests pass. Don't touch anything.",
            Joker::FreeTrial => "Enjoy it while it lasts.",
            Joker::MuscleMemory => "Your fingers type it before you think.",
            Joker::CiRunner => "Green checks, one after another.",
            Joker::OrphanCleanup => "pacman -Rns $(pacman -Qdtq)",
            Joker::Sponsorware => "Thank you for your sponsorship.",
            Joker::VintageThinkpad => "Appreciating asset. Allegedly.",
            Joker::DataHoarder => "Never delete anything. Ever.",
            Joker::ConsultingGig => "Faces pay. Sometimes.",
            Joker::Segfault => "Undefined behavior, defined fun.",
            Joker::BetaRelease => "Works great until it doesn't.",
            Joker::StableRelease => "Boring. Reliable. Beautiful.",
            Joker::BangBang => "!! — run it again, louder.",
            Joker::WatchMode => "Every face, hot-reloaded.",
            Joker::Fork => "Same features. MIT licensed.",
            Joker::Mvp => "Minimum viable flush.",
            Joker::Ultrawide => "More screen, more cards.",
            Joker::UndoHistory => "Ctrl+Z, but for cards.",
            Joker::ThemeCollector => "37 themes installed, 1 in use.",
            Joker::Unixporn => "Karma, but you can spend it.",
            Joker::FreeAsInBeer => "Also free as in freedom.",
            Joker::AutoUpdate => "pacman -Syu, every hand.",
            Joker::ConfigReload => "Super+Shift+R: everything, but fresher.",
            Joker::ClosedSource => "Can't read it. Can't remove it.",
            Joker::EnterpriseSales => "Every face, upsold.",
            Joker::GarbageCollector => "Frees memory. And your upgrades.",
            Joker::Systemd => "One daemon to rule them all.",
            Joker::BlobStorage => "S3 buckets all the way down.",
            Joker::SupportContract => "Billed per incident.",
            Joker::FlakyCi => "Green on the third try.",
            Joker::RmaDepartment => "We've seen worse. Probably.",
            Joker::SeniorEngineer => "Has seen every config.",
            Joker::IncrementalBuild => "A little faster every time.",
            Joker::CodeSigning => "Signed, sealed, delivered.",
            Joker::CronJob => "Runs again at midnight. And again.",
            Joker::AliasCollection => "alias ll='ls -la'. And 400 more.",
            Joker::ShellFu => "Pipes, loops, one-liners.",
            Joker::VimGolf => "Done in four keystrokes.",
            Joker::FreeTier => "Rate-limited, but it works.",
            Joker::Man8 => "System administration commands.",
        }
    }

    pub fn rarity(self) -> Rarity {
        match self {
            Joker::SuperKey
            | Joker::NoBuild
            | Joker::Heartbleed
            | Joker::Recursion
            | Joker::WatchMode
            | Joker::Mvp
            | Joker::ThemeCollector
            | Joker::Unixporn
            | Joker::FreeAsInBeer
            | Joker::ConfigReload
            | Joker::ClosedSource
            | Joker::EnterpriseSales
            | Joker::GarbageCollector
            | Joker::Systemd
            | Joker::BlobStorage
            | Joker::FlakyCi
            | Joker::RmaDepartment
            | Joker::IncrementalBuild
            | Joker::CodeSigning
            | Joker::CronJob
            | Joker::AliasCollection => Rarity::Extra,
            Joker::Hotwire
            | Joker::CodeReview
            | Joker::QuadCore
            | Joker::GitBisect
            | Joker::Monorepo
            | Joker::BenevolentDictator
            | Joker::OrphanCleanup
            | Joker::Fork
            | Joker::SeniorEngineer => Rarity::Aur,
            _ => Rarity::Core,
        }
    }

    /// Shop price, before any edition.
    pub fn price(self) -> u32 {
        match self {
            Joker::PairProgramming | Joker::RubberDuck => 3,
            Joker::Dotfiles
            | Joker::Omakase
            | Joker::DualMonitors
            | Joker::ThreeWayMerge
            | Joker::ColorScheme
            | Joker::SplitKeyboard
            | Joker::TripleBoot
            | Joker::Pipeline
            | Joker::DarkMode
            | Joker::TwoSpaces
            | Joker::OffByOne
            | Joker::ArchWiki
            | Joker::GreenBuild
            | Joker::VintageThinkpad
            | Joker::DataHoarder
            | Joker::ConsultingGig
            | Joker::Segfault
            | Joker::StableRelease
            | Joker::BangBang
            | Joker::Ultrawide
            | Joker::UndoHistory
            | Joker::VimGolf => 4,
            Joker::RubyGem
            | Joker::FanMail
            | Joker::RailsWorld
            | Joker::TilingManager
            | Joker::CloudExit
            | Joker::OpenSourceLove
            | Joker::RabbitHole
            | Joker::LinuxUserGroup
            | Joker::LowBattery
            | Joker::FreeTrial
            | Joker::MuscleMemory
            | Joker::CiRunner
            | Joker::BetaRelease
            | Joker::AutoUpdate
            | Joker::SupportContract
            | Joker::IncrementalBuild
            | Joker::CronJob
            | Joker::FreeTier
            | Joker::Man8 => 5,
            Joker::SuperKey
            | Joker::Uptime
            | Joker::Sponsorware
            | Joker::WatchMode
            | Joker::ThemeCollector
            | Joker::Unixporn
            | Joker::BlobStorage
            | Joker::FlakyCi
            | Joker::RmaDepartment
            | Joker::CodeSigning
            | Joker::AliasCollection
            | Joker::ShellFu => 6,
            Joker::NoBuild
            | Joker::Heartbleed
            | Joker::Mvp
            | Joker::ClosedSource
            | Joker::EnterpriseSales
            | Joker::GarbageCollector
            | Joker::Systemd
            | Joker::SeniorEngineer => 7,
            Joker::Hotwire
            | Joker::CodeReview
            | Joker::QuadCore
            | Joker::GitBisect
            | Joker::Monorepo
            | Joker::Recursion
            | Joker::BenevolentDictator
            | Joker::FreeAsInBeer
            | Joker::ConfigReload => 8,
            Joker::OrphanCleanup => 9,
            Joker::Fork => 10,
        }
    }

    /// Whether Fork can copy it: jokers that change the rules, or only grow,
    /// have nothing to copy.
    pub fn copyable(self) -> bool {
        !matches!(
            self,
            Joker::VintageThinkpad
                | Joker::Mvp
                | Joker::Ultrawide
                | Joker::UndoHistory
                | Joker::FreeAsInBeer
                | Joker::EnterpriseSales
        )
    }

    /// Starting value of the joker's counter, for jokers that grow or shrink.
    fn initial_counter(self) -> f64 {
        match self {
            Joker::FreeTrial => 100.0,
            Joker::OrphanCleanup
            | Joker::ThemeCollector
            | Joker::GarbageCollector
            | Joker::FlakyCi
            | Joker::RmaDepartment => 1.0,
            _ => 0.0,
        }
    }

    /// Triggered for each scoring card, right after its chips.
    pub fn on_card_scored(self, card: Card, rng: &mut Rng) -> Vec<Effect> {
        let face = card.is_face();
        match self {
            Joker::RubyGem if card.is_suit(Suit::Diamonds) => vec![Effect::Mult(2.0)],
            Joker::FanMail if card.is_suit(Suit::Hearts) => vec![Effect::Chips(15)],
            Joker::RailsWorld if face => vec![Effect::Chips(20)],
            Joker::SuperKey => vec![Effect::Mult(2.0)],
            Joker::OpenSourceLove if card.is_suit(Suit::Hearts) => vec![Effect::Mult(3.0)],
            Joker::RabbitHole if card.is_suit(Suit::Spades) => vec![Effect::Mult(3.0)],
            Joker::LinuxUserGroup if card.is_suit(Suit::Clubs) => vec![Effect::Mult(3.0)],
            Joker::Heartbleed if card.is_suit(Suit::Hearts) && rng.chance(2) => {
                vec![Effect::XMult(1.5)]
            }
            Joker::TwoSpaces if is_even(card) => vec![Effect::Mult(4.0)],
            Joker::OffByOne if is_odd(card) => vec![Effect::Chips(31)],
            Joker::Recursion if card.has_rank() && matches!(card.rank, ACE | 2 | 3 | 5 | 8) => {
                vec![Effect::Mult(8.0)]
            }
            Joker::ArchWiki if card.has_rank() && card.rank == ACE => {
                vec![Effect::Chips(20), Effect::Mult(4.0)]
            }
            Joker::ConsultingGig if face && rng.chance(2) => vec![Effect::Money(2)],
            Joker::SupportContract if card.is(Enhancement::Enterprise) => vec![Effect::Money(4)],
            _ => Vec::new(),
        }
    }

    /// How many more times a scoring card scores (`first`: the first scoring card).
    pub fn retriggers(self, card: Card, first: bool) -> u32 {
        match self {
            Joker::BangBang if first => 2,
            Joker::WatchMode if card.is_face() => 1,
            _ => 0,
        }
    }

    /// Triggered for each card held in hand (`lowest`: the lowest of them).
    pub fn on_held(self, card: Card, lowest: bool) -> Vec<Effect> {
        match self {
            Joker::LowBattery if lowest => vec![Effect::Mult(2.0 * card.rank_chips() as f64)],
            Joker::BenevolentDictator if card.has_rank() && card.rank == 13 => {
                vec![Effect::XMult(1.5)]
            }
            _ => Vec::new(),
        }
    }

    /// Triggered once per hand, after the cards, left to right (`counter`: the
    /// joker's grown value).
    pub fn on_hand(self, counter: f64, hand: &HandContext, rng: &mut Rng) -> Vec<Effect> {
        let kind = hand.evaluated.kind;
        let has = |k| hand.evaluated.contains(k);
        let effect = match self {
            Joker::Dotfiles => Effect::Chips(30),
            Joker::Omakase => Effect::Mult(5.0),
            Joker::TilingManager
                if matches!(kind, HandKind::Straight | HandKind::StraightFlush) =>
            {
                Effect::Mult(12.0)
            }
            Joker::NoBuild if hand.played.len() <= 3 => Effect::XMult(2.0),
            Joker::Hotwire if kind == HandKind::ThreeOfAKind => Effect::XMult(3.0),
            Joker::PairProgramming if has(HandKind::Pair) => Effect::Mult(8.0),
            Joker::DualMonitors if has(HandKind::TwoPair) => Effect::Mult(10.0),
            Joker::ThreeWayMerge if has(HandKind::ThreeOfAKind) => Effect::Mult(12.0),
            Joker::ColorScheme if has(HandKind::Flush) => Effect::Mult(10.0),
            Joker::RubberDuck if has(HandKind::Pair) => Effect::Chips(50),
            Joker::SplitKeyboard if has(HandKind::TwoPair) => Effect::Chips(80),
            Joker::TripleBoot if has(HandKind::ThreeOfAKind) => Effect::Chips(100),
            Joker::Pipeline if has(HandKind::Straight) => Effect::Chips(100),
            Joker::DarkMode if has(HandKind::Flush) => Effect::Chips(80),
            Joker::CodeReview if has(HandKind::Pair) => Effect::XMult(2.0),
            Joker::QuadCore if has(HandKind::FourOfAKind) => Effect::XMult(4.0),
            Joker::GitBisect if has(HandKind::Straight) => Effect::XMult(3.0),
            Joker::Monorepo if has(HandKind::Flush) => Effect::XMult(2.0),
            Joker::Uptime | Joker::GreenBuild | Joker::ShellFu if counter > 0.0 => {
                Effect::Mult(counter)
            }
            Joker::FreeTrial | Joker::CiRunner if counter > 0.0 => Effect::Chips(counter as u64),
            Joker::MuscleMemory if hand.times_played > 0 => Effect::Mult(hand.times_played as f64),
            Joker::OrphanCleanup | Joker::ThemeCollector if counter > 1.0 => Effect::XMult(counter),
            Joker::GarbageCollector | Joker::RmaDepartment if counter > 1.0 => {
                Effect::XMult(counter)
            }
            Joker::FlakyCi if counter + 0.25 * hand.lucky_hits as f64 > 1.0 => {
                Effect::XMult(counter + 0.25 * hand.lucky_hits as f64)
            }
            Joker::Systemd if count(hand.deck, Enhancement::Daemon) > 0 => {
                Effect::XMult(1.0 + 0.2 * count(hand.deck, Enhancement::Daemon) as f64)
            }
            Joker::BlobStorage if count(hand.deck, Enhancement::BinaryBlob) > 0 => {
                Effect::Chips(25 * count(hand.deck, Enhancement::BinaryBlob) as u64)
            }
            Joker::SeniorEngineer if enhanced_cards(hand.deck) >= SENIOR_THRESHOLD => {
                Effect::XMult(3.0)
            }
            Joker::Segfault => Effect::Mult(rng.below(24) as f64),
            Joker::BetaRelease => Effect::Mult(15.0),
            Joker::StableRelease => Effect::XMult(3.0),
            _ => return Vec::new(),
        };
        vec![effect]
    }

    /// Money earned at the end of a won blind.
    pub fn blind_money(self, discards_left: u32, discards_used: u32, unique_themes: u32) -> u32 {
        match self {
            Joker::CloudExit => discards_left,
            Joker::Unixporn => unique_themes,
            Joker::Sponsorware => 4,
            Joker::DataHoarder if discards_used == 0 => 2 * discards_left,
            _ => 0,
        }
    }
}

/// The current value of a growing joker, shown on a badge.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Bonus {
    /// `chips`, `mult`, `x_mult`, `money`, or `progress` towards an effect: the
    /// badge's colour.
    pub kind: &'static str,
    pub text: String,
}

/// Rounds away float noise (1.3 rather than 1.3000000000000003).
fn round(value: f64) -> f64 {
    (value * 100.0).round() / 100.0
}

/// A joker the player owns, with its edition and what it gained so far.
#[derive(Debug, Clone, PartialEq)]
pub struct OwnedJoker {
    pub joker: Joker,
    pub edition: Option<Edition>,
    /// The grown value of jokers that scale (mult, chips or ×mult).
    pub counter: f64,
    /// Added to the sell value (Vintage ThinkPad).
    pub sell_bonus: u32,
}

impl OwnedJoker {
    pub fn new(joker: Joker, edition: Option<Edition>) -> Self {
        OwnedJoker {
            joker,
            edition,
            counter: joker.initial_counter(),
            sell_bonus: 0,
        }
    }

    pub fn price(&self) -> u32 {
        self.joker.price() + self.edition.map_or(0, Edition::price)
    }

    /// Half the price, at least $1, plus what it gained.
    pub fn sell_value(&self) -> u32 {
        (self.price() / 2).max(1) + self.sell_bonus
    }

    /// What a joker that grows, shrinks or depends on the game gives right now,
    /// for its badge. `hand_plays`: times the selected hand type would have been
    /// played, this hand included (none when no hand is selected).
    pub fn bonus(
        &self,
        hand_plays: Option<u32>,
        unique_themes: u32,
        deck: &[Card],
    ) -> Option<Bonus> {
        let c = round(self.counter);
        let daemons = count(deck, Enhancement::Daemon);
        let enhanced = enhanced_cards(deck);
        let (kind, text) = match self.joker {
            Joker::GarbageCollector | Joker::FlakyCi | Joker::RmaDepartment => {
                ("x_mult", format!("×{c} mult"))
            }
            Joker::Systemd => (
                "x_mult",
                format!("×{} mult", round(1.0 + 0.2 * daemons as f64)),
            ),
            Joker::BlobStorage => (
                "chips",
                format!("+{} chips", 25 * count(deck, Enhancement::BinaryBlob)),
            ),
            Joker::SeniorEngineer if enhanced >= SENIOR_THRESHOLD => {
                ("x_mult", "×3 mult".to_string())
            }
            Joker::SeniorEngineer => (
                "progress",
                format!("{enhanced} / {SENIOR_THRESHOLD} enhanced"),
            ),
            Joker::Uptime | Joker::GreenBuild | Joker::ShellFu => ("mult", format!("+{c} mult")),
            Joker::FreeTrial | Joker::CiRunner => ("chips", format!("+{c} chips")),
            Joker::OrphanCleanup | Joker::ThemeCollector => ("x_mult", format!("×{c} mult")),
            Joker::MuscleMemory => ("mult", format!("+{} mult", hand_plays?)),
            Joker::Unixporn => ("money", format!("+${unique_themes} / blind")),
            Joker::VintageThinkpad => ("money", format!("sells ${}", self.sell_value())),
            _ => return None,
        };
        Some(Bonus { kind, text })
    }

    /// Before a hand is scored: jokers that grow with the hands played.
    pub fn before_hand(&mut self, evaluated: &Evaluated, scoring: &[Card]) {
        match self.joker {
            Joker::Uptime if scoring.iter().any(|c| c.is_face()) => self.counter = 0.0,
            Joker::Uptime | Joker::GreenBuild => self.counter += 1.0,
            Joker::CiRunner if evaluated.contains(HandKind::Straight) => self.counter += 15.0,
            _ => {}
        }
    }

    /// After a hand is scored; false when the joker is used up.
    pub fn after_hand(&mut self) -> bool {
        if self.joker == Joker::FreeTrial {
            self.counter -= 5.0;
            return self.counter > 0.0;
        }
        true
    }

    pub fn on_discard(&mut self) {
        if self.joker == Joker::GreenBuild {
            self.counter = (self.counter - 1.0).max(0.0);
        }
    }

    pub fn on_command_used(&mut self) {
        if self.joker == Joker::ShellFu {
            self.counter += 1.0;
        }
    }

    pub fn on_theme_used(&mut self) {
        if self.joker == Joker::ThemeCollector {
            self.counter += 0.1;
        }
    }

    /// Another card was sold.
    pub fn on_card_sold(&mut self) {
        if self.joker == Joker::OrphanCleanup {
            self.counter += 0.25;
        }
    }

    /// At the end of a won blind; false when the joker breaks.
    pub fn end_of_blind(&mut self, boss: bool, rng: &mut Rng) -> bool {
        match self.joker {
            Joker::VintageThinkpad => self.sell_bonus += 3,
            Joker::OrphanCleanup if boss => self.counter = 1.0,
            Joker::BetaRelease => return !rng.chance(6),
            Joker::StableRelease => return !rng.chance(1000),
            _ => {}
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_joker_is_listed_once() {
        for joker in Joker::ALL {
            assert_eq!(Joker::ALL.iter().filter(|&&j| j == joker).count(), 1);
            let owned = OwnedJoker::new(joker, None);
            assert!(owned.sell_value() >= 1 && owned.sell_value() < joker.price());
            // Longer quips overflow the joker card in the interface.
            let quip = joker.quip();
            assert!(!quip.is_empty() && quip.chars().count() <= 50, "{quip}");
        }
    }

    #[test]
    fn editions_add_to_the_price() {
        let owned = OwnedJoker::new(Joker::Fork, Some(Edition::Animated));
        assert_eq!((owned.price(), owned.sell_value()), (15, 7));
        assert!(Edition::ALL.iter().all(|e| e.quip().chars().count() <= 50));
    }

    #[test]
    fn counters_grow_and_shrink() {
        let mut trial = OwnedJoker::new(Joker::FreeTrial, None);
        for _ in 0..19 {
            assert!(trial.after_hand());
        }
        assert!(!trial.after_hand());

        let mut build = OwnedJoker::new(Joker::GreenBuild, None);
        build.on_discard();
        assert_eq!(build.counter, 0.0);

        let mut cleanup = OwnedJoker::new(Joker::OrphanCleanup, None);
        cleanup.on_card_sold();
        assert_eq!(cleanup.bonus(None, 0, &[]).unwrap().text, "×1.25 mult");

        let mut collector = OwnedJoker::new(Joker::ThemeCollector, None);
        assert_eq!(collector.bonus(None, 0, &[]).unwrap().text, "×1 mult");
        for _ in 0..3 {
            collector.on_theme_used();
        }
        assert_eq!(collector.bonus(None, 0, &[]).unwrap().text, "×1.3 mult");
        let memory = OwnedJoker::new(Joker::MuscleMemory, None);
        assert_eq!(memory.bonus(None, 0, &[]), None);
        assert_eq!(memory.bonus(Some(4), 0, &[]).unwrap().text, "+4 mult");
        cleanup.end_of_blind(true, &mut Rng::new(1));
        assert_eq!(cleanup.counter, 1.0);
    }

    #[test]
    fn retriggers_and_held_cards() {
        let king = Card::new(13, Suit::Spades);
        assert_eq!(Joker::BangBang.retriggers(king, true), 2);
        assert_eq!(Joker::BangBang.retriggers(king, false), 0);
        assert_eq!(Joker::WatchMode.retriggers(king, false), 1);
        assert_eq!(Joker::LowBattery.on_held(king, true), [Effect::Mult(20.0)]);
        assert_eq!(
            Joker::BenevolentDictator.on_held(king, false),
            [Effect::XMult(1.5)]
        );
    }
}
