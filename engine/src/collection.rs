//! The collection: everything a game can hold, which the player finds as
//! they obtain it (a joker bought, a badge won, a boss beaten…), and the
//! hidden items achievements unlock. The profile keeps what was found.

use crate::achievement::Achievement;
use crate::boss::Boss;
use crate::consumable::{Command, Consumable, Oligarchy, Theme};
use crate::hand::HandKind;
use crate::joker::{Joker, Rarity};
use crate::modifier::{Edition, Enhancement, Seal};
use crate::pack::PackKind;
use crate::profile::{Outcome, Profile};
use crate::run_deck::Deck;
use crate::stake::Stake;
use crate::tag::Tag;
use crate::voucher::Voucher;
use serde::Serialize;
use serde::de::DeserializeOwned;

/// A family of items, one tab of the collection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Family {
    Joker,
    Deck,
    Theme,
    Command,
    Oligarchy,
    Voucher,
    Tag,
    Boss,
    Pack,
    Enhancement,
    Seal,
    Edition,
    Hand,
}

impl Family {
    pub const ALL: [Family; 13] = [
        Family::Joker,
        Family::Deck,
        Family::Theme,
        Family::Command,
        Family::Oligarchy,
        Family::Voucher,
        Family::Tag,
        Family::Boss,
        Family::Pack,
        Family::Enhancement,
        Family::Seal,
        Family::Edition,
        Family::Hand,
    ];

    /// The family's name, as a tab says it.
    pub fn name(self) -> &'static str {
        match self {
            Family::Joker => "Jokers",
            Family::Deck => "Decks",
            Family::Theme => "Themes",
            Family::Command => "Commands",
            Family::Oligarchy => "Oligarchy cards",
            Family::Voucher => "Pull requests",
            Family::Tag => "Badges",
            Family::Boss => "Bosses",
            Family::Pack => "Packages",
            Family::Enhancement => "Enhancements",
            Family::Seal => "Seals",
            Family::Edition => "Editions",
            Family::Hand => "Poker hands",
        }
    }

    /// One item of the family, for the news ("New: Fork, joker").
    pub fn singular(self) -> &'static str {
        match self {
            Family::Joker => "joker",
            Family::Deck => "deck",
            Family::Theme => "theme",
            Family::Command => "command",
            Family::Oligarchy => "oligarchy card",
            Family::Voucher => "pull request",
            Family::Tag => "badge",
            Family::Boss => "boss",
            Family::Pack => "package",
            Family::Enhancement => "enhancement",
            Family::Seal => "seal",
            Family::Edition => "edition",
            Family::Hand => "hand",
        }
    }

    fn id(self) -> String {
        id_of(self)
    }
}

/// Anything the collection lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item {
    Joker(Joker),
    Deck(Deck),
    Consumable(Consumable),
    Voucher(Voucher),
    Tag(Tag),
    Boss(Boss),
    Pack(PackKind),
    Enhancement(Enhancement),
    Seal(Seal),
    Edition(Edition),
    Hand(HandKind),
}

