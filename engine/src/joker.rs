//! Jokers: what each one does, how rare it is, the editions that decorate it,
//! and the state an owned joker keeps between hands.

use crate::card::{ACE, Card, Suit};
use crate::hand::{Evaluated, HandKind};
pub use crate::modifier::Edition;
use crate::modifier::Enhancement;
use crate::rng::Rng;
use crate::stake::Stickers;
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
    /// A boss keeps the card from scoring.
    Debuffed,
    /// Hyprland Dwindle: chips and mult are balanced.
    Balance,
}

impl Effect {
    pub fn apply(self, chips: &mut u64, mult: &mut f64) {
        match self {
            Effect::Chips(c) => *chips += c,
            Effect::Mult(m) => *mult += m,
            Effect::XMult(x) => *mult *= x,
            Effect::Money(_) | Effect::Again | Effect::Debuffed | Effect::Balance => {}
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
    MinimalInstall,
    OptionalDependencies,
    Dhh,
    ThePrimeagen,
    MailingList,
    Kill9,
    CopyOnWrite,
    SeedRound,
    AngelInvestor,
    IncidentCommander,
    Rollback,
    SkipIntro,
    DeepDive,
    MeetupPizza,
    GemInstall,
    EmojiReaction,
    Error404,
    BitTwiddling,
    ProfilePicture,
    QueueManager,
    TwoFactorAuth,
    MiddleManagement,
    SplitBrain,
    RainbowPrompt,
    PitchBlack,
    JqFilter,
    FuzzyFinder,
    Identicon,
    BigBangRelease,
    FastForward,
    ChaosMonkey,
    RollingRelease,
    DailyDriver,
    LogRotation,
    BugBounty,
    TodoMd,
    GarageSale,
    Quine,
    Headhunter,
    EmptyWorkspace,
    InboxZero,
    ReleaseTrain,
    LastCommit,
    FridayDeploy,
    WarmStart,
    GridLayout,
    DotfilesBak,
    HipsterStack,
    EspressoShot,
    HypeCycle,
    EnergyDrink,
    SwapFile,
    Bloatware,
    Backspace,
    VerticalMonitor,
    ForcePush,
    MinimalWaybar,
    PluginManager,
    PacmanCache,
    BitRot,
    FreeRam,
    OwnedServers,
    Bootstrapped,
    Homelab,
    PackageMaintainer,
    CloudCredits,
    SemverInflation,
    NineNines,
    HockeyStick,
    CompoundInterest,
    StickerPack,
    RefreshSpam,
    Paru,
    Flatpak,
    BtrfsSnapshot,
    Syncthing,
    Upstream,
    TransitiveDependencies,
    RmRfNodeModules,
    DistroHopping,
    HashCollision,
    Postmortem,
    SingleBinary,
    IntegerOverflow,
}

/// What the jokers that aim at something get at each blind.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Targets {
    /// Rolling Release: the suit that scores ×1.5.
    pub suit: Suit,
    /// Daily Driver: the card (rank, suit) that scores ×2.
    pub card: (u8, Suit),
    /// Log Rotation: the suit to discard.
    pub discard_suit: Suit,
    /// Bug Bounty: the rank to discard.
    pub rank: u8,
    /// TODO.md: the hand type to play.
    pub hand: HandKind,
}

impl Default for Targets {
    fn default() -> Self {
        Targets {
            suit: Suit::Spades,
            card: (ACE, Suit::Spades),
            discard_suit: Suit::Spades,
            rank: ACE,
            hand: HandKind::HighCard,
        }
    }
}

impl Targets {
    pub fn draw(rng: &mut Rng, deck: &[Card]) -> Self {
        let suit = |rng: &mut Rng| Suit::ALL[rng.below(4) as usize];
        let ranked: Vec<&Card> = deck.iter().filter(|c| c.has_rank()).collect();
        let card = match ranked.len() {
            0 => (ACE, Suit::Spades),
            n => {
                let c = ranked[rng.below(n as u64) as usize];
                (c.rank, c.suit)
            }
        };
        let hands: Vec<HandKind> = HandKind::ALL.into_iter().filter(|k| !k.secret()).collect();
        Targets {
            suit: suit(rng),
            card,
            discard_suit: suit(rng),
            rank: 2 + rng.below(13) as u8,
            hand: hands[rng.below(hands.len() as u64) as usize],
        }
    }
}

/// A rank as written on the cards: 2 to 10, J, Q, K, A.
pub fn rank_label(rank: u8) -> String {
    match rank {
        11 => "J".into(),
        12 => "Q".into(),
        13 => "K".into(),
        ACE => "A".into(),
        n => n.to_string(),
    }
}

/// What jokers see of the game, while scoring and for their badges.
#[derive(Default)]
pub struct Board<'a> {
    /// The whole deck.
    pub deck: &'a [Card],
    /// Cards left to draw.
    pub draw_pile: usize,
    /// Size of the deck at the start of the game (Bit Rot).
    pub starting_deck: usize,
    pub jokers: &'a [OwnedJoker],
    pub joker_slots: usize,
    pub money: i64,
    /// Hands left, the one being played included.
    pub hands_left: u32,
    pub discards_left: u32,
    pub unique_themes: u32,
    pub targets: Targets,
    /// Hand types played earlier in this blind.
    pub blind_hands: &'a [HandKind],
}

impl Board<'_> {
    /// Minimal Waybar: empty slots, and itself.
    fn waybar(&self) -> f64 {
        let waybars = self
            .jokers
            .iter()
            .filter(|j| j.joker == Joker::MinimalWaybar)
            .count();
        (self.joker_slots.saturating_sub(self.jokers.len()) + waybars) as f64
    }

    /// Bit Rot: cards the deck lost since the start.
    fn lost_cards(&self) -> usize {
        self.starting_deck.saturating_sub(self.deck.len())
    }

    /// Homelab: sell value of the other jokers.
    fn resale(&self) -> u32 {
        self.jokers
            .iter()
            .filter(|j| j.joker != Joker::Homelab)
            .map(OwnedJoker::sell_value)
            .sum()
    }

    /// Package Maintainer: ×1.5 per extra joker.
    fn maintained(&self) -> f64 {
        let extras = self
            .jokers
            .iter()
            .filter(|j| j.joker.rarity() == Rarity::Extra)
            .count();
        1.5f64.powi(extras as i32)
    }

    fn nines(&self) -> u32 {
        self.deck
            .iter()
            .filter(|c| c.has_rank() && c.rank == 9)
            .count() as u32
    }
}

/// What a joker sees of the hand being scored.
pub struct HandContext<'a> {
    pub played: &'a [Card],
    /// Cards left in hand.
    pub held: &'a [Card],
    pub evaluated: &'a Evaluated,
    /// Times this hand type was played this game, this one included.
    pub times_played: u32,
    /// Flaky Test cards that triggered in this hand.
    pub lucky_hits: u32,
    pub board: &'a Board<'a>,
}

impl HandContext<'_> {
    fn scoring(&self) -> Vec<Card> {
        self.evaluated
            .scoring
            .iter()
            .map(|&i| self.played[i])
            .collect()
    }
}

/// Hash Collision: two cards of the same rank and suit.
fn has_twins(cards: &[Card]) -> bool {
    cards.iter().enumerate().any(|(i, a)| {
        a.has_rank()
            && cards[i + 1..]
                .iter()
                .any(|b| b.has_rank() && b.rank == a.rank && b.suit == a.suit)
    })
}

