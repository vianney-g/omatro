//! Pull requests (vouchers, in the original): permanent upgrades, one for sale
//! per ante; each has a follow-up, sold once it was merged.

use serde::{Deserialize, Serialize};

/// What a pull request costs to merge.
pub const VOUCHER_PRICE: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Voucher {
    MoreWorkspaces,
    EvenMoreWorkspaces,
    EarlyBird,
    LifetimeDeal,
    CompositorTweaks,
    EyeCandy,
    MirrorList,
    FastestMirror,
    ClipboardManager,
    InsiderAccess,
    ThemePreview,
    ThemeShowcase,
    Overtime,
    CrunchTime,
    RmI,
    EmptyTrash,
    ShellAliases,
    ShellPlugins,
    ThemeGallery,
    ThemeMarketplace,
    HighYieldSavings,
    IndexFund,
    LoremIpsum,
    FeatureFlag,
    CardMarketplace,
    PremiumCards,
    GitRevert,
    GitResetHard,
    ChangeRequest,
    EmergencyChange,
    Hidpi,
    Display4k,
}

impl Voucher {
    pub const ALL: [Voucher; 32] = [
        Voucher::MoreWorkspaces,
        Voucher::EvenMoreWorkspaces,
        Voucher::EarlyBird,
        Voucher::LifetimeDeal,
        Voucher::CompositorTweaks,
        Voucher::EyeCandy,
        Voucher::MirrorList,
        Voucher::FastestMirror,
        Voucher::ClipboardManager,
        Voucher::InsiderAccess,
        Voucher::ThemePreview,
        Voucher::ThemeShowcase,
        Voucher::Overtime,
        Voucher::CrunchTime,
        Voucher::RmI,
        Voucher::EmptyTrash,
        Voucher::ShellAliases,
        Voucher::ShellPlugins,
        Voucher::ThemeGallery,
        Voucher::ThemeMarketplace,
        Voucher::HighYieldSavings,
        Voucher::IndexFund,
        Voucher::LoremIpsum,
        Voucher::FeatureFlag,
        Voucher::CardMarketplace,
        Voucher::PremiumCards,
        Voucher::GitRevert,
        Voucher::GitResetHard,
        Voucher::ChangeRequest,
        Voucher::EmergencyChange,
        Voucher::Hidpi,
        Voucher::Display4k,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Voucher::MoreWorkspaces => "More Workspaces",
            Voucher::EvenMoreWorkspaces => "Even More Workspaces",
            Voucher::EarlyBird => "Early Bird",
            Voucher::LifetimeDeal => "Lifetime Deal",
            Voucher::CompositorTweaks => "Compositor Tweaks",
            Voucher::EyeCandy => "Eye Candy",
            Voucher::MirrorList => "Mirror List",
            Voucher::FastestMirror => "Fastest Mirror",
            Voucher::ClipboardManager => "Clipboard Manager",
            Voucher::InsiderAccess => "Insider Access",
            Voucher::ThemePreview => "Theme Preview",
            Voucher::ThemeShowcase => "Theme Showcase",
            Voucher::Overtime => "Overtime",
            Voucher::CrunchTime => "Crunch Time",
            Voucher::RmI => "rm -i",
            Voucher::EmptyTrash => "Empty Trash",
            Voucher::ShellAliases => "Shell Aliases",
            Voucher::ShellPlugins => "Shell Plugins",
            Voucher::ThemeGallery => "Theme Gallery",
            Voucher::ThemeMarketplace => "Theme Marketplace",
            Voucher::HighYieldSavings => "High-Yield Savings",
            Voucher::IndexFund => "Index Fund",
            Voucher::LoremIpsum => "Lorem Ipsum",
            Voucher::FeatureFlag => "Feature Flag",
            Voucher::CardMarketplace => "Card Marketplace",
            Voucher::PremiumCards => "Premium Cards",
            Voucher::GitRevert => "git revert",
            Voucher::GitResetHard => "git reset --hard",
            Voucher::ChangeRequest => "Change Request",
            Voucher::EmergencyChange => "Emergency Change",
            Voucher::Hidpi => "HiDPI",
            Voucher::Display4k => "4K Display",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Voucher::MoreWorkspaces => "+1 card slot in the shop",
            Voucher::EvenMoreWorkspaces => "+1 more card slot in the shop",
            Voucher::EarlyBird => "Cards and packs in the shop are 25% off",
            Voucher::LifetimeDeal => "Cards and packs in the shop are 50% off",
            Voucher::CompositorTweaks => "Editions show up twice as often",
            Voucher::EyeCandy => "Editions show up four times as often",
            Voucher::MirrorList => "Rerolls cost $2 less",
            Voucher::FastestMirror => "Rerolls cost $2 less again",
            Voucher::ClipboardManager => "+1 clipboard slot",
            Voucher::InsiderAccess => "Oligarchy cards may show up in coreutils packs",
            Voucher::ThemePreview => "base16 packs always hold the theme of your most played hand",
            Voucher::ThemeShowcase => "Themes in the clipboard give ×1.5 mult to their hand type",
            Voucher::Overtime => "+1 hand each blind",
            Voucher::CrunchTime => "+1 more hand each blind",
            Voucher::RmI => "+1 discard each blind",
            Voucher::EmptyTrash => "+1 more discard each blind",
            Voucher::ShellAliases => "Commands show up twice as often in the shop",
            Voucher::ShellPlugins => "Commands show up four times as often in the shop",
            Voucher::ThemeGallery => "Themes show up twice as often in the shop",
            Voucher::ThemeMarketplace => "Themes show up four times as often in the shop",
            Voucher::HighYieldSavings => "Interest up to $10 per blind",
            Voucher::IndexFund => "Interest up to $20 per blind",
            Voucher::LoremIpsum => "Does nothing",
            Voucher::FeatureFlag => "+1 joker slot",
            Voucher::CardMarketplace => "Playing cards can show up in the shop",
            Voucher::PremiumCards => "Playing cards in the shop may have modifiers",
            Voucher::GitRevert => "−1 ante; −1 hand each blind",
            Voucher::GitResetHard => "−1 ante again; −1 discard each blind",
            Voucher::ChangeRequest => "Reroll the boss once per ante, for $10",
            Voucher::EmergencyChange => "Reroll the boss as often as you like, for $10",
            Voucher::Hidpi => "+1 hand size",
            Voucher::Display4k => "+1 more hand size",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Voucher::MoreWorkspaces => "Super+5 through Super+9, finally.",
            Voucher::EvenMoreWorkspaces => "Nobody needs ten workspaces. You do.",
            Voucher::EarlyBird => "Bought before the hype.",
            Voucher::LifetimeDeal => "Pay once. Regret never. Probably.",
            Voucher::CompositorTweaks => "decoration { blur = true }",
            Voucher::EyeCandy => "Wobbly everything.",
            Voucher::MirrorList => "reflector --sort rate",
            Voucher::FastestMirror => "Downloads at line speed.",
            Voucher::ClipboardManager => "Remembers more than one thing.",
            Voucher::InsiderAccess => "Elite capital, now in coreutils.",
            Voucher::ThemePreview => "See it before you set it.",
            Voucher::ThemeShowcase => "Your favorite themes, on display.",
            Voucher::Overtime => "One more hand. Just one.",
            Voucher::CrunchTime => "Nobody sleeps before the release.",
            Voucher::RmI => "Asks before deleting. Once more.",
            Voucher::EmptyTrash => "Gone for good. Twice.",
            Voucher::ShellAliases => "alias ll='ls -la', and friends.",
            Voucher::ShellPlugins => "oh-my-zsh, but tasteful.",
            Voucher::ThemeGallery => "Thirty themes, one click away.",
            Voucher::ThemeMarketplace => "Every color scheme ever made.",
            Voucher::HighYieldSavings => "Responsible. Mostly.",
            Voucher::IndexFund => "Very boring. Very rich.",
            Voucher::LoremIpsum => "Content coming soon.",
            Voucher::FeatureFlag => "It was always there. Now it's on.",
            Voucher::CardMarketplace => "Playing cards, now for sale.",
            Voucher::PremiumCards => "Some come pre-configured.",
            Voucher::GitRevert => "Back one ante. One hand short.",
            Voucher::GitResetHard => "Back again. Lose a discard.",
            Voucher::ChangeRequest => "File a ticket for a different boss.",
            Voucher::EmergencyChange => "No ticket needed. Just pay.",
            Voucher::Hidpi => "More pixels, more cards.",
            Voucher::Display4k => "Scaling at 1.25. Perfect.",
        }
    }

    /// The pull request this one follows up on (none for the first of a pair).
    pub fn follows(self) -> Option<Voucher> {
        match self {
            Voucher::EvenMoreWorkspaces => Some(Voucher::MoreWorkspaces),
            Voucher::LifetimeDeal => Some(Voucher::EarlyBird),
            Voucher::EyeCandy => Some(Voucher::CompositorTweaks),
            Voucher::FastestMirror => Some(Voucher::MirrorList),
            Voucher::InsiderAccess => Some(Voucher::ClipboardManager),
            Voucher::ThemeShowcase => Some(Voucher::ThemePreview),
            Voucher::CrunchTime => Some(Voucher::Overtime),
            Voucher::EmptyTrash => Some(Voucher::RmI),
            Voucher::ShellPlugins => Some(Voucher::ShellAliases),
            Voucher::ThemeMarketplace => Some(Voucher::ThemeGallery),
            Voucher::IndexFund => Some(Voucher::HighYieldSavings),
            Voucher::FeatureFlag => Some(Voucher::LoremIpsum),
            Voucher::PremiumCards => Some(Voucher::CardMarketplace),
            Voucher::GitResetHard => Some(Voucher::GitRevert),
            Voucher::EmergencyChange => Some(Voucher::ChangeRequest),
            Voucher::Display4k => Some(Voucher::Hidpi),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vouchers_come_in_pairs_with_short_quips() {
        assert_eq!(
            Voucher::ALL
                .iter()
                .filter(|v| v.follows().is_some())
                .count(),
            16
        );
        for voucher in Voucher::ALL {
            assert_eq!(Voucher::ALL.iter().filter(|&&v| v == voucher).count(), 1);
            assert!(voucher.quip().chars().count() <= 50, "{}", voucher.quip());
        }
        assert_eq!(Voucher::FeatureFlag.follows(), Some(Voucher::LoremIpsum));
    }
}
