//! The terms card texts mention, with what they mean: the interface explains
//! them beside a card that mentions them, as Balatro does.

use crate::consumable::{Command, Oligarchy, Theme};
use crate::hand::HandKind;
use crate::joker::Joker;
use crate::modifier::{Edition, Enhancement, Seal};
use crate::pack::PackKind;
use crate::stake::Sticker;
use crate::tag::Tag;
use crate::voucher::Voucher;
use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Term {
    pub name: String,
    /// What it is: "edition", "pull request", "joker · AUR"…
    pub category: String,
    /// What it does; empty for words that only keep a shorter name from
    /// matching inside them (hand types: "Fork Bomb" is not the joker Fork).
    pub description: String,
}

fn term(
    name: impl Into<String>,
    category: impl Into<String>,
    description: impl Into<String>,
) -> Term {
    Term {
        name: name.into(),
        category: category.into(),
        description: description.into(),
    }
}

/// Every term, from the content itself.
pub fn glossary() -> Vec<Term> {
    let mut terms = Vec::new();
    for e in Enhancement::ALL {
        terms.push(term(e.name(), "enhancement", e.description()));
    }
    for s in Seal::ALL {
        terms.push(term(s.name(), "seal", s.description()));
    }
    for e in Edition::ALL {
        terms.push(term(e.name(), "edition", e.description()));
    }
    for s in Sticker::ALL {
        terms.push(term(s.name(), "sticker", s.description()));
    }
    for t in Tag::ALL {
        terms.push(term(t.name(), "badge", t.description()));
    }
    for v in Voucher::ALL {
        terms.push(term(v.name(), "pull request", v.description()));
    }
    for c in Command::ALL {
        terms.push(term(c.name(), "command", c.description()));
    }
    for o in Oligarchy::ALL {
        terms.push(term(o.name(), "oligarchy card", o.description()));
    }
    for t in Theme::ALL {
        terms.push(term(t.name(), "theme", t.description()));
    }
    for j in Joker::ALL {
        terms.push(term(
            j.name(),
            format!("joker · {}", j.rarity().name()),
            j.description(),
        ));
    }
    for p in PackKind::ALL {
        terms.push(term(
            p.name(),
            "package",
            format!(
                "Bought in the shop and opened on the spot: pick from its {}",
                p.contents()
            ),
        ));
    }
    terms.push(term(
        "oligarchy card",
        "consumable",
        "Powerful, at a price: from oligarchy packages and a few jokers, never from the shop",
    ));
    for h in HandKind::ALL {
        terms.push(term(h.name(), "hand", ""));
    }
    terms
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn every_name_means_one_thing() {
        let mut seen: HashMap<String, String> = HashMap::new();
        for t in glossary() {
            if let Some(category) = seen.insert(t.name.clone(), t.category.clone()) {
                panic!("{:?} is both a {category} and a {}", t.name, t.category);
            }
        }
    }
}
