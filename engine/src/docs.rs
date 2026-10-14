//! Human-readable references in `docs/`, generated from the code. A test fails
//! when one no longer matches; `make docs` rewrites them.

use crate::consumable::Theme;
use crate::hand::HandKind;
use crate::joker::{Edition, Joker};
use crate::modifier::{Enhancement, Seal};

/// Snake-case id of a value, as in the protocol.
fn id(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

/// Escapes a table cell.
fn cell(text: &str) -> String {
    text.replace('|', "\\|")
}

fn header(title: &str, intro: &str) -> String {
    format!(
        "# {title}\n\n\
         <!-- Generated from the engine code by `make docs`: do not edit by hand. -->\n\n\
         {intro}\n\n"
    )
}

fn jokers() -> String {
    let mut jokers = Joker::ALL.to_vec();
    jokers.sort_by_key(|j| (j.rarity(), j.price()));
    let mut doc = header(
        "Jokers",
        "Jokers are bought in the shop between blinds (5 slots at most) and sell for\n\
         half their price. Each shop slot draws a rarity, named after Arch's\n\
         repositories: core (70%), extra (25%) or AUR (5%); legendary jokers never\n\
         show up in the shop. A joker may come with an edition (see\n\
         [editions.md](editions.md)).\n\n\
         While a hand is scored, each scoring card scores its chips, then card jokers\n\
         trigger, and retriggers score it again; then jokers trigger for the cards\n\
         held in hand; then each joker, left to right, adds its effect. Order matters:\n\
         Fork copies the joker on its right.",
    );
    doc += "| Id | Name | Rarity | Price | Effect | Quip |\n|---|---|---|---|---|---|\n";
    for joker in jokers {
        doc += &format!(
            "| `{}` | {} | {} | ${} | {} | *{}* |\n",
            id(joker),
            joker.name(),
            joker.rarity().name(),
            joker.price(),
            joker.description(),
            cell(joker.quip())
        );
    }
    doc
}

fn editions() -> String {
    let mut doc = header(
        "Editions",
        "Hyprland decorations a joker may come with in the shop, for a higher price.\n\
         Blur and Shadow apply before the joker's own effect, Animated after it.",
    );
    doc += "| Id | Name | Effect | Extra price | Shop odds | Quip |\n|---|---|---|---|---|---|\n";
    for edition in Edition::ALL {
        doc += &format!(
            "| `{}` | {} | {} | +${} | {}% | *{}* |\n",
            id(edition),
            edition.name(),
            edition.description(),
            edition.price(),
            edition.odds() * 100.0,
            cell(edition.quip())
        );
    }
    doc
}

fn hands() -> String {
    let mut doc = header(
        "Poker hands",
        "A hand scores its chips and mult at its current level, plus the chips of its\n\
         scoring cards, before the jokers. Each level past the first adds the per-level\n\
         chips and mult. The secret hands need duplicate cards; they stay hidden in the\n\
         game until played.",
    );
    doc += "| Id | Hand | Level 1 | Per level | Secret |\n|---|---|---|---|---|\n";
    for kind in HandKind::ALL.iter().rev() {
        let (chips, mult) = kind.base();
        let (step_chips, step_mult) = kind.per_level();
        let secret = if kind.secret() {
            format!("yes: *{}*", cell(kind.quip()))
        } else {
            String::new()
        };
        doc += &format!(
            "| `{}` | {} | {chips} × {mult} | +{step_chips} × +{step_mult} | {secret} |\n",
            id(kind),
            kind.name()
        );
    }
    doc
}

fn themes() -> String {
    let mut doc = header(
        "Themes",
        "Themes are consumables (planets, in the original): bought in the shop for $3,\n\
         they wait in the clipboard (2 slots) until applied, at any time; each one\n\
         levels up a poker hand type (see [hands.md](hands.md)). The themes of the\n\
         secret hands only show up once the hand was played.",
    );
    doc += "| Id | Theme | Effect | Quip |\n|---|---|---|---|\n";
    for theme in Theme::ALL {
        doc += &format!(
            "| `{}` | {} | {} | *{}* |\n",
            id(theme),
            theme.name(),
            theme.description(),
            cell(theme.quip())
        );
    }
    doc
}

fn modifiers() -> String {
    let mut doc = header(
        "Card modifiers",
        "Each card of the deck may carry an enhancement, a seal and an edition, for the\n\
         whole game. When a card scores: its chips (with Optimized, and what it gained),\n\
         then its enhancement, its edition, its seal, and the jokers; Retry and jokers\n\
         may score it again. Cards held in hand then trigger (Daemon), once more per\n\
         Cron Job.",
    );
    let table = |title: &str, rows: Vec<(String, &str, &str, &str)>| {
        let mut part = format!("## {title}\n\n| Id | Name | Effect | Quip |\n|---|---|---|---|\n");
        for (id, name, effect, quip) in rows {
            part += &format!("| `{id}` | {name} | {effect} | *{}* |\n", cell(quip));
        }
        part
    };
    doc += &table(
        "Enhancements",
        Enhancement::ALL
            .iter()
            .map(|&e| (id(e), e.name(), e.description(), e.quip()))
            .collect(),
    );
    doc += "\n";
    doc += &table(
        "Seals",
        Seal::ALL
            .iter()
            .map(|&s| (id(s), s.name(), s.description(), s.quip()))
            .collect(),
    );
    doc += "\nEditions are the same as on jokers ([editions.md](editions.md)); on a card,\n\
            Transparent adds one card to the hand while the card is in it.\n";
    doc
}

#[test]
fn docs_are_up_to_date() {
    let docs: [(&str, fn() -> String); 5] = [
        ("jokers.md", jokers),
        ("editions.md", editions),
        ("hands.md", hands),
        ("themes.md", themes),
        ("modifiers.md", modifiers),
    ];
    for (file, render) in docs {
        let path = format!("{}/../docs/{file}", env!("CARGO_MANIFEST_DIR"));
        if std::env::var_os("UPDATE_DOCS").is_some() {
            std::fs::write(&path, render()).unwrap();
        }
        let current = std::fs::read_to_string(&path).unwrap_or_default();
        assert!(
            current == render(),
            "docs/{file} is out of date: run `make docs`"
        );
    }
}