fn id_of(value: impl Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

impl Item {
    /// Every item, family by family.
    pub fn all() -> Vec<Item> {
        let mut items: Vec<Item> = Vec::new();
        items.extend(Joker::ALL.map(Item::Joker));
        items.extend(Deck::ALL.map(Item::Deck));
        items.extend(Theme::ALL.map(|t| Item::Consumable(Consumable::Theme(t))));
        items.extend(Command::ALL.map(|c| Item::Consumable(Consumable::Command(c))));
        items.extend(Oligarchy::ALL.map(|o| Item::Consumable(Consumable::Oligarchy(o))));
        items.extend(Voucher::ALL.map(Item::Voucher));
        items.extend(Tag::ALL.map(Item::Tag));
        items.extend(Boss::ALL.map(Item::Boss));
        items.extend(PackKind::ALL.map(Item::Pack));
        items.extend(Enhancement::ALL.map(Item::Enhancement));
        items.extend(Seal::ALL.map(Item::Seal));
        items.extend(Edition::ALL.map(Item::Edition));
        items.extend(HandKind::ALL.map(Item::Hand));
        items
    }

    pub fn family(self) -> Family {
        match self {
            Item::Joker(_) => Family::Joker,
            Item::Deck(_) => Family::Deck,
            Item::Consumable(Consumable::Theme(_)) => Family::Theme,
            Item::Consumable(Consumable::Command(_)) => Family::Command,
            Item::Consumable(Consumable::Oligarchy(_)) => Family::Oligarchy,
            Item::Voucher(_) => Family::Voucher,
            Item::Tag(_) => Family::Tag,
            Item::Boss(_) => Family::Boss,
            Item::Pack(_) => Family::Pack,
            Item::Enhancement(_) => Family::Enhancement,
            Item::Seal(_) => Family::Seal,
            Item::Edition(_) => Family::Edition,
            Item::Hand(_) => Family::Hand,
        }
    }

    /// A stable id, `family:id` ("joker:fork"): what the profile keeps.
    pub fn id(self) -> String {
        let inner = match self {
            Item::Joker(j) => id_of(j),
            Item::Deck(d) => id_of(d),
            Item::Consumable(Consumable::Theme(t)) => id_of(t),
            Item::Consumable(Consumable::Command(c)) => id_of(c),
            Item::Consumable(Consumable::Oligarchy(o)) => id_of(o),
            Item::Voucher(v) => id_of(v),
            Item::Tag(t) => id_of(t),
            Item::Boss(b) => id_of(b),
            Item::Pack(p) => id_of(p),
            Item::Enhancement(e) => id_of(e),
            Item::Seal(s) => id_of(s),
            Item::Edition(e) => id_of(e),
            Item::Hand(h) => id_of(h),
        };
        format!("{}:{inner}", self.family().id())
    }

    pub fn name(self) -> String {
        match self {
            Item::Joker(j) => j.name().into(),
            Item::Deck(d) => d.name().into(),
            Item::Consumable(c) => c.name().into(),
            Item::Voucher(v) => v.name().into(),
            Item::Tag(t) => t.name(),
            Item::Boss(b) => b.name().into(),
            Item::Pack(p) => p.name().into(),
            Item::Enhancement(e) => e.name().into(),
            Item::Seal(s) => s.name().into(),
            Item::Edition(e) => e.name().into(),
            Item::Hand(h) => h.name().into(),
        }
    }

    pub fn description(self) -> String {
        match self {
            Item::Joker(j) => j.description().into(),
            Item::Deck(d) => d.description().into(),
            Item::Consumable(c) => c.description(),
            Item::Voucher(v) => v.description().into(),
            Item::Tag(t) => t.description().into(),
            Item::Boss(b) => b.description().into(),
            Item::Pack(p) => format!(
                "Bought in the shop and opened on the spot: pick from its {}",
                p.contents()
            ),
            Item::Enhancement(e) => e.description().into(),
            Item::Seal(s) => s.description().into(),
            Item::Edition(e) => e.description().into(),
            Item::Hand(_) => String::new(),
        }
    }

    pub fn quip(self) -> &'static str {
        match self {
            Item::Joker(j) => j.quip(),
            Item::Deck(d) => d.quip(),
            Item::Consumable(c) => c.quip(),
            Item::Voucher(v) => v.quip(),
            Item::Tag(t) => t.quip(),
            Item::Boss(b) => b.quip(),
            Item::Pack(p) => p.quip(),
            Item::Enhancement(e) => e.quip(),
            Item::Seal(s) => s.quip(),
            Item::Edition(e) => e.quip(),
            Item::Hand(h) => h.quip(),
        }
    }

    /// Shown as "?????" until found, rather than a plain "?": legendary
    /// jokers, secret hands, and what achievements unlock.
    pub fn secret(self) -> bool {
        match self {
            Item::Joker(j) if j.rarity() == Rarity::Legendary => true,
            Item::Hand(h) => h.secret(),
            item => Achievement::unlocking(item).is_some(),
        }
    }
}

