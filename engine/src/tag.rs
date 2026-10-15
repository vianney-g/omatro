//! Badges (tags, in the original): README badges won by skipping a blind. Some
//! act right away, others at the next shop, blind or boss.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tag {
    Stars100,
    Stars10k,
    Opacity0,
    BlurOn,
    ShadowOn,
    AnimationsOn,
    SeriesA,
    PrsWelcome,
    Wontfix,
    BuildPassing,
    DocsLatest,
    ThemeDark,
    FunYes,
    FundingOligarchy,
    Commits1k,
    GcDone,
    PriceFree,
    Badges2x,
    Resolution4k,
    CacheHit,
    Contributors2,
    SpeedBlazing,
    Version3,
    Sponsors2x,
}

impl Tag {
    pub const ALL: [Tag; 24] = [
        Tag::Stars100,
        Tag::Stars10k,
        Tag::Opacity0,
        Tag::BlurOn,
        Tag::ShadowOn,
        Tag::AnimationsOn,
        Tag::SeriesA,
        Tag::PrsWelcome,
        Tag::Wontfix,
        Tag::BuildPassing,
        Tag::DocsLatest,
        Tag::ThemeDark,
        Tag::FunYes,
        Tag::FundingOligarchy,
        Tag::Commits1k,
        Tag::GcDone,
        Tag::PriceFree,
        Tag::Badges2x,
        Tag::Resolution4k,
        Tag::CacheHit,
        Tag::Contributors2,
        Tag::SpeedBlazing,
        Tag::Version3,
        Tag::Sponsors2x,
    ];

