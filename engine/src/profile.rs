//! The player's profile, kept from one game to the next: what the collection
//! found, the achievements, and a few lifetime counts. The app writes it to
//! `~/.local/state/omatro/profile.json`; ids are strings, so an item renamed
//! or removed later never keeps a profile from loading.

use crate::achievement::{Achievement, BARE_METAL_JOKERS, Count};
use crate::collection::Item;
use crate::game::{Event, Game};
use crate::hand::HandKind;
use crate::joker::Joker;
use crate::run_deck::Deck;
use crate::stake::Stake;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::OnceLock;
use std::path::{Path, PathBuf};

/// Bumped when profiles of older versions can no longer be read.
pub const PROFILE_VERSION: u32 = 1;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    #[serde(default)]
    pub games_won: u64,
    #[serde(default)]
    pub games_lost: u64,
    #[serde(default)]
    pub blinds_won: u64,
    #[serde(default)]
    pub hands_played: u64,
    #[serde(default)]
    pub jokers_sold: u64,
    #[serde(default)]
    pub best_hand: f64,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    #[serde(default)]
    version: u32,
    /// Ids of the items found (`joker:fork`).
    #[serde(default)]
    found: BTreeSet<String>,
    /// Ids of the achievements done.
    #[serde(default)]
    achievements: BTreeSet<String>,
    #[serde(default)]
    pub stats: Stats,
    /// The highest difficulty level won with each deck (`plain` for none).
    #[serde(default)]
    stakes: BTreeMap<String, Stake>,
}

/// Something new for the player: an item found, an achievement done, or what
/// it unlocked.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct News {
    /// `found`, `achievement` or `unlocked`.
    pub kind: &'static str,
    pub name: String,
    /// The item's family, or what the achievement took.
    pub text: String,
}

/// Where the app keeps the profile: next to the save.
pub fn default_profile_path() -> Option<PathBuf> {
    crate::protocol::default_save_path().map(|p| p.with_file_name("profile.json"))
}

fn deck_id(deck: Option<Deck>) -> String {
    deck.and_then(|d| serde_json::to_value(d).ok())
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_else(|| "plain".into())
}

