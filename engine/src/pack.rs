//! Booster packs, named after Arch packages: bought in the shop, opened on the
//! spot to pick one or two of their cards.

use serde::{Deserialize, Serialize};

/// Packs offered by each shop, besides the cards.
pub const SHOP_PACKS: usize = 2;

/// What a pack holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackKind {
    /// Commands, used on the spot on a hand drawn from the deck.
    Coreutils,
    /// Themes, applied on the spot.
    Base16,
    /// Playing cards for the deck, often with modifiers.
    BaseDevel,
    /// Jokers.
    JokersGit,
}

impl PackKind {
    pub const ALL: [PackKind; 4] = [
        PackKind::Coreutils,
        PackKind::Base16,
        PackKind::BaseDevel,
        PackKind::JokersGit,
    ];

    pub fn name(self) -> &'static str {
        match self {
            PackKind::Coreutils => "coreutils",
            PackKind::Base16 => "base16",
            PackKind::BaseDevel => "base-devel",
            PackKind::JokersGit => "jokers-git",
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            PackKind::Coreutils => "ls, cp, mv… pick one.",
            PackKind::Base16 => "Sixteen colors. Pick a vibe.",
            PackKind::BaseDevel => "Everything you need to build a deck.",
            PackKind::JokersGit => "Unstable, untested, irresistible.",
        }
    }

    /// What the cards are called, for "Pick 1 of 3 commands".
    pub fn contents(self) -> &'static str {
        match self {
            PackKind::Coreutils => "commands",
            PackKind::Base16 => "themes",
            PackKind::BaseDevel => "cards",
            PackKind::JokersGit => "jokers",
        }
    }

    /// How often a shop draws it, for a normal size.
    fn weight(self) -> f64 {
        match self {
            PackKind::Coreutils | PackKind::Base16 | PackKind::BaseDevel => 4.0,
            PackKind::JokersGit => 1.2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackSize {
    Normal,
    /// Jumbo, in the original.
    Full,
    /// Mega: a metapackage installs two.
    Meta,
}

impl PackSize {
    pub const ALL: [PackSize; 3] = [PackSize::Normal, PackSize::Full, PackSize::Meta];

    pub fn suffix(self) -> &'static str {
        match self {
            PackSize::Normal => "",
            PackSize::Full => "-full",
            PackSize::Meta => "-meta",
        }
    }

    pub fn price(self) -> u32 {
        match self {
            PackSize::Normal => 4,
            PackSize::Full => 6,
            PackSize::Meta => 8,
        }
    }

    /// Share of the pack kind's weight.
    fn weight(self) -> f64 {
        match self {
            PackSize::Normal => 1.0,
            PackSize::Full => 0.5,
            PackSize::Meta => 0.125,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Pack {
    pub kind: PackKind,
    pub size: PackSize,
}

impl Pack {
    /// Every pack, with its chance to fill a shop slot.
    pub fn odds() -> Vec<(Pack, f64)> {
        let all: Vec<(Pack, f64)> = PackKind::ALL
            .iter()
            .flat_map(|&kind| {
                PackSize::ALL
                    .iter()
                    .map(move |&size| (Pack { kind, size }, kind.weight() * size.weight()))
            })
            .collect();
        let total: f64 = all.iter().map(|(_, w)| w).sum();
        all.into_iter().map(|(pack, w)| (pack, w / total)).collect()
    }

    pub fn name(self) -> String {
        format!("{}{}", self.kind.name(), self.size.suffix())
    }

    pub fn price(self) -> u32 {
        self.size.price()
    }

    /// Cards shown; jokers come one fewer.
    pub fn shown(self) -> usize {
        let shown = match self.size {
            PackSize::Normal => 3,
            PackSize::Full | PackSize::Meta => 5,
        };
        if self.kind == PackKind::JokersGit {
            shown - 1
        } else {
            shown
        }
    }

    /// Cards to pick.
    pub fn picks(self) -> usize {
        match self.size {
            PackSize::Meta => 2,
            PackSize::Normal | PackSize::Full => 1,
        }
    }

    /// "Pick 1 of 3 commands".
    pub fn description(self) -> String {
        format!(
            "Pick {} of {} {}",
            self.picks(),
            self.shown(),
            self.kind.contents()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packs_follow_the_original_sizes() {
        let pack = |kind, size| Pack { kind, size };
        assert_eq!(
            pack(PackKind::Coreutils, PackSize::Full).name(),
            "coreutils-full"
        );
        assert_eq!(
            pack(PackKind::Base16, PackSize::Meta).description(),
            "Pick 2 of 5 themes"
        );
        assert_eq!(
            pack(PackKind::JokersGit, PackSize::Normal).description(),
            "Pick 1 of 2 jokers"
        );
        assert_eq!(pack(PackKind::BaseDevel, PackSize::Meta).price(), 8);
    }

    #[test]
    fn odds_add_up_and_favour_normal_packs() {
        let odds = Pack::odds();
        let total: f64 = odds.iter().map(|(_, w)| w).sum();
        assert!((total - 1.0).abs() < 1e-9);
        let of = |kind, size| {
            odds.iter()
                .find(|(p, _)| *p == Pack { kind, size })
                .unwrap()
                .1
        };
        assert!(
            of(PackKind::Coreutils, PackSize::Normal) > of(PackKind::Coreutils, PackSize::Meta)
        );
        assert!(of(PackKind::Base16, PackSize::Normal) > of(PackKind::JokersGit, PackSize::Normal));
    }
}