    /// Left part of the badge, in grey.
    pub fn label(self) -> &'static str {
        match self {
            Tag::Stars100 => "stars",
            Tag::Stars10k => "stars",
            Tag::Opacity0 => "opacity",
            Tag::BlurOn => "blur",
            Tag::ShadowOn => "shadow",
            Tag::AnimationsOn => "animations",
            Tag::SeriesA => "funding",
            Tag::PrsWelcome => "PRs",
            Tag::Wontfix => "wontfix",
            Tag::BuildPassing => "build",
            Tag::DocsLatest => "docs",
            Tag::ThemeDark => "theme",
            Tag::FunYes => "fun",
            Tag::FundingOligarchy => "funding",
            Tag::Commits1k => "commits",
            Tag::GcDone => "gc",
            Tag::PriceFree => "price",
            Tag::Badges2x => "badges",
            Tag::Resolution4k => "resolution",
            Tag::CacheHit => "cache",
            Tag::Contributors2 => "contributors",
            Tag::SpeedBlazing => "speed",
            Tag::Version3 => "version",
            Tag::Sponsors2x => "sponsors",
        }
    }

    /// Right part of the badge, in colour (empty for a single-part badge).
    pub fn value(self) -> &'static str {
        match self {
            Tag::Stars100 => "100",
            Tag::Stars10k => "10k",
            Tag::Opacity0 => "0",
            Tag::BlurOn => "on",
            Tag::ShadowOn => "on",
            Tag::AnimationsOn => "on",
            Tag::SeriesA => "series A",
            Tag::PrsWelcome => "welcome",
            Tag::Wontfix => "",
            Tag::BuildPassing => "passing",
            Tag::DocsLatest => "latest",
            Tag::ThemeDark => "dark",
            Tag::FunYes => "yes",
            Tag::FundingOligarchy => "oligarchy",
            Tag::Commits1k => "1k",
            Tag::GcDone => "done",
            Tag::PriceFree => "free",
            Tag::Badges2x => "2x",
            Tag::Resolution4k => "4k",
            Tag::CacheHit => "hit",
            Tag::Contributors2 => "2",
            Tag::SpeedBlazing => "blazing",
            Tag::Version3 => "3.0",
            Tag::Sponsors2x => "2x",
        }
    }

    /// "build: passing", or "wontfix".
    pub fn name(self) -> String {
        if self.value().is_empty() {
            self.label().to_string()
        } else {
            format!("{}: {}", self.label(), self.value())
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Tag::Stars100 => "The next shop has a free extra joker",
            Tag::Stars10k => "The next shop has a free AUR joker",
            Tag::Opacity0 => "The next shop joker becomes Transparent, for free",
            Tag::BlurOn => "The next shop joker gets Blur, for free",
            Tag::ShadowOn => "The next shop joker gets Shadow, for free",
            Tag::AnimationsOn => "The next shop joker gets Animated, for free",
            Tag::SeriesA => "+$25 after beating the next boss",
            Tag::PrsWelcome => "Adds a pull request to the next shop",
            Tag::Wontfix => "Rerolls the next boss",
            Tag::BuildPassing => "Opens a free base-devel-meta pack",
            Tag::DocsLatest => "Opens a free coreutils-meta pack",
            Tag::ThemeDark => "Opens a free base16-meta pack",
            Tag::FunYes => "Opens a free jokers-git-meta pack",
            Tag::FundingOligarchy => "Opens a free oligarchy pack",
            Tag::Commits1k => "+$1 per hand played this game",
            Tag::GcDone => "+$1 per discard left unused this game",
            Tag::PriceFree => "Cards and packs in the next shop are free",
            Tag::Badges2x => "Copies the next badge you get",
            Tag::Resolution4k => "+3 hand size for the next blind",
            Tag::CacheHit => "Rerolls in the next shop start at $0",
            Tag::Contributors2 => "Creates up to 2 core jokers",
            Tag::SpeedBlazing => "+$5 per blind skipped this game",
            Tag::Version3 => "A random hand type gains 3 levels",
            Tag::Sponsors2x => "Doubles your money (+$40 at most)",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Tag::Stars100 => "Popular enough.",
            Tag::Stars10k => "Trending on GitHub.",
            Tag::Opacity0 => "You can't see it. It's there.",
            Tag::BlurOn => "Looks expensive. Isn't.",
            Tag::ShadowOn => "Depth, on the house.",
            Tag::AnimationsOn => "Wobbly, and free.",
            Tag::SeriesA => "Burn rate: acceptable.",
            Tag::PrsWelcome => "Contributions encouraged.",
            Tag::Wontfix => "Closed as not planned.",
            Tag::BuildPassing => "Green across the board.",
            Tag::DocsLatest => "RTFM, generously.",
            Tag::ThemeDark => "Light mode not found.",
            Tag::FunYes => "Jokes included.",
            Tag::FundingOligarchy => "Elite capital incoming.",
            Tag::Commits1k => "Small commits, often.",
            Tag::GcDone => "Memory reclaimed. Money too.",
            Tag::PriceFree => "Free as in beer. Again.",
            Tag::Badges2x => "A badge for your badge.",
            Tag::Resolution4k => "Temporarily huge.",
            Tag::CacheHit => "Served from cache.",
            Tag::Contributors2 => "New contributors welcome.",
            Tag::SpeedBlazing => "Blazingly fast.",
            Tag::Version3 => "Major release. Three of them.",
            Tag::Sponsors2x => "Matched by your sponsors.",
        }
    }

    /// First ante it may be offered at.
    pub fn min_ante(self) -> u32 {
        match self {
            Tag::Opacity0
            | Tag::BuildPassing
            | Tag::ThemeDark
            | Tag::FunYes
            | Tag::FundingOligarchy
            | Tag::Commits1k
            | Tag::GcDone
            | Tag::Contributors2
            | Tag::Version3 => 2,
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badges_have_names_and_short_quips() {
        for tag in Tag::ALL {
            assert_eq!(Tag::ALL.iter().filter(|&&t| t == tag).count(), 1);
            assert!(tag.quip().chars().count() <= 50, "{}", tag.quip());
        }
        assert_eq!(Tag::BuildPassing.name(), "build: passing");
        assert_eq!(Tag::Wontfix.name(), "wontfix");
    }
}
