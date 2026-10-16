//! Translations. The engine speaks English; a `Session` set to another
//! language translates its responses on the way out, from the catalog
//! `locales/<lang>.json`: English text → translation. Keys with `{name}`
//! placeholders are templates ("+{n} mult"), for text built with numbers or
//! names. Card names and quips stay in English; so does the code.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lang {
    #[default]
    En,
    Fr,
}

impl Lang {
    pub const ALL: [Lang; 2] = [Lang::En, Lang::Fr];

    /// The language's own name, for the language choice.
    pub fn name(self) -> &'static str {
        match self {
            Lang::En => "English",
            Lang::Fr => "Français",
        }
    }

    fn source(self) -> Option<&'static str> {
        match self {
            Lang::En => None,
            Lang::Fr => Some(include_str!("../../locales/fr.json")),
        }
    }
}

/// Fields holding text to translate; `name` only when the catalog has it
/// word for word (hand types, the small and big blinds: card names stay).
const TEXT_FIELDS: [&str; 7] = [
    "description",
    "text",
    "current",
    "hand_name",
    "message",
    "problem",
    "category",
];

/// A template cut into literal text and placeholder names.
struct Template {
    parts: Vec<Part>,
    translation: String,
}

enum Part {
    Text(String),
    Slot(String),
}

pub struct Catalog {
    exact: HashMap<String, String>,
    /// Most literal text first, so that the most specific one wins.
    templates: Vec<Template>,
    decimal_comma: bool,
}

fn parse_template(key: &str) -> Vec<Part> {
    let mut parts = Vec::new();
    let mut rest = key;
    while let Some(open) = rest.find('{') {
        let Some(close) = rest[open..].find('}') else { break };
        if open > 0 {
            parts.push(Part::Text(rest[..open].to_string()));
        }
        parts.push(Part::Slot(rest[open + 1..open + close].to_string()));
        rest = &rest[open + close + 1..];
    }
    if !rest.is_empty() {
        parts.push(Part::Text(rest.to_string()));
    }
    parts
}

/// Matches the text against the parts; each slot takes the shortest text up
/// to the next literal part (the last slot, the rest).
fn capture(parts: &[Part], text: &str, slots: &mut Vec<(String, String)>) -> bool {
    match parts.split_first() {
        None => text.is_empty(),
        Some((Part::Text(literal), rest)) => {
            text.starts_with(literal.as_str()) && capture(rest, &text[literal.len()..], slots)
        }
        Some((Part::Slot(name), rest)) => {
            let ends: Vec<usize> = match rest.first() {
                None => vec![text.len()],
                Some(Part::Text(next)) => text.match_indices(next.as_str()).map(|(i, _)| i).collect(),
                Some(Part::Slot(_)) => return false,
            };
            for end in ends.into_iter().filter(|&e| e > 0) {
                slots.push((name.clone(), text[..end].to_string()));
                if capture(rest, &text[end..], slots) {
                    return true;
                }
                slots.pop();
            }
            false
        }
    }
}

impl Catalog {
    fn load(lang: Lang) -> Option<Catalog> {
        let source = lang.source()?;
        let table: HashMap<String, String> =
            serde_json::from_str(source).expect("the catalog is valid JSON");
        let mut exact = HashMap::new();
        let mut templates = Vec::new();
        for (key, translation) in table {
            if key.contains('{') {
                templates.push(Template {
                    parts: parse_template(&key),
                    translation,
                });
            } else {
                exact.insert(key, translation);
            }
        }
        let literal = |t: &Template| -> usize {
            t.parts
                .iter()
                .map(|p| match p {
                    Part::Text(s) => s.len(),
                    Part::Slot(_) => 0,
                })
                .sum()
        };
        templates.sort_by_key(|t| std::cmp::Reverse(literal(t)));
        Some(Catalog {
            exact,
            templates,
            decimal_comma: lang == Lang::Fr,
        })
    }

    /// A number as the language writes it: 2,5 and 12 345 in French.
    fn number(&self, text: &str) -> String {
        let numeric = !text.is_empty()
            && text.chars().any(|c| c.is_ascii_digit())
            && text
                .chars()
                .all(|c| c.is_ascii_digit() || matches!(c, '.' | ',' | '-' | '+' | '×' | '$'));
        if !self.decimal_comma || !numeric {
            return text.to_string();
        }
        text.replace(',', "\u{202f}").replace('.', ",")
    }

    /// The translation of a text, if the catalog has one (word for word, or
    /// through a template, whose slots are translated in turn).
    pub fn translate(&self, text: &str) -> Option<String> {
        if let Some(t) = self.exact.get(text) {
            return Some(t.clone());
        }
        for template in &self.templates {
            let mut slots = Vec::new();
            if capture(&template.parts, text, &mut slots) {
                let mut out = template.translation.clone();
                for (name, value) in slots {
                    // A slot holds text to translate in turn ("invalid
                    // selection: {why}"), a name, or a number.
                    let value = self
                        .translate(&value)
                        .unwrap_or_else(|| self.number(&value));
                    out = out.replace(&format!("{{{name}}}"), &value);
                }
                return Some(out);
            }
        }
        None
    }