/// An item as the collection shows it: its text only once found.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ItemInfo {
    pub id: String,
    /// `found`, `unknown`, or `locked` (hidden until its achievement).
    pub state: &'static str,
    pub secret: bool,
    pub name: Option<String>,
    pub description: Option<String>,
    pub quip: Option<&'static str>,
    /// A joker's rarity.
    pub rarity: Option<Rarity>,
    /// What unlocks a locked item: its achievement.
    pub achievement: Option<&'static str>,
    pub condition: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FamilyInfo {
    pub id: Family,
    pub name: &'static str,
    pub found: usize,
    pub total: usize,
    pub items: Vec<ItemInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AchievementInfo {
    pub id: Achievement,
    pub name: &'static str,
    pub description: &'static str,
    pub quip: &'static str,
    pub done: bool,
    /// Where a counting achievement stands, and its goal.
    pub progress: Option<u64>,
    pub goal: Option<u64>,
    /// What it unlocks: its name once done, its family before.
    pub unlocks: Option<String>,
    pub unlocks_family: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Collection {
    pub found: usize,
    pub total: usize,
    pub families: Vec<FamilyInfo>,
    pub achievements: Vec<AchievementInfo>,
    pub stats: StatsInfo,
    /// The latest games, the last one first.
    pub history: Vec<RecordInfo>,
}

/// The lifetime stats, for the collection's stats tab.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct StatsInfo {
    /// Games ended: won, lost or abandoned.
    pub games: u64,
    pub won: u64,
    pub lost: u64,
    pub abandoned: u64,
    pub streak: u64,
    pub best_streak: u64,
    pub best_ante: u32,
    pub best_hand: f64,
    pub blinds_won: u64,
    pub hands_played: u64,
    pub jokers_sold: u64,
    /// Decks played, most games first.
    pub decks: Vec<DeckStatsInfo>,
    /// Hand types played, most played first.
    pub hands: Vec<HandStatsInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DeckStatsInfo {
    /// The deck's name (none: the plain deck).
    pub deck: Option<&'static str>,
    pub games: u64,
    pub won: u64,
    pub best_ante: u32,
    /// The highest difficulty level won with it.
    pub best_stake: Option<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HandStatsInfo {
    pub kind: HandKind,
    pub hand_name: &'static str,
    pub played: u64,
}

/// A game of the history, its items by name.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RecordInfo {
    /// When it ended, in seconds since 1970.
    pub ended: u64,
    pub seed: u64,
    /// The seed as the player types it.
    pub seed_code: String,
    /// The deck's name (none: the plain deck).
    pub deck: Option<&'static str>,
    pub stake: &'static str,
    pub outcome: Outcome,
    pub ante: u32,
    /// The boss whose blind ended the game.
    pub boss: Option<&'static str>,
    pub best_hand: f64,
    /// The hand type played most.
    pub hand_name: Option<&'static str>,
    pub jokers: Vec<&'static str>,
}

/// An item from its id, as the profile keeps it (none if it is gone).
fn from_id<T: DeserializeOwned>(id: &str) -> Option<T> {
    serde_json::from_value(serde_json::Value::String(id.to_string())).ok()
}

fn stats_info(profile: &Profile) -> StatsInfo {
    let s = &profile.stats;
    let mut decks: Vec<DeckStatsInfo> = s
        .decks
        .iter()
        .filter_map(|(id, d)| {
            let deck = (id != "plain").then(|| from_id::<Deck>(id));
            if deck == Some(None) {
                return None;
            }
            let deck = deck.flatten();
            Some(DeckStatsInfo {
                deck: deck.map(Deck::name),
                games: d.games,
                won: d.won,
                best_ante: d.best_ante,
                best_stake: profile.best_stake(deck).map(Stake::name),
            })
        })
        .collect();
    decks.sort_by_key(|d| std::cmp::Reverse(d.games));
    let mut hands: Vec<HandStatsInfo> = s
        .hands
        .iter()
        .filter_map(|(id, &played)| {
            let kind = from_id::<HandKind>(id)?;
            Some(HandStatsInfo {
                kind,
                hand_name: kind.name(),
                played,
            })
        })
        .collect();
    hands.sort_by_key(|h| std::cmp::Reverse(h.played));
    StatsInfo {
        games: s.games_won + s.games_lost + s.games_abandoned,
        won: s.games_won,
        lost: s.games_lost,
        abandoned: s.games_abandoned,
        streak: s.streak,
        best_streak: s.best_streak,
        best_ante: s.best_ante,
        best_hand: s.best_hand,
        blinds_won: s.blinds_won,
        hands_played: s.hands_played,
        jokers_sold: s.jokers_sold,
        decks,
        hands,
    }
}

fn history_info(profile: &Profile) -> Vec<RecordInfo> {
    profile
        .history()
        .iter()
        .map(|r| RecordInfo {
            ended: r.ended,
            seed: r.seed,
            seed_code: crate::rng::seed_code(r.seed),
            deck: from_id::<Deck>(&r.deck).map(Deck::name),
            stake: r.stake.name(),
            outcome: r.outcome,
            ante: r.ante,
            boss: r
                .lost_to
                .as_deref()
                .and_then(from_id::<Boss>)
                .map(Boss::name),
            best_hand: r.best_hand,
            hand_name: r
                .main_hand
                .as_deref()
                .and_then(from_id::<HandKind>)
                .map(HandKind::name),
            jokers: r
                .jokers
                .iter()
                .filter_map(|j| from_id::<Joker>(j))
                .map(Joker::name)
                .collect(),
        })
        .collect()
}

/// The collection as the profile has it.
pub fn collection(profile: &Profile) -> Collection {
    let items = Item::all();
    let families: Vec<FamilyInfo> = Family::ALL
        .into_iter()
        .map(|family| {
            let items: Vec<ItemInfo> = items
                .iter()
                .filter(|i| i.family() == family)
                .map(|&item| item_info(item, profile))
                .collect();
            FamilyInfo {
                id: family,
                name: family.name(),
                found: items.iter().filter(|i| i.state == "found").count(),
                total: items.len(),
                items,
            }
        })
        .collect();
    let achievements = Achievement::ALL
        .into_iter()
        .map(|a| {
            let done = profile.is_done(a);
            let (progress, goal) = profile.progress(a).unzip();
            AchievementInfo {
                id: a,
                name: a.name(),
                description: a.description(),
                quip: a.quip(),
                done,
                progress,
                goal,
                unlocks: a.unlocks().filter(|_| done).map(Item::name),
                unlocks_family: a.unlocks().map(|i| i.family().singular()),
            }
        })
        .collect();
    Collection {
        found: families.iter().map(|f| f.found).sum(),
        total: families.iter().map(|f| f.total).sum(),
        families,
        achievements,
        stats: stats_info(profile),
        history: history_info(profile),
    }
}

fn item_info(item: Item, profile: &Profile) -> ItemInfo {
    let found = profile.has_found(item);
    let locked = !found && !profile.unlocked(item);
    let joker = match item {
        Item::Joker(j) => Some(j),
        _ => None,
    };
    let hidden_by = Achievement::unlocking(item).filter(|_| locked);
    ItemInfo {
        id: item.id(),
        state: if found {
            "found"
        } else if locked {
            "locked"
        } else {
            "unknown"
        },
        secret: item.secret(),
        name: found.then(|| item.name()),
        description: found.then(|| item.description()).filter(|d| !d.is_empty()),
        quip: found.then(|| item.quip()).filter(|q| !q.is_empty()),
        rarity: joker.filter(|_| found).map(Joker::rarity),
        achievement: hidden_by.map(Achievement::name),
        condition: hidden_by.map(Achievement::description),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn ids_are_unique_and_read_back() {
        let items = Item::all();
        let ids: HashSet<String> = items.iter().map(|i| i.id()).collect();
        assert_eq!(ids.len(), items.len());
        assert_eq!(Item::Joker(Joker::Fork).id(), "joker:fork");
        assert_eq!(
            Item::Consumable(Consumable::Command(Command::RmRf)).id(),
            "command:rm_rf"
        );
        assert!(items.iter().all(|i| !i.id().ends_with(':')));
    }

    #[test]
    fn unknown_items_keep_their_text() {
        let mut profile = Profile::default();
        profile.find(Item::Joker(Joker::Fork));
        let c = collection(&profile);
        let jokers = &c.families[0];
        assert_eq!((jokers.found, jokers.total), (1, Joker::ALL.len()));
        let fork = jokers.items.iter().find(|i| i.id == "joker:fork").unwrap();
        assert_eq!((fork.state, fork.name.as_deref()), ("found", Some("Fork")));
        let other = jokers
            .items
            .iter()
            .find(|i| i.id == "joker:dotfiles")
            .unwrap();
        assert_eq!(
            (other.state, &other.name, &other.description),
            ("unknown", &None, &None)
        );
        let hidden = jokers
            .items
            .iter()
            .find(|i| i.id == "joker:hash_collision")
            .unwrap();
        assert_eq!((hidden.state, hidden.secret), ("locked", true));
        assert_eq!(hidden.achievement, Some("Duplicate Detected"));
        assert_eq!(c.total, Item::all().len());
    }
}
