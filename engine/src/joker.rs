//! Jokers: what each one does, how rare it is, the editions that decorate it,
//! and the state an owned joker keeps between hands.

use crate::card::{ACE, Card, Suit};
use crate::hand::{Evaluated, HandKind};
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

/// Hyprland decorations on a joker (and, later, on cards).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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
            Edition::Transparent => "+1 joker slot",
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
}

/// What a joker sees of the hand being scored.
pub struct HandContext<'a> {
    pub played: &'a [Card],
    pub evaluated: &'a Evaluated,
    /// Times this hand type was played this game, this one included.
    pub times_played: u32,
}

fn is_odd(card: Card) -> bool {
    card.rank == ACE || (card.rank <= 9 && card.rank % 2 == 1)
}

fn is_even(card: Card) -> bool {
    card.rank <= 10 && card.rank % 2 == 0
}

impl Joker {
    pub const ALL: [Joker; 52] = [
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
        }
    }

    pub fn rarity(self) -> Rarity {
        match self {
            Joker::SuperKey
            | Joker::NoBuild
            | Joker::Heartbleed
            | Joker::Recursion
            | Joker::WatchMode
            | Joker::Mvp => Rarity::Extra,
            Joker::Hotwire
            | Joker::CodeReview
            | Joker::QuadCore
            | Joker::GitBisect
            | Joker::Monorepo
            | Joker::BenevolentDictator
            | Joker::OrphanCleanup
            | Joker::Fork => Rarity::Aur,
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
            | Joker::UndoHistory => 4,
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
            | Joker::BetaRelease => 5,
            Joker::SuperKey | Joker::Uptime | Joker::Sponsorware | Joker::WatchMode => 6,
            Joker::NoBuild | Joker::Heartbleed | Joker::Mvp => 7,
            Joker::Hotwire
            | Joker::CodeReview
            | Joker::QuadCore
            | Joker::GitBisect
            | Joker::Monorepo
            | Joker::Recursion
            | Joker::BenevolentDictator => 8,
            Joker::OrphanCleanup => 9,
            Joker::Fork => 10,
        }
    }

    /// Whether Fork can copy it: jokers that change the rules, or only grow,
    /// have nothing to copy.
    pub fn copyable(self) -> bool {
        !matches!(
            self,
            Joker::VintageThinkpad | Joker::Mvp | Joker::Ultrawide | Joker::UndoHistory
        )
    }

    /// Starting value of the joker's counter, for jokers that grow or shrink.
    fn initial_counter(self) -> f64 {
        match self {
            Joker::FreeTrial => 100.0,
            Joker::OrphanCleanup => 1.0,
            _ => 0.0,
        }
    }

    /// Triggered for each scoring card, right after its chips.
    pub fn on_card_scored(self, card: Card, rng: &mut Rng) -> Vec<Effect> {
        let face = card.is_face();
        match self {
            Joker::RubyGem if card.suit == Suit::Diamonds => vec![Effect::Mult(2.0)],
            Joker::FanMail if card.suit == Suit::Hearts => vec![Effect::Chips(15)],
            Joker::RailsWorld if face => vec![Effect::Chips(20)],
            Joker::SuperKey => vec![Effect::Mult(2.0)],
            Joker::OpenSourceLove if card.suit == Suit::Hearts => vec![Effect::Mult(3.0)],
            Joker::RabbitHole if card.suit == Suit::Spades => vec![Effect::Mult(3.0)],
            Joker::LinuxUserGroup if card.suit == Suit::Clubs => vec![Effect::Mult(3.0)],
            Joker::Heartbleed if card.suit == Suit::Hearts && rng.chance(2) => {
                vec![Effect::XMult(1.5)]
            }
            Joker::TwoSpaces if is_even(card) => vec![Effect::Mult(4.0)],
            Joker::OffByOne if is_odd(card) => vec![Effect::Chips(31)],
            Joker::Recursion if matches!(card.rank, ACE | 2 | 3 | 5 | 8) => vec![Effect::Mult(8.0)],
            Joker::ArchWiki if card.rank == ACE => vec![Effect::Chips(20), Effect::Mult(4.0)],
            Joker::ConsultingGig if face && rng.chance(2) => vec![Effect::Money(2)],
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
            Joker::LowBattery if lowest => vec![Effect::Mult(2.0 * card.chips() as f64)],
            Joker::BenevolentDictator if card.rank == 13 => vec![Effect::XMult(1.5)],
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
            Joker::Uptime | Joker::GreenBuild if counter > 0.0 => Effect::Mult(counter),
            Joker::FreeTrial | Joker::CiRunner if counter > 0.0 => Effect::Chips(counter as u64),
            Joker::MuscleMemory if hand.times_played > 0 => Effect::Mult(hand.times_played as f64),
            Joker::OrphanCleanup if counter > 1.0 => Effect::XMult(counter),
            Joker::Segfault => Effect::Mult(rng.below(24) as f64),
            Joker::BetaRelease => Effect::Mult(15.0),
            Joker::StableRelease => Effect::XMult(3.0),
            _ => return Vec::new(),
        };
        vec![effect]
    }

    /// Money earned at the end of a won blind.
    pub fn blind_money(self, discards_left: u32, discards_used: u32) -> u32 {
        match self {
            Joker::CloudExit => discards_left,
            Joker::Sponsorware => 4,
            Joker::DataHoarder if discards_used == 0 => 2 * discards_left,
            _ => 0,
        }
    }
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

    /// What the joker currently adds, for jokers that grow or shrink.
    pub fn current(&self) -> Option<String> {
        let c = self.counter;
        match self.joker {
            Joker::Uptime | Joker::GreenBuild => Some(format!("Currently +{c} mult")),
            Joker::FreeTrial | Joker::CiRunner => Some(format!("Currently +{c} chips")),
            Joker::OrphanCleanup => Some(format!("Currently ×{c} mult")),
            _ => None,
        }
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
        assert_eq!(cleanup.current().as_deref(), Some("Currently ×1.25 mult"));
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