    /// The catalog as it is, for the interface's own text.
    pub fn table(lang: Lang) -> HashMap<String, String> {
        lang.source()
            .map(|s| serde_json::from_str(s).expect("the catalog is valid JSON"))
            .unwrap_or_default()
    }
}

pub fn catalog(lang: Lang) -> Option<&'static Catalog> {
    static FR: OnceLock<Option<Catalog>> = OnceLock::new();
    match lang {
        Lang::En => None,
        Lang::Fr => FR.get_or_init(|| Catalog::load(Lang::Fr)).as_ref(),
    }
}

thread_local! {
    /// Texts left untranslated, when recording (the bot's `--lang`).
    static MISSES: RefCell<Option<BTreeSet<String>>> = const { RefCell::new(None) };
}

/// Starts recording the texts a translation misses (on this thread).
pub fn record_misses() {
    MISSES.with(|m| *m.borrow_mut() = Some(BTreeSet::new()));
}

/// The texts missed since `record_misses`.
pub fn take_misses() -> BTreeSet<String> {
    MISSES.with(|m| m.borrow_mut().as_mut().map(std::mem::take).unwrap_or_default())
}

fn missed(text: &str) {
    MISSES.with(|m| {
        if let Some(set) = m.borrow_mut().as_mut() {
            set.insert(text.to_string());
        }
    });
}

/// Translates a response in place.
pub fn translate_response(value: &mut Value, lang: Lang) {
    if let Some(catalog) = catalog(lang) {
        walk(value, catalog);
    }
}

fn walk(value: &mut Value, catalog: &Catalog) {
    match value {
        Value::Object(map) => {
            for (key, field) in map.iter_mut() {
                match (key.as_str(), &mut *field) {
                    ("name", Value::String(s)) => {
                        if let Some(t) = catalog.exact.get(s.as_str()) {
                            *s = t.clone();
                        }
                    }
                    ("notes", Value::Array(notes)) => {
                        for note in notes {
                            if let Value::String(s) = note {
                                translate_text(s, catalog);
                            }
                        }
                    }
                    (k, Value::String(s)) if TEXT_FIELDS.contains(&k) => translate_text(s, catalog),
                    _ => walk(field, catalog),
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|v| walk(v, catalog)),
        _ => {}
    }
}

fn translate_text(text: &mut String, catalog: &Catalog) {
    if text.is_empty() {
        return;
    }
    match catalog.translate(text) {
        Some(t) => *text = t,
        None => missed(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fr(text: &str) -> Option<String> {
        catalog(Lang::Fr).unwrap().translate(text)
    }

    #[test]
    fn templates_capture_and_translate_their_slots() {
        let parts = parse_template("+{n} mult per {what}");
        let mut slots = Vec::new();
        assert!(capture(&parts, "+3 mult per joker", &mut slots));
        assert_eq!(slots, [("n".into(), "3".into()), ("what".into(), "joker".into())]);
        assert!(!capture(&parts, "+3 chips per joker", &mut Vec::new()));
    }

    #[test]
    fn french_numbers() {
        let c = catalog(Lang::Fr).unwrap();
        assert_eq!(c.number("2.5"), "2,5");
        assert_eq!(c.number("12,345"), "12\u{202f}345");
        assert_eq!(c.number("Fork"), "Fork");
        assert!(fr("Small Blind").is_some());
        assert_eq!(
            fr("invalid selection: choose 1 to 5 cards").unwrap(),
            "sélection invalide : choisissez de 1 à 5 cartes"
        );
        assert_eq!(fr("$4 for Two Pair").unwrap(), "4 $ pour Double paire");
        assert_eq!(fr("×1.5 mult").unwrap(), "×1,5 mult");
    }

    /// Every content text has its translation: a new card without one fails
    /// here (add it to `locales/fr.json`).
    #[test]
    fn the_catalog_covers_the_content() {
        let mut texts: Vec<String> = Vec::new();
        for term in crate::glossary::glossary() {
            texts.push(term.category);
            if !term.description.is_empty() {
                texts.push(term.description);
            }
        }
        texts.extend(crate::run_deck::Deck::ALL.iter().map(|d| d.description().to_string()));
        texts.extend(crate::boss::Boss::ALL.iter().map(|b| b.description().to_string()));
        texts.extend(crate::hand::HandKind::ALL.iter().map(|h| h.name().to_string()));
        let missing: Vec<&String> = texts.iter().filter(|t| fr(t).is_none()).collect();
        assert!(missing.is_empty(), "untranslated: {missing:#?}");
    }
}