/// Split Brain: a club and a card of another suit, two different cards.
fn split_brain(cards: &[Card]) -> bool {
    let others = [Suit::Hearts, Suit::Diamonds, Suit::Spades];
    cards.iter().enumerate().any(|(i, a)| {
        a.is_suit(Suit::Clubs)
            && cards
                .iter()
                .enumerate()
                .any(|(j, b)| i != j && others.iter().any(|&s| b.is_suit(s)))
    })
}

/// Rainbow Prompt: each suit covered by a different card.
fn all_suits(cards: &[Card]) -> bool {
    fn cover(cards: &[Card], used: &mut Vec<usize>, suit: usize) -> bool {
        if suit == Suit::ALL.len() {
            return true;
        }
        for i in 0..cards.len() {
            if !used.contains(&i) && cards[i].is_suit(Suit::ALL[suit]) {
                used.push(i);
                if cover(cards, used, suit + 1) {
                    return true;
                }
                used.pop();
            }
        }
        false
    }
    cover(cards, &mut Vec::new(), 0)
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
    pub const ALL: [Joker; 158] = [
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
        Joker::MinimalInstall,
        Joker::OptionalDependencies,
        Joker::Dhh,
        Joker::ThePrimeagen,
        Joker::MailingList,
        Joker::Kill9,
        Joker::CopyOnWrite,
        Joker::SeedRound,
        Joker::AngelInvestor,
        Joker::IncidentCommander,
        Joker::Rollback,
        Joker::SkipIntro,
        Joker::DeepDive,
        Joker::MeetupPizza,
        Joker::GemInstall,
        Joker::EmojiReaction,
        Joker::Error404,
        Joker::BitTwiddling,
        Joker::ProfilePicture,
        Joker::QueueManager,
        Joker::TwoFactorAuth,
        Joker::MiddleManagement,
        Joker::SplitBrain,
        Joker::RainbowPrompt,
        Joker::PitchBlack,
        Joker::JqFilter,
        Joker::FuzzyFinder,
        Joker::Identicon,
        Joker::BigBangRelease,
        Joker::FastForward,
        Joker::ChaosMonkey,
        Joker::RollingRelease,
        Joker::DailyDriver,
        Joker::LogRotation,
        Joker::BugBounty,
        Joker::TodoMd,
        Joker::GarageSale,
        Joker::Quine,
        Joker::Headhunter,
        Joker::EmptyWorkspace,
        Joker::InboxZero,
        Joker::ReleaseTrain,
        Joker::LastCommit,
        Joker::FridayDeploy,
        Joker::WarmStart,
        Joker::GridLayout,
        Joker::DotfilesBak,
        Joker::HipsterStack,
        Joker::EspressoShot,
        Joker::HypeCycle,
        Joker::EnergyDrink,
        Joker::SwapFile,
        Joker::Bloatware,
        Joker::Backspace,
        Joker::VerticalMonitor,
        Joker::ForcePush,
        Joker::MinimalWaybar,
        Joker::PluginManager,
        Joker::PacmanCache,
        Joker::BitRot,
        Joker::FreeRam,
        Joker::OwnedServers,
        Joker::Bootstrapped,
        Joker::Homelab,
        Joker::PackageMaintainer,
        Joker::CloudCredits,
        Joker::SemverInflation,
        Joker::NineNines,
        Joker::HockeyStick,
        Joker::CompoundInterest,
        Joker::StickerPack,
        Joker::RefreshSpam,
        Joker::Paru,
        Joker::Flatpak,
        Joker::BtrfsSnapshot,
        Joker::Syncthing,
        Joker::Upstream,
        Joker::TransitiveDependencies,
        Joker::RmRfNodeModules,
        Joker::DistroHopping,
        Joker::HashCollision,
        Joker::Postmortem,
        Joker::SingleBinary,
        Joker::IntegerOverflow,
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
            Joker::MinimalInstall => "Minimal Install",
            Joker::OptionalDependencies => "Optional Dependencies",
            Joker::Dhh => "DHH",
            Joker::ThePrimeagen => "ThePrimeagen",
            Joker::MailingList => "Mailing List",
            Joker::Kill9 => "kill -9",
            Joker::CopyOnWrite => "Copy-on-Write",
            Joker::SeedRound => "Seed Round",
            Joker::AngelInvestor => "Angel Investor",
            Joker::IncidentCommander => "Incident Commander",
            Joker::Rollback => "Rollback",
            Joker::SkipIntro => "Skip Intro",
            Joker::DeepDive => "Deep Dive",
            Joker::MeetupPizza => "Meetup Pizza",
            Joker::GemInstall => "Gem Install",
            Joker::EmojiReaction => "Emoji Reaction",
            Joker::Error404 => "Error 404",
            Joker::BitTwiddling => "Bit Twiddling",
            Joker::ProfilePicture => "Profile Picture",
            Joker::QueueManager => "Queue Manager",
            Joker::TwoFactorAuth => "Two-Factor Auth",
            Joker::MiddleManagement => "Middle Management",
            Joker::SplitBrain => "Split Brain",
            Joker::RainbowPrompt => "Rainbow Prompt",
            Joker::PitchBlack => "Pitch Black",
            Joker::JqFilter => "jq Filter",
            Joker::FuzzyFinder => "Fuzzy Finder",
            Joker::Identicon => "Identicon",
            Joker::BigBangRelease => "Big Bang Release",
            Joker::FastForward => "Fast-Forward",
            Joker::ChaosMonkey => "Chaos Monkey",
            Joker::RollingRelease => "Rolling Release",
            Joker::DailyDriver => "Daily Driver",
            Joker::LogRotation => "Log Rotation",
            Joker::BugBounty => "Bug Bounty",
            Joker::TodoMd => "TODO.md",
            Joker::GarageSale => "Garage Sale",
            Joker::Quine => "Quine",
            Joker::Headhunter => "Headhunter",
            Joker::EmptyWorkspace => "Empty Workspace",
            Joker::InboxZero => "Inbox Zero",
            Joker::ReleaseTrain => "Release Train",
            Joker::LastCommit => "Last Commit",
            Joker::FridayDeploy => "Friday Deploy",
            Joker::WarmStart => "Warm Start",
            Joker::GridLayout => "Grid Layout",
            Joker::DotfilesBak => "dotfiles.bak",
            Joker::HipsterStack => "Hipster Stack",
            Joker::EspressoShot => "Espresso Shot",
            Joker::HypeCycle => "Hype Cycle",
            Joker::EnergyDrink => "Energy Drink",
            Joker::SwapFile => "Swap File",
            Joker::Bloatware => "Bloatware",
            Joker::Backspace => "Backspace",
            Joker::VerticalMonitor => "Vertical Monitor",
            Joker::ForcePush => "Force Push",
            Joker::MinimalWaybar => "Minimal Waybar",
            Joker::PluginManager => "Plugin Manager",
            Joker::PacmanCache => "Pacman Cache",
            Joker::BitRot => "Bit Rot",
            Joker::FreeRam => "Free RAM",
            Joker::OwnedServers => "Owned Servers",
            Joker::Bootstrapped => "Bootstrapped",
            Joker::Homelab => "Homelab",
            Joker::PackageMaintainer => "Package Maintainer",
            Joker::CloudCredits => "Cloud Credits",
            Joker::SemverInflation => "Semver Inflation",
            Joker::NineNines => "Nine Nines",
            Joker::HockeyStick => "Hockey Stick",
            Joker::CompoundInterest => "Compound Interest",
            Joker::StickerPack => "Sticker Pack",
            Joker::RefreshSpam => "Refresh Spam",
            Joker::Paru => "paru",
            Joker::Flatpak => "Flatpak",
            Joker::BtrfsSnapshot => "Btrfs Snapshot",
            Joker::Syncthing => "Syncthing",
            Joker::Upstream => "Upstream",
            Joker::TransitiveDependencies => "Transitive Dependencies",
            Joker::RmRfNodeModules => "rm -rf node_modules",
            Joker::DistroHopping => "Distro Hopping",
            Joker::HashCollision => "Hash Collision",
            Joker::Postmortem => "Postmortem",
            Joker::SingleBinary => "Single Binary",
            Joker::IntegerOverflow => "Integer Overflow",
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
            Joker::FreeAsInBeer => "Themes and base16 packs in the shop cost $0",
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
            Joker::MinimalInstall => "Gains +3 mult per pack skipped",
            Joker::OptionalDependencies => {
                "1 in 2 chance to create a command when a pack is opened"
            }
            Joker::Dhh => "Played kings and queens each give ×2 mult",
            Joker::ThePrimeagen => "Gains ×1 mult when a face card is destroyed",
            Joker::MailingList => "Gains ×1 mult every 23 cards discarded",
            Joker::Kill9 => "Disables every boss blind's effect",
            Joker::CopyOnWrite => {
                "At the end of the shop, adds a Transparent copy of a random consumable"
            }
            Joker::SeedRound => "Creates an oligarchy card if the hand is a Straight Flush",
            Joker::AngelInvestor => {
                "If the first hand of a blind is a single 6, destroys it and creates an oligarchy card"
            }
            Joker::IncidentCommander => "+$8 each time a boss blind's effect triggers",
            Joker::Rollback => "Sell it to disable the current boss blind's effect",
            Joker::SkipIntro => "Gains ×0.25 mult per blind skipped",
            Joker::DeepDive => "+50 chips for each scoring spade",
            Joker::MeetupPizza => "+7 mult for each scoring club",
            Joker::GemInstall => "+$1 for each scoring diamond",
            Joker::EmojiReaction => "+5 mult for each scoring face card",
            Joker::Error404 => "+10 chips and +4 mult for each scoring 10 or 4",
            Joker::BitTwiddling => "Retriggers every scoring 2, 3, 4 and 5",
            Joker::ProfilePicture => "×2 mult for the first scoring face card",
            Joker::QueueManager => "+13 mult for each queen held in hand",
            Joker::TwoFactorAuth => "Gains +8 chips for each scoring 2",
            Joker::MiddleManagement => "1 in 2 chance of +$1 for each face card held in hand",
            Joker::SplitBrain => "×2 mult if a club and a card of another suit score",
            Joker::RainbowPrompt => "×3 mult if all four suits score",
            Joker::PitchBlack => "×3 mult if every card held in hand is a spade or a club",
            Joker::JqFilter => "Gains ×0.5 mult per jack discarded; resets after each blind",
            Joker::FuzzyFinder => {
                "Hearts and diamonds count as each other, and so do spades and clubs"
            }
            Joker::Identicon => "Every card counts as a face card",
            Joker::BigBangRelease => "Every played card scores",
            Joker::FastForward => "Straights may skip one rank between two cards",
            Joker::ChaosMonkey => "Doubles every chance (1 in 4 becomes 2 in 4)",
            Joker::RollingRelease => {
                "×1.5 mult for each scoring card of a suit drawn at each blind"
            }
            Joker::DailyDriver => {
                "×2 mult for each scoring card matching a card drawn at each blind"
            }
            Joker::LogRotation => "Gains +3 chips per discarded card of a suit drawn at each blind",
            Joker::BugBounty => "+$5 per discarded card of a rank drawn at each blind",
            Joker::TodoMd => "+$4 if the hand is of a type drawn at each blind",
            Joker::GarageSale => {
                "If the first discard of a blind is a single card, destroys it and gives $3"
            }
            Joker::Quine => {
                "If the first hand of a blind is a single card, adds a copy of it to the deck and the hand"
            }
            Joker::Headhunter => "+$5 when 3 face cards or more are discarded at once",
            Joker::EmptyWorkspace => "+30 chips per discard left",
            Joker::InboxZero => "+15 mult when no discard is left",
            Joker::ReleaseTrain => "×4 mult every 6 hands played",
            Joker::LastCommit => "Retriggers every scoring card of the last hand of a blind",
            Joker::FridayDeploy => "×3 mult on the last hand of a blind",
            Joker::WarmStart => "×3 mult if the hand type was already played this blind",
            Joker::GridLayout => "Gains +4 chips per hand of exactly 4 cards",
            Joker::DotfilesBak => "Gains +2 mult per hand containing a Two Pair",
            Joker::HipsterStack => {
                "Gains ×0.2 mult per hand in a row that isn't your most played type"
            }
            Joker::EspressoShot => "Retriggers every scoring card, for 10 hands",
            Joker::HypeCycle => "+20 mult, −4 mult at the end of each blind",
            Joker::EnergyDrink => "×2 mult, −×0.01 mult per card discarded",
            Joker::SwapFile => "+5 hand size, −1 at the end of each blind",
            Joker::Bloatware => "+250 chips, −2 hand size",
            Joker::Backspace => "+3 discards each blind, −1 hand size",
            Joker::VerticalMonitor => "+2 hand size, −1 hand each blind",
            Joker::ForcePush => "When a blind starts, +3 hands and no discards",
            Joker::MinimalWaybar => "×1 mult per empty joker slot (Minimal Waybar counts as empty)",
            Joker::PluginManager => "+3 mult per joker",
            Joker::PacmanCache => "Gains ×0.25 mult per card added to the deck",
            Joker::BitRot => "+4 mult per card the deck has lost since the start",
            Joker::FreeRam => "+2 chips per card left to draw",
            Joker::OwnedServers => "+2 chips per $1 you have",
            Joker::Bootstrapped => "+2 mult per $5 you have",
            Joker::Homelab => "Adds the sell value of your other jokers to mult",
            Joker::PackageMaintainer => "×1.5 mult per extra joker you own",
            Joker::CloudCredits => "You can spend down to −$20",
            Joker::SemverInflation => {
                "+$1 sell value to every joker and consumable at the end of each blind"
            }
            Joker::NineNines => "+$1 per 9 in the deck at the end of each blind",
            Joker::HockeyStick => "+$1 at the end of each blind, +$2 more after each boss",
            Joker::CompoundInterest => "+$1 of interest per $5 you have (same cap)",
            Joker::StickerPack => "Sell it to win the badge badges: 2x",
            Joker::RefreshSpam => "Gains +2 mult per shop reroll",
            Joker::Paru => "One free reroll per shop",
            Joker::Flatpak => "Jokers you own can show up in the shop again",
            Joker::BtrfsSnapshot => {
                "Saves a lost blind if you scored 25% of the target, then disappears"
            }
            Joker::Syncthing => "Sold after 2 blinds, copies another random joker",
            Joker::Upstream => "Copies the ability of the leftmost joker",
            Joker::TransitiveDependencies => {
                "When a blind starts, creates 2 core jokers (if there is room)"
            }
            Joker::RmRfNodeModules => {
                "When a blind starts, destroys the joker on its right and gains twice its sell value as mult"
            }
            Joker::DistroHopping => {
                "When a small or big blind starts, gains ×0.5 mult and destroys another random joker"
            }
            Joker::HashCollision => {
                "×2 mult if the hand contains two identical cards (same rank and suit)"
            }
            Joker::Postmortem => "Gains ×0.5 mult when a blind is won on its last hand",
            Joker::SingleBinary => "×3 mult if it is your only joker",
            Joker::IntegerOverflow => {
                "Doubles its mult when a blind is won in one hand; back to ×1 otherwise"
            }
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
            Joker::MinimalInstall => "Fewer packages, more speed.",
            Joker::OptionalDependencies => "Optional, but you'll install them anyway.",
            Joker::Dhh => "Strong opinions, loosely held. Mostly strong.",
            Joker::ThePrimeagen => "Blazingly fast. Skill issue for everyone else.",
            Joker::MailingList => "23 replies later, still no consensus.",
            Joker::Kill9 => "No signal handlers. No mercy.",
            Joker::CopyOnWrite => "Free copies until someone writes.",
            Joker::SeedRound => "Straight to the top. Then raise again.",
            Joker::AngelInvestor => "Sacrifices a six. Expects a unicorn.",
            Joker::IncidentCommander => "Paged at 3 a.m. Billed at 9.",
            Joker::Rollback => "git revert HEAD. Everyone relax.",
            Joker::SkipIntro => "Netflix taught us well.",
            Joker::DeepDive => "Three hours in, still reading source.",
            Joker::MeetupPizza => "Free pizza, mandatory lightning talk.",
            Joker::GemInstall => "bundle install, then go make coffee.",
            Joker::EmojiReaction => "A rocket on every pull request.",
            Joker::Error404 => "Card not found. Scored anyway.",
            Joker::BitTwiddling => "Small values, flipped twice as hard.",
            Joker::ProfilePicture => "First impression: doubled.",
            Joker::QueueManager => "Queens waiting in line. Patiently.",
            Joker::TwoFactorAuth => "Every 2 adds a layer. Every layer, a prompt.",
            Joker::MiddleManagement => "Paid to be in the room.",
            Joker::SplitBrain => "Two nodes, both think they're leader.",
            Joker::RainbowPrompt => "Starship, with every glyph enabled.",
            Joker::PitchBlack => "Black on black. Readable to you only.",
            Joker::JqFilter => "Filters out the jacks. Keeps the value.",
            Joker::FuzzyFinder => "fzf: close enough is a match.",
            Joker::Identicon => "Every card gets a face. Even the twos.",
            Joker::BigBangRelease => "Every card ships. No feature flags.",
            Joker::FastForward => "Skips a commit or two. No one checks.",
            Joker::ChaosMonkey => "Doubles the odds of everything.",
            Joker::RollingRelease => "Today's favorite suit, until the next update.",
            Joker::DailyDriver => "This one card. Today, anyway.",
            Joker::LogRotation => "Rotates daily. Keeps every line.",
            Joker::BugBounty => "Report the right bug, get paid.",
            Joker::TodoMd => "One item. Changes every blind.",
            Joker::GarageSale => "Old hardware, cash only.",
            Joker::Quine => "Prints itself. Adds itself to the deck.",
            Joker::Headhunter => "Three managers out. Budget's back.",
            Joker::EmptyWorkspace => "Workspaces 6 through 10, still empty.",
            Joker::InboxZero => "Everything discarded. Finally at peace.",
            Joker::ReleaseTrain => "Leaves every six hands. Packed.",
            Joker::LastCommit => "Final hand. Everything runs twice.",
            Joker::FridayDeploy => "Last hand, no rollback, maximum mult.",
            Joker::WarmStart => "Seen it this blind. Faster now.",
            Joker::GridLayout => "Four windows, not a pixel wasted.",
            Joker::DotfilesBak => "Two of everything. Just in case.",
            Joker::HipsterStack => "Never the popular framework.",
            Joker::EspressoShot => "Everything twice, for ten hands.",
            Joker::HypeCycle => "Peak of inflated expectations.",
            Joker::EnergyDrink => "×2 now. Crash later.",
            Joker::SwapFile => "Plenty of room. Shrinks every reboot.",
            Joker::Bloatware => "Huge numbers. No room for anything.",
            Joker::Backspace => "Worn smooth from all the deleting.",
            Joker::VerticalMonitor => "Taller view, fewer turns.",
            Joker::ForcePush => "No discards. No regrets. Push.",
            Joker::MinimalWaybar => "Less widgets, more wallpaper.",
            Joker::PluginManager => "lazy.nvim, 87 plugins, 12 ms startup.",
            Joker::PacmanCache => "Every package you add, kept forever.",
            Joker::BitRot => "Fewer cards, sharper teeth.",
            Joker::FreeRam => "Unused RAM is wasted RAM.",
            Joker::OwnedServers => "Paid once. Yours forever.",
            Joker::Bootstrapped => "No VCs. Profitable since day one.",
            Joker::Homelab => "Spare parts, adding up.",
            Joker::PackageMaintainer => "Every extra package, lovingly maintained.",
            Joker::CloudCredits => "Free until the invoice arrives.",
            Joker::SemverInflation => "1.0.0 to 47.2.0, same features.",
            Joker::NineNines => "99.9999999% uptime, billed per nine.",
            Joker::HockeyStick => "Growth chart up and to the right.",
            Joker::CompoundInterest => "Einstein's favorite. Allegedly.",
            Joker::StickerPack => "Free stickers at every conference.",
            Joker::RefreshSpam => "F5 until something changes.",
            Joker::Paru => "Free reroll, now in Rust.",
            Joker::Flatpak => "The same app, installed three times.",
            Joker::BtrfsSnapshot => "Rolled back from the brink. Once.",
            Joker::Syncthing => "Two blinds to sync, then a perfect copy.",
            Joker::Upstream => "Tracks the first joker, always.",
            Joker::TransitiveDependencies => "You installed one. Got two more.",
            Joker::RmRfNodeModules => "Frees 2 GB. Feels like a win.",
            Joker::DistroHopping => "Wipe everything, reinstall, feel alive.",
            Joker::HashCollision => "Two files, one checksum. Uh oh.",
            Joker::Postmortem => "We learned a lot. Mostly about failure.",
            Joker::SingleBinary => "Statically linked, zero dependencies.",
            Joker::IntegerOverflow => "MAX_INT + 1 = surprise.",
        }
    }

    pub fn rarity(self) -> Rarity {
        match self {
            Joker::Dhh
            | Joker::ThePrimeagen
            | Joker::MailingList
            | Joker::Kill9
            | Joker::CopyOnWrite => Rarity::Legendary,
            Joker::SeedRound
            | Joker::AngelInvestor
            | Joker::IncidentCommander
            | Joker::Rollback
            | Joker::SkipIntro => Rarity::Extra,
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
            Joker::DeepDive
            | Joker::MeetupPizza
            | Joker::GemInstall
            | Joker::BitTwiddling
            | Joker::SplitBrain
            | Joker::RainbowPrompt
            | Joker::PitchBlack
            | Joker::FuzzyFinder
            | Joker::Identicon
            | Joker::FastForward
            | Joker::ChaosMonkey
            | Joker::DailyDriver
            | Joker::LogRotation
            | Joker::GarageSale
            | Joker::ReleaseTrain
            | Joker::LastCommit
            | Joker::FridayDeploy
            | Joker::WarmStart
            | Joker::DotfilesBak
            | Joker::EspressoShot
            | Joker::EnergyDrink
            | Joker::SwapFile
            | Joker::Backspace
            | Joker::VerticalMonitor
            | Joker::ForcePush
            | Joker::MinimalWaybar
            | Joker::PacmanCache
            | Joker::BitRot
            | Joker::OwnedServers
            | Joker::Bootstrapped
            | Joker::SemverInflation
            | Joker::NineNines
            | Joker::HockeyStick
            | Joker::CompoundInterest
            | Joker::StickerPack
            | Joker::RefreshSpam
            | Joker::Flatpak
            | Joker::BtrfsSnapshot
            | Joker::RmRfNodeModules
            | Joker::DistroHopping
            | Joker::Postmortem
            | Joker::SingleBinary => Rarity::Extra,
            Joker::TwoFactorAuth
            | Joker::JqFilter
            | Joker::RollingRelease
            | Joker::Quine
            | Joker::HipsterStack
            | Joker::Bloatware
            | Joker::PackageMaintainer
            | Joker::Syncthing
            | Joker::Upstream
            | Joker::HashCollision
            | Joker::IntegerOverflow => Rarity::Aur,
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
            | Joker::VimGolf
            | Joker::OptionalDependencies => 4,
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
            | Joker::Man8
            | Joker::MinimalInstall => 5,
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
            Joker::SeedRound | Joker::AngelInvestor => 6,
            Joker::IncidentCommander => 7,
            Joker::Rollback => 5,
            Joker::SkipIntro => 6,
            Joker::Dhh
            | Joker::ThePrimeagen
            | Joker::MailingList
            | Joker::Kill9
            | Joker::CopyOnWrite => 20,
            Joker::CloudCredits => 2,
            Joker::BigBangRelease => 3,
            Joker::EmojiReaction
            | Joker::Error404
            | Joker::ChaosMonkey
            | Joker::BugBounty
            | Joker::TodoMd
            | Joker::Headhunter
            | Joker::GridLayout
            | Joker::PluginManager
            | Joker::Homelab
            | Joker::Paru => 4,
            Joker::ProfilePicture
            | Joker::QueueManager
            | Joker::Identicon
            | Joker::EmptyWorkspace
            | Joker::InboxZero
            | Joker::ReleaseTrain
            | Joker::LastCommit
            | Joker::HypeCycle
            | Joker::FreeRam
            | Joker::CompoundInterest
            | Joker::RefreshSpam
            | Joker::Flatpak
            | Joker::BtrfsSnapshot => 5,
            Joker::DeepDive
            | Joker::BitTwiddling
            | Joker::MiddleManagement
            | Joker::SplitBrain
            | Joker::RainbowPrompt
            | Joker::PitchBlack
            | Joker::DailyDriver
            | Joker::LogRotation
            | Joker::GarageSale
            | Joker::FridayDeploy
            | Joker::WarmStart
            | Joker::DotfilesBak
            | Joker::EspressoShot
            | Joker::EnergyDrink
            | Joker::SwapFile
            | Joker::VerticalMonitor
            | Joker::ForcePush
            | Joker::BitRot
            | Joker::OwnedServers
            | Joker::SemverInflation
            | Joker::HockeyStick
            | Joker::StickerPack
            | Joker::TransitiveDependencies
            | Joker::RmRfNodeModules
            | Joker::Postmortem
            | Joker::SingleBinary => 6,
            Joker::MeetupPizza
            | Joker::GemInstall
            | Joker::FuzzyFinder
            | Joker::FastForward
            | Joker::Bloatware
            | Joker::Backspace
            | Joker::PacmanCache
            | Joker::Bootstrapped
            | Joker::NineNines
            | Joker::DistroHopping => 7,
            Joker::TwoFactorAuth
            | Joker::JqFilter
            | Joker::RollingRelease
            | Joker::Quine
            | Joker::HipsterStack
            | Joker::MinimalWaybar
            | Joker::PackageMaintainer
            | Joker::Syncthing
            | Joker::HashCollision
            | Joker::IntegerOverflow => 8,
            Joker::Upstream => 10,
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
                | Joker::Kill9
                | Joker::Rollback
                | Joker::FuzzyFinder
                | Joker::Identicon
                | Joker::BigBangRelease
                | Joker::FastForward
                | Joker::ChaosMonkey
                | Joker::SwapFile
                | Joker::Backspace
                | Joker::VerticalMonitor
                | Joker::CloudCredits
                | Joker::CompoundInterest
                | Joker::StickerPack
                | Joker::Paru
                | Joker::Flatpak
                | Joker::BtrfsSnapshot
                | Joker::Syncthing
        )
    }

    /// Whether a shop joker may get a sticker: not those that run out, break,
    /// or are sold to work.
    pub fn sticker_compat(self) -> bool {
        !matches!(
            self,
            Joker::FreeTrial
                | Joker::EspressoShot
                | Joker::HypeCycle
                | Joker::SwapFile
                | Joker::EnergyDrink
                | Joker::BetaRelease
                | Joker::StableRelease
                | Joker::BtrfsSnapshot
                | Joker::Syncthing
                | Joker::StickerPack
                | Joker::Rollback
        )
    }

    /// Starting value of the joker's counter, for jokers that grow or shrink.
    fn initial_counter(self) -> f64 {
        match self {
            Joker::FreeTrial => 100.0,
            Joker::HypeCycle => 20.0,
            Joker::EspressoShot => 10.0,
            Joker::SwapFile => 5.0,
            Joker::EnergyDrink => 2.0,
            Joker::HockeyStick => 1.0,
            Joker::JqFilter
            | Joker::HipsterStack
            | Joker::PacmanCache
            | Joker::DistroHopping
            | Joker::OrphanCleanup
            | Joker::ThemeCollector
            | Joker::GarbageCollector
            | Joker::FlakyCi
            | Joker::RmaDepartment
            | Joker::ThePrimeagen
            | Joker::MailingList
            | Joker::SkipIntro
            | Joker::Postmortem
            | Joker::IntegerOverflow => 1.0,
            _ => 0.0,
        }
    }

    /// Triggered for each scoring card, right after its chips (`first_face`:
    /// the first scoring face card).
    pub fn on_card_scored(
        self,
        card: Card,
        first_face: bool,
        board: &Board,
        rng: &mut Rng,
    ) -> Vec<Effect> {
        let face = card.is_face();
        let targets = board.targets;
        match self {
            Joker::DeepDive if card.is_suit(Suit::Spades) => vec![Effect::Chips(50)],
            Joker::MeetupPizza if card.is_suit(Suit::Clubs) => vec![Effect::Mult(7.0)],
            Joker::GemInstall if card.is_suit(Suit::Diamonds) => vec![Effect::Money(1)],
            Joker::EmojiReaction if face => vec![Effect::Mult(5.0)],
            Joker::Error404 if card.has_rank() && matches!(card.rank, 4 | 10) => {
                vec![Effect::Chips(10), Effect::Mult(4.0)]
            }
            Joker::ProfilePicture if first_face => vec![Effect::XMult(2.0)],
            Joker::RollingRelease if card.is_suit(targets.suit) => vec![Effect::XMult(1.5)],
            Joker::DailyDriver
                if card.has_rank()
                    && card.rank == targets.card.0
                    && card.is_suit(targets.card.1) =>
            {
                vec![Effect::XMult(2.0)]
            }
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
            Joker::Dhh if card.has_rank() && matches!(card.rank, 12 | 13) => {
                vec![Effect::XMult(2.0)]
            }
            _ => Vec::new(),
        }
    }

    /// How many more times a scoring card scores (`first`: the first scoring
    /// card; `final_hand`: the last hand of the blind).
    pub fn retriggers(self, card: Card, first: bool, final_hand: bool) -> u32 {
        match self {
            Joker::BangBang if first => 2,
            Joker::WatchMode if card.is_face() => 1,
            Joker::BitTwiddling if card.has_rank() && (2..=5).contains(&card.rank) => 1,
            Joker::LastCommit if final_hand => 1,
            Joker::EspressoShot => 1,
            _ => 0,
        }
    }

    /// Triggered for each card held in hand (`lowest`: the lowest of them).
    pub fn on_held(self, card: Card, lowest: bool, rng: &mut Rng) -> Vec<Effect> {
        match self {
            Joker::QueueManager if card.has_rank() && card.rank == 12 => vec![Effect::Mult(13.0)],
            Joker::MiddleManagement if card.is_face() && rng.chance(2) => vec![Effect::Money(1)],
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
        let board = hand.board;
        let effect = match self {
            Joker::TwoFactorAuth | Joker::LogRotation | Joker::GridLayout if counter > 0.0 => {
                Effect::Chips(counter as u64)
            }
            Joker::DotfilesBak | Joker::HypeCycle | Joker::RefreshSpam | Joker::RmRfNodeModules
                if counter > 0.0 =>
            {
                Effect::Mult(counter)
            }
            Joker::JqFilter
            | Joker::HipsterStack
            | Joker::EnergyDrink
            | Joker::PacmanCache
            | Joker::DistroHopping
                if counter > 1.0 =>
            {
                Effect::XMult(counter)
            }
            Joker::SplitBrain if split_brain(&hand.scoring()) => Effect::XMult(2.0),
            Joker::HashCollision if has_twins(hand.played) => Effect::XMult(2.0),
            Joker::SingleBinary if board.jokers.len() == 1 => Effect::XMult(3.0),
            Joker::RainbowPrompt if all_suits(&hand.scoring()) => Effect::XMult(3.0),
            Joker::PitchBlack
                if hand
                    .held
                    .iter()
                    .all(|c| c.is_suit(Suit::Spades) || c.is_suit(Suit::Clubs)) =>
            {
                Effect::XMult(3.0)
            }
            Joker::TodoMd if kind == board.targets.hand => Effect::Money(4),
            Joker::EmptyWorkspace if board.discards_left > 0 => {
                Effect::Chips(30 * board.discards_left as u64)
            }
            Joker::InboxZero if board.discards_left == 0 => Effect::Mult(15.0),
            Joker::ReleaseTrain if counter >= RELEASE_TRAIN as f64 => Effect::XMult(4.0),
            Joker::FridayDeploy if board.hands_left == 1 => Effect::XMult(3.0),
            Joker::WarmStart if board.blind_hands.contains(&kind) => Effect::XMult(3.0),
            Joker::Bloatware => Effect::Chips(250),
            Joker::MinimalWaybar if board.waybar() > 1.0 => Effect::XMult(board.waybar()),
            Joker::PluginManager => Effect::Mult(3.0 * board.jokers.len() as f64),
            Joker::BitRot if board.lost_cards() > 0 => {
                Effect::Mult(4.0 * board.lost_cards() as f64)
            }
            Joker::FreeRam if board.draw_pile > 0 => Effect::Chips(2 * board.draw_pile as u64),
            Joker::OwnedServers if board.money > 0 => Effect::Chips(2 * board.money as u64),
            Joker::Bootstrapped if board.money >= 5 => Effect::Mult(2.0 * (board.money / 5) as f64),
            Joker::Homelab if board.resale() > 0 => Effect::Mult(board.resale() as f64),
            Joker::PackageMaintainer if board.maintained() > 1.0 => {
                Effect::XMult(board.maintained())
            }
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
            Joker::Uptime | Joker::GreenBuild | Joker::ShellFu | Joker::MinimalInstall
                if counter > 0.0 =>
            {
                Effect::Mult(counter)
            }
            Joker::FreeTrial | Joker::CiRunner if counter > 0.0 => Effect::Chips(counter as u64),
            Joker::MuscleMemory if hand.times_played > 0 => Effect::Mult(hand.times_played as f64),
            Joker::OrphanCleanup | Joker::ThemeCollector if counter > 1.0 => Effect::XMult(counter),
            Joker::GarbageCollector
            | Joker::RmaDepartment
            | Joker::ThePrimeagen
            | Joker::MailingList
            | Joker::SkipIntro
            | Joker::Postmortem
            | Joker::IntegerOverflow
                if counter > 1.0 =>
            {
                Effect::XMult(counter)
            }
            Joker::FlakyCi if counter + 0.25 * hand.lucky_hits as f64 > 1.0 => {
                Effect::XMult(counter + 0.25 * hand.lucky_hits as f64)
            }
            Joker::Systemd if count(board.deck, Enhancement::Daemon) > 0 => {
                Effect::XMult(1.0 + 0.2 * count(board.deck, Enhancement::Daemon) as f64)
            }
            Joker::BlobStorage if count(board.deck, Enhancement::BinaryBlob) > 0 => {
                Effect::Chips(25 * count(board.deck, Enhancement::BinaryBlob) as u64)
            }
            Joker::SeniorEngineer if enhanced_cards(board.deck) >= SENIOR_THRESHOLD => {
                Effect::XMult(3.0)
            }
            Joker::Segfault => Effect::Mult(rng.below(24) as f64),
            Joker::BetaRelease => Effect::Mult(15.0),
            Joker::StableRelease => Effect::XMult(3.0),
            _ => return Vec::new(),
        };
        vec![effect]
    }

    /// Money earned at the end of a won blind (`counter`: the joker's grown value).
    pub fn blind_money(self, counter: f64, discards_used: u32, board: &Board) -> u32 {
        let discards_left = board.discards_left;
        match self {
            Joker::CloudExit => discards_left,
            Joker::Unixporn => board.unique_themes,
            Joker::Sponsorware => 4,
            Joker::DataHoarder if discards_used == 0 => 2 * discards_left,
            Joker::NineNines => board.nines(),
            Joker::HockeyStick => counter as u32,
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OwnedJoker {
    pub joker: Joker,
    pub edition: Option<Edition>,
    /// The grown value of jokers that scale (mult, chips or ×mult).
    pub counter: f64,
    /// Added to the sell value (Vintage ThinkPad).
    pub sell_bonus: u32,
    /// Progress towards the next gain (Mailing List: cards discarded).
    pub progress: u32,
    /// Welded, Worn, Leased (the higher difficulty levels).
    #[serde(default)]
    pub stickers: Stickers,
}

/// How a won blind went, for the jokers that care.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BlindEnd {
    pub boss: bool,
    /// Won on its last hand (Postmortem).
    pub last_hand: bool,
    /// Won with a single hand (Integer Overflow).
    pub one_hand: bool,
}

/// Cards Mailing List needs discarded to gain ×1 mult.
pub const MAILING_LIST_DISCARDS: u32 = 23;

/// Hands Release Train waits for, the last one at ×4.
pub const RELEASE_TRAIN: u32 = 6;

/// Blinds Syncthing needs before it copies a joker.
pub const SYNCTHING_BLINDS: f64 = 2.0;

impl OwnedJoker {
    pub fn new(joker: Joker, edition: Option<Edition>) -> Self {
        OwnedJoker {
            joker,
            edition,
            counter: joker.initial_counter(),
            sell_bonus: 0,
            progress: 0,
            stickers: Stickers::default(),
        }
    }

    /// A Worn joker past its blinds: it does nothing.
    pub fn worn_out(&self) -> bool {
        self.stickers.worn_out()
    }

    pub fn price(&self) -> u32 {
        self.joker.price() + self.edition.map_or(0, Edition::price)
    }

    /// Half the price, at least $1, plus what it gained; a Leased joker
    /// sells for $1.
    pub fn sell_value(&self) -> u32 {
        let base = if self.stickers.leased {
            1
        } else {
            (self.price() / 2).max(1)
        };
        base + self.sell_bonus
    }

    /// What a joker that grows, shrinks or depends on the game gives right now,
    /// for its badge. `selected`: the selected hand type and the times it would
    /// have been played, this hand included (none when no hand is selected).
    pub fn bonus(&self, selected: Option<(HandKind, u32)>, board: &Board) -> Option<Bonus> {
        let c = round(self.counter);
        let deck = board.deck;
        let unique_themes = board.unique_themes;
        let hand_plays = selected.map(|(_, plays)| plays);
        let daemons = count(deck, Enhancement::Daemon);
        let enhanced = enhanced_cards(deck);
        let targets = board.targets;
        let to_go = |n: u32| format!("{n} hand{} to go", if n == 1 { "" } else { "s" });
        let (kind, text) = match self.joker {
            Joker::TwoFactorAuth
            | Joker::GridLayout
            | Joker::EmptyWorkspace
            | Joker::FreeRam
            | Joker::OwnedServers => {
                let chips = match self.joker {
                    Joker::EmptyWorkspace => 30 * board.discards_left as u64,
                    Joker::FreeRam => 2 * board.draw_pile as u64,
                    Joker::OwnedServers => 2 * board.money.max(0) as u64,
                    _ => c as u64,
                };
                ("chips", format!("+{chips} chips"))
            }
            Joker::LogRotation => (
                "chips",
                format!("+{c} chips · {}", targets.discard_suit.symbol()),
            ),
            Joker::JqFilter
            | Joker::HipsterStack
            | Joker::EnergyDrink
            | Joker::PacmanCache
            | Joker::DistroHopping => ("x_mult", format!("×{c} mult")),
            Joker::DotfilesBak | Joker::HypeCycle | Joker::RefreshSpam | Joker::RmRfNodeModules => {
                ("mult", format!("+{c} mult"))
            }
            Joker::PluginManager | Joker::BitRot | Joker::Bootstrapped | Joker::Homelab => {
                let mult = match self.joker {
                    Joker::PluginManager => 3 * board.jokers.len() as u64,
                    Joker::BitRot => 4 * board.lost_cards() as u64,
                    Joker::Bootstrapped => 2 * (board.money.max(0) / 5) as u64,
                    _ => board.resale() as u64,
                };
                ("mult", format!("+{mult} mult"))
            }
            Joker::MinimalWaybar => ("x_mult", format!("×{} mult", board.waybar())),
            Joker::PackageMaintainer => ("x_mult", format!("×{} mult", round(board.maintained()))),
            Joker::RollingRelease => ("x_mult", format!("×1.5 per {}", targets.suit.symbol())),
            Joker::DailyDriver => (
                "x_mult",
                format!(
                    "×2 per {}{}",
                    rank_label(targets.card.0),
                    targets.card.1.symbol()
                ),
            ),
            Joker::BugBounty => ("money", format!("$5 per {}", rank_label(targets.rank))),
            Joker::TodoMd => ("money", format!("$4 for {}", targets.hand.name())),
            Joker::InboxZero if board.discards_left == 0 => ("mult", "+15 mult".to_string()),
            Joker::InboxZero => ("progress", format!("{} discards left", board.discards_left)),
            Joker::ReleaseTrain => match RELEASE_TRAIN - (self.counter as u32).min(RELEASE_TRAIN) {
                0 | 1 => ("x_mult", "×4 next hand".to_string()),
                n => ("progress", to_go(n - 1)),
            },
            Joker::FridayDeploy if board.hands_left == 1 => ("x_mult", "×3 mult".to_string()),
            Joker::LastCommit if board.hands_left == 1 => ("progress", "Retriggers".to_string()),
            Joker::FridayDeploy | Joker::LastCommit => {
                ("progress", to_go(board.hands_left.saturating_sub(1)))
            }
            Joker::WarmStart if selected.is_some_and(|(k, _)| board.blind_hands.contains(&k)) => {
                ("x_mult", "×3 mult".to_string())
            }
            Joker::EspressoShot => ("progress", format!("{c} hands left")),
            Joker::SwapFile => ("progress", format!("+{c} hand size")),
            Joker::NineNines => ("money", format!("+${} / blind", board.nines())),
            Joker::HockeyStick => ("money", format!("+${c} / blind")),
            Joker::Syncthing if self.counter >= SYNCTHING_BLINDS => {
                ("progress", "Synced: sell it".to_string())
            }
            Joker::Syncthing => ("progress", format!("{c} / {SYNCTHING_BLINDS} blinds")),
            Joker::GarbageCollector
            | Joker::FlakyCi
            | Joker::RmaDepartment
            | Joker::ThePrimeagen
            | Joker::SkipIntro => ("x_mult", format!("×{c} mult")),
            Joker::MailingList => (
                "x_mult",
                format!("×{c} mult · {}/{MAILING_LIST_DISCARDS}", self.progress),
            ),
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
            Joker::Uptime | Joker::GreenBuild | Joker::ShellFu | Joker::MinimalInstall => {
                ("mult", format!("+{c} mult"))
            }
            Joker::FreeTrial | Joker::CiRunner => ("chips", format!("+{c} chips")),
            Joker::OrphanCleanup
            | Joker::ThemeCollector
            | Joker::Postmortem
            | Joker::IntegerOverflow => ("x_mult", format!("×{c} mult")),
            Joker::SingleBinary => {
                let alone = board.jokers.len() == 1;
                ("x_mult", format!("×{} mult", if alone { 3 } else { 1 }))
            }
            Joker::MuscleMemory => ("mult", format!("+{} mult", hand_plays?)),
            Joker::Unixporn => ("money", format!("+${unique_themes} / blind")),
            Joker::VintageThinkpad => ("money", format!("sells ${}", self.sell_value())),
            _ => return None,
        };
        Some(Bonus { kind, text })
    }

    /// Before a hand is scored: jokers that grow with the hands played
    /// (`most_played`: the hand type is the most played one so far).
    pub fn before_hand(
        &mut self,
        evaluated: &Evaluated,
        played: &[Card],
        scoring: &[Card],
        most_played: bool,
    ) {
        match self.joker {
            Joker::TwoFactorAuth => {
                let twos = scoring
                    .iter()
                    .filter(|c| c.has_rank() && c.rank == 2)
                    .count();
                self.counter += 8.0 * twos as f64;
            }
            Joker::GridLayout if played.len() == 4 => self.counter += 4.0,
            Joker::DotfilesBak if evaluated.contains(HandKind::TwoPair) => self.counter += 2.0,
            Joker::HipsterStack if most_played => self.counter = 1.0,
            Joker::HipsterStack => self.counter += 0.2,
            Joker::ReleaseTrain => self.counter += 1.0,
            Joker::Uptime if scoring.iter().any(|c| c.is_face()) => self.counter = 0.0,
            Joker::Uptime | Joker::GreenBuild => self.counter += 1.0,
            Joker::CiRunner if evaluated.contains(HandKind::Straight) => self.counter += 15.0,
            _ => {}
        }
    }

    /// After a hand is scored; false when the joker is used up.
    pub fn after_hand(&mut self) -> bool {
        match self.joker {
            Joker::FreeTrial => {
                self.counter -= 5.0;
                return self.counter > 0.0;
            }
            Joker::EspressoShot => {
                self.counter -= 1.0;
                return self.counter > 0.0;
            }
            Joker::ReleaseTrain if self.counter >= RELEASE_TRAIN as f64 => self.counter = 0.0,
            _ => {}
        }
        true
    }

    pub fn on_discard(&mut self) {
        if self.joker == Joker::GreenBuild {
            self.counter = (self.counter - 1.0).max(0.0);
        }
    }

    /// Face cards were destroyed (ThePrimeagen grows).
    pub fn on_faces_destroyed(&mut self, faces: usize) {
        if self.joker == Joker::ThePrimeagen {
            self.counter += faces as f64;
        }
    }

    /// Cards were discarded (Mailing List grows every 23); false when the
    /// joker is used up (Energy Drink).
    pub fn on_cards_discarded(&mut self, cards: &[Card], targets: &Targets) -> bool {
        match self.joker {
            Joker::MailingList => {
                self.progress += cards.len() as u32;
                self.counter += (self.progress / MAILING_LIST_DISCARDS) as f64;
                self.progress %= MAILING_LIST_DISCARDS;
            }
            Joker::JqFilter => {
                let jacks = cards
                    .iter()
                    .filter(|c| c.has_rank() && c.rank == 11)
                    .count();
                self.counter += 0.5 * jacks as f64;
            }
            Joker::LogRotation => {
                let suited = cards
                    .iter()
                    .filter(|c| c.is_suit(targets.discard_suit))
                    .count();
                self.counter += 3.0 * suited as f64;
            }
            Joker::EnergyDrink => {
                self.counter = round(self.counter - 0.01 * cards.len() as f64);
                return self.counter > 1.0;
            }
            _ => {}
        }
        true
    }

    /// The shop was rerolled.
    pub fn on_reroll(&mut self) {
        if self.joker == Joker::RefreshSpam {
            self.counter += 2.0;
        }
    }

    /// Cards were added to the deck.
    pub fn on_cards_added(&mut self, cards: usize) {
        if self.joker == Joker::PacmanCache {
            self.counter += 0.25 * cards as f64;
        }
    }

    pub fn on_blind_skipped(&mut self) {
        if self.joker == Joker::SkipIntro {
            self.counter += 0.25;
        }
    }

    pub fn on_pack_skipped(&mut self) {
        if self.joker == Joker::MinimalInstall {
            self.counter += 3.0;
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
    pub fn end_of_blind(&mut self, end: BlindEnd, rng: &mut Rng) -> bool {
        let boss = end.boss;
        match self.joker {
            Joker::VintageThinkpad => self.sell_bonus += 3,
            Joker::OrphanCleanup if boss => self.counter = 1.0,
            Joker::BetaRelease => return !rng.chance(6),
            Joker::StableRelease => return !rng.chance(1000),
            Joker::JqFilter => self.counter = 1.0,
            Joker::HypeCycle | Joker::SwapFile => {
                self.counter -= if self.joker == Joker::HypeCycle {
                    4.0
                } else {
                    1.0
                };
                return self.counter > 0.0;
            }
            Joker::Syncthing => self.counter += 1.0,
            Joker::HockeyStick if boss => self.counter += 2.0,
            Joker::Postmortem if end.last_hand => self.counter += 0.5,
            Joker::IntegerOverflow if end.one_hand => self.counter *= 2.0,
            Joker::IntegerOverflow => self.counter = 1.0,
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
        assert_eq!(
            cleanup.bonus(None, &Board::default()).unwrap().text,
            "×1.25 mult"
        );

        let mut collector = OwnedJoker::new(Joker::ThemeCollector, None);
        assert_eq!(
            collector.bonus(None, &Board::default()).unwrap().text,
            "×1 mult"
        );
        for _ in 0..3 {
            collector.on_theme_used();
        }
        assert_eq!(
            collector.bonus(None, &Board::default()).unwrap().text,
            "×1.3 mult"
        );
        let memory = OwnedJoker::new(Joker::MuscleMemory, None);
        assert_eq!(memory.bonus(None, &Board::default()), None);
        assert_eq!(
            memory
                .bonus(Some((HandKind::Pair, 4)), &Board::default())
                .unwrap()
                .text,
            "+4 mult"
        );
        cleanup.end_of_blind(
            BlindEnd {
                boss: true,
                ..BlindEnd::default()
            },
            &mut Rng::new(1),
        );
        assert_eq!(cleanup.counter, 1.0);
    }

    #[test]
    fn retriggers_and_held_cards() {
        let king = Card::new(13, Suit::Spades);
        let rng = &mut Rng::new(1);
        assert_eq!(Joker::BangBang.retriggers(king, true, false), 2);
        assert_eq!(Joker::BangBang.retriggers(king, false, false), 0);
        assert_eq!(Joker::WatchMode.retriggers(king, false, false), 1);
        assert_eq!(Joker::LastCommit.retriggers(king, false, true), 1);
        assert_eq!(Joker::LastCommit.retriggers(king, false, false), 0);
        let two = Card::new(2, Suit::Clubs);
        assert_eq!(Joker::BitTwiddling.retriggers(two, false, false), 1);
        assert_eq!(Joker::BitTwiddling.retriggers(king, false, false), 0);
        assert_eq!(
            Joker::LowBattery.on_held(king, true, rng),
            [Effect::Mult(20.0)]
        );
        assert_eq!(
            Joker::BenevolentDictator.on_held(king, false, rng),
            [Effect::XMult(1.5)]
        );
        let queen = Card::new(12, Suit::Hearts);
        assert_eq!(
            Joker::QueueManager.on_held(queen, false, rng),
            [Effect::Mult(13.0)]
        );
    }

    #[test]
    fn counters_that_run_out() {
        let mut espresso = OwnedJoker::new(Joker::EspressoShot, None);
        for _ in 0..9 {
            assert!(espresso.after_hand());
        }
        assert!(!espresso.after_hand());
        let mut hype = OwnedJoker::new(Joker::HypeCycle, None);
        let rng = &mut Rng::new(1);
        assert!((0..4).all(|_| hype.end_of_blind(BlindEnd::default(), rng)));
        assert!(!hype.end_of_blind(BlindEnd::default(), rng));
        let mut drink = OwnedJoker::new(Joker::EnergyDrink, None);
        let five = crate::card::cards("2H 3H 4H 5H 6H");
        assert!((0..19).all(|_| drink.on_cards_discarded(&five, &Targets::default())));
        assert!(!drink.on_cards_discarded(&five, &Targets::default()));
        let mut train = OwnedJoker::new(Joker::ReleaseTrain, None);
        let pair = crate::hand::evaluate(&five[..2]);
        for _ in 0..5 {
            train.before_hand(&pair, &five, &five, false);
            train.after_hand();
        }
        assert_eq!(
            train.bonus(None, &Board::default()).unwrap().text,
            "×4 next hand"
        );
    }
}
