//! Boss blinds: each ante's boss has an effect that gets in the way (the final
//! bosses, at the winning ante and its multiples, more so). kill -9 disables
//! them all; Rollback, sold, disables the current one.

use crate::card::Suit;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Boss {
    OomKiller,
    CloudBill,
    BlackScreen,
    KubernetesCluster,
    FlakyWifi,
    Regression,
    ElectronApp,
    ScreenLock,
    MandatoryMeeting,
    NvidiaDriver,
    ReadOnlyFilesystem,
    Paywall,
    Monitor43,
    CodeReviewer,
    StyleGuide,
    AnonymousAccount,
    RateLimiter,
    StaleCache,
    Deadline,
    Burnout,
    PerSeatLicense,
    ThermalThrottle,
    DarkPattern,
    KernelPanic,
    VendorLockIn,
    Microservices,
    WindowsUpdate,
    Clippy,
}

impl Boss {
    pub const ALL: [Boss; 28] = [
        Boss::OomKiller,
        Boss::CloudBill,
        Boss::BlackScreen,
        Boss::KubernetesCluster,
        Boss::FlakyWifi,
        Boss::Regression,
        Boss::ElectronApp,
        Boss::ScreenLock,
        Boss::MandatoryMeeting,
        Boss::NvidiaDriver,
        Boss::ReadOnlyFilesystem,
        Boss::Paywall,
        Boss::Monitor43,
        Boss::CodeReviewer,
        Boss::StyleGuide,
        Boss::AnonymousAccount,
        Boss::RateLimiter,
        Boss::StaleCache,
        Boss::Deadline,
        Boss::Burnout,
        Boss::PerSeatLicense,
        Boss::ThermalThrottle,
        Boss::DarkPattern,
        Boss::KernelPanic,
        Boss::VendorLockIn,
        Boss::Microservices,
        Boss::WindowsUpdate,
        Boss::Clippy,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Boss::OomKiller => "The OOM Killer",
            Boss::CloudBill => "The Cloud Bill",
            Boss::BlackScreen => "The Black Screen",
            Boss::KubernetesCluster => "The Kubernetes Cluster",
            Boss::FlakyWifi => "The Flaky Wi-Fi",
            Boss::Regression => "The Regression",
            Boss::ElectronApp => "The Electron App",
            Boss::ScreenLock => "The Screen Lock",
            Boss::MandatoryMeeting => "The Mandatory Meeting",
            Boss::NvidiaDriver => "The Nvidia Driver",
            Boss::ReadOnlyFilesystem => "The Read-Only Filesystem",
            Boss::Paywall => "The Paywall",
            Boss::Monitor43 => "The 4:3 Monitor",
            Boss::CodeReviewer => "The Code Reviewer",
            Boss::StyleGuide => "The Style Guide",
            Boss::AnonymousAccount => "The Anonymous Account",
            Boss::RateLimiter => "The Rate Limiter",
            Boss::StaleCache => "The Stale Cache",
            Boss::Deadline => "The Deadline",
            Boss::Burnout => "The Burnout",
            Boss::PerSeatLicense => "The Per-Seat License",
            Boss::ThermalThrottle => "The Thermal Throttle",
            Boss::DarkPattern => "The Dark Pattern",
            Boss::KernelPanic => "The Kernel Panic",
            Boss::VendorLockIn => "The Vendor Lock-in",
            Boss::Microservices => "The Microservices",
            Boss::WindowsUpdate => "The Windows Update",
            Boss::Clippy => "The Clippy",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Boss::OomKiller => "Discards 2 random cards from the hand after each hand played",
            Boss::CloudBill => "Playing your most played hand type sets money to $0",
            Boss::BlackScreen => "The first hand is dealt face down",
            Boss::KubernetesCluster => "A huge blind: ×4 the base score",
            Boss::FlakyWifi => "1 in 7 cards is drawn face down",
            Boss::Regression => "Each hand played loses one level",
            Boss::ElectronApp => "Clubs don't score",
            Boss::ScreenLock => "Cards drawn after each hand are face down",
            Boss::MandatoryMeeting => "Hands must have 5 cards",
            Boss::NvidiaDriver => "Spades don't score",
            Boss::ReadOnlyFilesystem => "No discards",
            Boss::Paywall => "Diamonds don't score",
            Boss::Monitor43 => "−1 hand size",
            Boss::CodeReviewer => "No hand type can be played twice",
            Boss::StyleGuide => "Only one hand type can be played",
            Boss::AnonymousAccount => "Face cards don't score",
            Boss::RateLimiter => "After each hand or discard, draw only 3 cards",
            Boss::StaleCache => "Cards already played this ante don't score",
            Boss::Deadline => "Only 1 hand, but a smaller blind (×1)",
            Boss::Burnout => "Hearts don't score",
            Boss::PerSeatLicense => "−$1 per card played",
            Boss::ThermalThrottle => "Base chips and mult are halved",
            Boss::DarkPattern => "Face cards are drawn face down",
            Boss::KernelPanic => "Jokers are flipped face down and shuffled",
            Boss::VendorLockIn => "No card scores until a joker is sold",
            Boss::Microservices => "A colossal blind: ×6 the base score",
            Boss::WindowsUpdate => "A random joker is disabled each hand",
            Boss::Clippy => "One card is always forced into the selection",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Boss::OomKiller => "Something had to go.",
            Boss::CloudBill => "Your usage spiked. So did the invoice.",
            Boss::BlackScreen => "Did it boot? Nobody knows.",
            Boss::KubernetesCluster => "Seventeen YAML files for one hello world.",
            Boss::FlakyWifi => "Connected, no internet.",
            Boss::Regression => "It worked in the last release.",
            Boss::ElectronApp => "500 MB of RAM to show a club.",
            Boss::ScreenLock => "Hyprlock kicked in. Type your password.",
            Boss::MandatoryMeeting => "Everyone has to attend.",
            Boss::NvidiaDriver => "Spades? Not supported on Wayland.",
            Boss::ReadOnlyFilesystem => "mount: / is read-only.",
            Boss::Paywall => "Diamonds are for premium users.",
            Boss::Monitor43 => "Remember when screens were square?",
            Boss::CodeReviewer => "Seen this pattern before. Rejected.",
            Boss::StyleGuide => "Pick a convention. Forever.",
            Boss::AnonymousAccount => "No faces, only opinions.",
            Boss::RateLimiter => "429 Too Many Requests.",
            Boss::StaleCache => "Invalidate and try again.",
            Boss::Deadline => "Ship it today. One shot.",
            Boss::Burnout => "No love left for this codebase.",
            Boss::PerSeatLicense => "Every seat is billed. Every one.",
            Boss::ThermalThrottle => "Fans at 100%. Clocks at 50%.",
            Boss::DarkPattern => "Hidden on purpose.",
            Boss::KernelPanic => "Attempted to kill init!",
            Boss::VendorLockIn => "Pay to leave.",
            Boss::Microservices => "Two hundred services for a todo app.",
            Boss::WindowsUpdate => "Restarting in 10 minutes. Or now.",
            Boss::Clippy => "It looks like you're playing a hand!",
        }
    }

    /// A final boss: at the winning ante and its multiples only.
    pub fn is_final(self) -> bool {
        self.min_ante() == 0
    }

    /// First ante it may show up at (0 for the final bosses).
    pub fn min_ante(self) -> u32 {
        match self {
            Boss::OomKiller => 1,
            Boss::CloudBill => 6,
            Boss::BlackScreen => 2,
            Boss::KubernetesCluster => 2,
            Boss::FlakyWifi => 2,
            Boss::Regression => 2,
            Boss::ElectronApp => 1,
            Boss::ScreenLock => 2,
            Boss::MandatoryMeeting => 1,
            Boss::NvidiaDriver => 1,
            Boss::ReadOnlyFilesystem => 2,
            Boss::Paywall => 1,
            Boss::Monitor43 => 1,
            Boss::CodeReviewer => 3,
            Boss::StyleGuide => 2,
            Boss::AnonymousAccount => 4,
            Boss::RateLimiter => 5,
            Boss::StaleCache => 1,
            Boss::Deadline => 2,
            Boss::Burnout => 1,
            Boss::PerSeatLicense => 3,
            Boss::ThermalThrottle => 2,
            Boss::DarkPattern => 2,
            Boss::KernelPanic => 0,
            Boss::VendorLockIn => 0,
            Boss::Microservices => 0,
            Boss::WindowsUpdate => 0,
            Boss::Clippy => 0,
        }
    }

    /// Target multiplier over the ante's base score.
    pub fn multiplier(self) -> f64 {
        match self {
            Boss::KubernetesCluster => 4.0,
            Boss::Microservices => 6.0,
            Boss::Deadline => 1.0,
            _ => 2.0,
        }
    }

    /// Money for beating it.
    pub fn reward(self) -> u32 {
        if self.is_final() { 8 } else { 5 }
    }

    /// The suit it keeps from scoring, if any.
    pub fn debuffed_suit(self) -> Option<Suit> {
        match self {
            Boss::ElectronApp => Some(Suit::Clubs),
            Boss::NvidiaDriver => Some(Suit::Spades),
            Boss::Paywall => Some(Suit::Diamonds),
            Boss::Burnout => Some(Suit::Hearts),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bosses_are_listed_once_with_short_quips() {
        for boss in Boss::ALL {
            assert_eq!(Boss::ALL.iter().filter(|&&b| b == boss).count(), 1);
            assert!(boss.quip().chars().count() <= 50, "{}", boss.quip());
        }
        assert_eq!(Boss::ALL.iter().filter(|b| b.is_final()).count(), 5);
        assert_eq!(
            (Boss::Microservices.reward(), Boss::Burnout.reward()),
            (8, 5)
        );
    }
}