fn achievement_id(a: Achievement) -> String {
    serde_json::to_value(a)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

impl Profile {
    /// Reads the profile; a new one if there is none. One it can't read is
    /// kept aside (`profile.json.bak`) rather than lost.
    pub fn load(path: &Path) -> Profile {
        let text = match std::fs::read_to_string(path) {
            Ok(text) => text,
            Err(_) => return Profile::default(),
        };
        match serde_json::from_str::<Profile>(&text) {
            Ok(profile) if profile.version <= PROFILE_VERSION => profile,
            _ => {
                eprintln!("omatro: the profile {} can't be read, starting a new one", path.display());
                let _ = std::fs::rename(path, path.with_extension("json.bak"));
                Profile::default()
            }
        }
    }

    /// Writes the profile (to a temporary file first, as the save does).
    pub fn save(&self, path: &Path) {
        let profile = Profile {
            version: PROFILE_VERSION,
            ..self.clone()
        };
        let text = serde_json::to_string(&profile).expect("profiles serialize");
        let tmp = path.with_extension("json.tmp");
        let written = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| std::fs::write(&tmp, text))
            .and_then(|_| std::fs::rename(&tmp, path));
        if let Err(e) = written {
            eprintln!("omatro: could not save the profile to {}: {e}", path.display());
        }
    }

    pub fn has_found(&self, item: Item) -> bool {
        self.found.contains(&item.id())
    }

    /// Items found, among those that still exist.
    pub fn found_count(&self) -> usize {
        static IDS: OnceLock<HashSet<String>> = OnceLock::new();
        let ids = IDS.get_or_init(|| Item::all().into_iter().map(Item::id).collect());
        self.found.iter().filter(|id| ids.contains(*id)).count()
    }

    pub fn is_done(&self, a: Achievement) -> bool {
        self.achievements.contains(&achievement_id(a))
    }

    /// Hidden items wait for their achievement; the others are always there.
    pub fn unlocked(&self, item: Item) -> bool {
        Achievement::unlocking(item).is_none_or(|a| self.is_done(a))
    }

    /// Hidden jokers games must not offer yet.
    pub fn locked_jokers(&self) -> Vec<Joker> {
        Joker::ALL
            .into_iter()
            .filter(|&j| !self.unlocked(Item::Joker(j)))
            .collect()
    }

    pub fn deck_unlocked(&self, deck: Deck) -> bool {
        self.unlocked(Item::Deck(deck))
    }

    /// The highest difficulty level won with a deck.
    pub fn best_stake(&self, deck: Option<Deck>) -> Option<Stake> {
        self.stakes.get(&deck_id(deck)).copied()
    }

    /// The highest difficulty level a deck may be played at: the one after
    /// the highest won.
    pub fn top_stake(&self, deck: Option<Deck>) -> Stake {
        self.best_stake(deck)
            .map_or(Stake::DailyCommute, |best| best.next().unwrap_or(best))
    }

    /// A game was won with a deck at a level; true when it is the deck's
    /// best so far.
    fn won_at(&mut self, deck: Option<Deck>, stake: Stake) -> bool {
        let best = self.stakes.entry(deck_id(deck)).or_insert(stake);
        let better = *best <= stake;
        *best = (*best).max(stake);
        better
    }

    /// Where a counting achievement stands, and its goal.
    pub fn progress(&self, a: Achievement) -> Option<(u64, u64)> {
        let (count, goal) = a.goal()?;
        let now = match count {
            Count::GamesLost => self.stats.games_lost,
            Count::JokersSold => self.stats.jokers_sold,
            Count::Found => self.found_count() as u64,
        };
        let goal = goal.unwrap_or(Item::all().len() as u64);
        Some((now.min(goal), goal))
    }

    /// Adds an item to the collection; true when it is new.
    pub fn find(&mut self, item: Item) -> bool {
        self.found.insert(item.id())
    }

    fn found(&mut self, item: Item, news: &mut Vec<News>) {
        if self.find(item) {
            news.push(News {
                kind: "found",
                name: item.name(),
                text: item.family().singular().into(),
            });
        }
    }

    fn achieve(&mut self, a: Achievement, news: &mut Vec<News>) {
        if !self.achievements.insert(achievement_id(a)) {
            return;
        }
        news.push(News {
            kind: "achievement",
            name: a.name().into(),
            text: a.description().into(),
        });
        if let Some(item) = a.unlocks() {
            news.push(News {
                kind: "unlocked",
                name: item.name(),
                text: item.family().singular().into(),
            });
        }
    }

    /// The collection was opened.
    pub fn opened_collection(&mut self) -> Vec<News> {
        let mut news = Vec::new();
        self.achieve(Achievement::Rtfm, &mut news);
        news
    }

    /// Takes in what happened in the game since the last look. A game with
    /// cheats fills the collection, but no achievement nor count.
    pub fn observe(&mut self, game: &mut Game) -> Vec<News> {
        let mut news = Vec::new();
        let fair = !game.cheated();
        for event in game.take_events() {
            match event {
                Event::Obtained(item) => self.found(item, &mut news),
                _ if !fair => {}
                Event::HandPlayed { kind, total } => {
                    self.stats.hands_played += 1;
                    self.stats.best_hand = self.stats.best_hand.max(total);
                    if kind == HandKind::ForkBomb {
                        self.achieve(Achievement::DuplicateDetected, &mut news);
                    }
                    if total >= 1e6 {
                        self.achieve(Achievement::BigO, &mut news);
                    }
                    if total >= 1e8 {
                        self.achieve(Achievement::HeatDeath, &mut news);
                    }
                }
                Event::BlindWon { boss, hands, discards } => {
                    self.stats.blinds_won += 1;
                    self.achieve(Achievement::HelloWorld, &mut news);
                    if boss && hands == 1 && discards == 0 {
                        self.achieve(Achievement::OneLiner, &mut news);
                    }
                }
                Event::JokerSold => self.stats.jokers_sold += 1,
                Event::Rerolled { in_shop } if in_shop >= 10 => {
                    self.achieve(Achievement::SunkCost, &mut news)
                }
                Event::Rerolled { .. } => {}
                Event::GameOver { won: true } => {
                    self.stats.games_won += 1;
                    let (deck, stake) = (game.run_deck(), game.stake());
                    let before = self.top_stake(deck);
                    self.won_at(deck, stake);
                    let after = self.top_stake(deck);
                    if after > before {
                        news.push(News {
                            kind: "unlocked",
                            name: after.name().into(),
                            text: format!(
                                "level for {}",
                                deck.map_or("the plain deck", Deck::name)
                            ),
                        });
                    }
                    self.achieve(Achievement::ItsNotAPhase, &mut news);
                    if game.most_jokers() <= BARE_METAL_JOKERS {
                        self.achieve(Achievement::BareMetal, &mut news);
                    }
                }
                Event::GameOver { won: false } => self.stats.games_lost += 1,
            }
        }
        for item in game.owned_items() {
            self.found(item, &mut news);
        }
        if fair {
            if game.money() >= 200 {
                self.achieve(Achievement::SavingsAccount, &mut news);
            }
            if game.ante() >= 12 {
                self.achieve(Achievement::LongTermSupport, &mut news);
            }
        }
        for a in Achievement::ALL {
            let counted = fair || a.goal().is_some_and(|(count, _)| count == Count::Found);
            if counted && self.progress(a).is_some_and(|(now, goal)| now >= goal) {
                self.achieve(a, &mut news);
            }
        }
        news
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Cheat;

    #[test]
    fn reads_back_and_ignores_unknown_ids() {
        let mut profile = Profile::default();
        profile.find(Item::Joker(Joker::Fork));
        let mut text: serde_json::Value = serde_json::to_value(&profile).unwrap();
        text["found"].as_array_mut().unwrap().push("joker:gone_since".into());
        text["achievements"] = serde_json::json!(["hello_world", "retired"]);
        let back: Profile = serde_json::from_value(text).unwrap();
        assert!(back.has_found(Item::Joker(Joker::Fork)));
        assert_eq!(back.found_count(), 1);
        assert!(back.is_done(Achievement::HelloWorld));
        assert!(!back.is_done(Achievement::Rtfm));
    }

    #[test]
    fn hidden_items_wait_for_their_achievement() {
        let mut profile = Profile::default();
        assert!(profile.locked_jokers().contains(&Joker::HashCollision));
        assert!(!profile.deck_unlocked(Deck::Raid1));
        assert!(profile.deck_unlocked(Deck::ThinkPad));
        let mut news = Vec::new();
        profile.achieve(Achievement::DuplicateDetected, &mut news);
        assert!(!profile.locked_jokers().contains(&Joker::HashCollision));
        assert_eq!(
            news.iter().map(|n| (n.kind, n.name.as_str())).collect::<Vec<_>>(),
            [("achievement", "Duplicate Detected"), ("unlocked", "Hash Collision")]
        );
        // Done once only.
        profile.achieve(Achievement::DuplicateDetected, &mut news);
        assert_eq!(news.len(), 2);
    }

    #[test]
    fn games_fill_the_collection_and_count() {
        let mut profile = Profile::default();
        let mut game = Game::with_deck(1, vec![Joker::Fork], Some(Deck::ThinkPad));
        let news = profile.observe(&mut game);
        assert!(profile.has_found(Item::Joker(Joker::Fork)));
        assert!(profile.has_found(Item::Deck(Deck::ThinkPad)));
        assert!(news.iter().any(|n| n.kind == "found" && n.name == "Fork"));
        assert!(profile.observe(&mut game).is_empty());

        game.select_blind().unwrap();
        game.cheat(Cheat { win: true, ..Cheat::default() }).unwrap();
        profile.observe(&mut game);
        // A blind won with a cheat is no achievement.
        assert!(!profile.is_done(Achievement::HelloWorld));
        assert_eq!(profile.stats.blinds_won, 0);
    }

    #[test]
    fn a_fair_win_and_a_loss() {
        let mut profile = Profile::default();
        let mut game = Game::new(3, vec![]);
        game.select_blind().unwrap();
        while game.state().phase == crate::game::Phase::Playing {
            game.play(&[0]).unwrap();
        }
        profile.observe(&mut game);
        assert_eq!(profile.stats.hands_played, 4);
        assert_eq!(profile.stats.games_lost, 1);
        assert!(profile.has_found(Item::Hand(HandKind::HighCard)));
        assert_eq!(profile.progress(Achievement::CoreDump), Some((1, 5)));
        profile.stats.games_lost = 4;
        game.events_for_tests(vec![Event::GameOver { won: false }]);
        let news = profile.observe(&mut game);
        assert!(profile.is_done(Achievement::CoreDump));
        assert!(news.iter().any(|n| n.kind == "unlocked" && n.name == "Postmortem"));
    }

    #[test]
    fn winning_unlocks_the_next_level_of_the_deck() {
        let mut profile = Profile::default();
        assert_eq!(profile.top_stake(Some(Deck::ThinkPad)), Stake::DailyCommute);
        let mut game = Game::with_stake(1, vec![], Some(Deck::ThinkPad), Stake::TrackDay);
        game.events_for_tests(vec![Event::GameOver { won: true }]);
        let news = profile.observe(&mut game);
        assert_eq!(profile.best_stake(Some(Deck::ThinkPad)), Some(Stake::TrackDay));
        assert_eq!(profile.top_stake(Some(Deck::ThinkPad)), Stake::ClubRacing);
        assert_eq!(profile.top_stake(Some(Deck::Basecamp)), Stake::DailyCommute);
        assert!(news.iter().any(|n| n.kind == "unlocked" && n.name == "Club Racing"));
        // A lower level never lowers the best.
        let mut game = Game::with_stake(1, vec![], Some(Deck::ThinkPad), Stake::DailyCommute);
        game.events_for_tests(vec![Event::GameOver { won: true }]);
        assert!(profile.observe(&mut game).iter().all(|n| n.kind != "unlocked"));
        assert_eq!(profile.best_stake(Some(Deck::ThinkPad)), Some(Stake::TrackDay));
        // The last level unlocks nothing more.
        profile.won_at(None, Stake::Hypercar);
        assert_eq!(profile.top_stake(None), Stake::Hypercar);
    }

    #[test]
    fn saved_and_loaded() {
        let dir = std::env::temp_dir().join(format!("omatro-profile-{}", std::process::id()));
        let path = dir.join("profile.json");
        let _ = std::fs::remove_dir_all(&dir);
        assert_eq!(Profile::load(&path), Profile::default());
        let mut profile = Profile::default();
        profile.find(Item::Boss(crate::boss::Boss::Clippy));
        profile.save(&path);
        assert!(Profile::load(&path).has_found(Item::Boss(crate::boss::Boss::Clippy)));
        std::fs::write(&path, "not a profile").unwrap();
        assert_eq!(Profile::load(&path), Profile::default());
        assert!(path.with_extension("json.bak").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
