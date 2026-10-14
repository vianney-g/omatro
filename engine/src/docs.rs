//! Human-readable references in `docs/`, generated from the code. A test fails
//! when one no longer matches; `make docs` rewrites them.

use crate::hand::HandKind;
use crate::joker::Joker;

/// Snake-case id of a value, as in the protocol.
fn id(value: impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
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
    jokers.sort_by_key(|j| j.price());
    let mut doc = header(
        "Jokers",
        "Jokers are bought in the shop between blinds (5 slots at most) and sell for\n\
         half their price. While a hand is scored, card jokers trigger for each scoring\n\
         card right after its chips; hand jokers then trigger once, left to right.",
    );
    doc += "| Id | Name | Price | Sells for | Effect | Quip |\n|---|---|---|---|---|---|\n";
    for joker in jokers {
        doc += &format!(
            "| `{}` | {} | ${} | ${} | {} | *{}* |\n",
            id(joker),
            joker.name(),
            joker.price(),
            joker.sell_value(),
            joker.description(),
            joker.quip()
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
            format!("yes: *{}*", kind.quip().replace('|', "\\|"))
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

#[test]
fn docs_are_up_to_date() {
    let docs: [(&str, fn() -> String); 2] = [("jokers.md", jokers), ("hands.md", hands)];
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
