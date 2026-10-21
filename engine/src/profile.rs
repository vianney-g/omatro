//! The player's profile, kept from one game to the next: what the collection
//! found, the achievements, and a few lifetime counts. The app writes it to
//! `~/.local/state/omatro/profile.json`; ids are strings, so an item renamed
//! or removed later never keeps a profile from loading.

use crate::achievement::{Achievement, BARE_METAL_JOKERS, Count};
use crate::boss::Boss;
use crate::collection::Item;
use crate::game::{Event, Game};
use crate::hand::HandKind;
use crate::joker::Joker;
use crate::run_deck::Deck;
use crate::stake::Stake;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// Bumped when profiles of older versions can no longer be read.
pub const PROFILE_VERSION: u32 = 1;

/// Games the history keeps, the latest ones (the stats count them all).
pub const HISTORY_LEN: usize = 50;

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
    /// Games left for a new one before their end.
    #[serde(default)]
    pub games_abandoned: u64,
    /// Games won in a row, up to now, and the most ever.
    #[serde(default)]
    pub streak: u64,
    #[serde(default)]
    pub best_streak: u64,
    /// The highest ante a game reached.
    #[serde(default)]
    pub best_ante: u32,
    /// How often each hand type was played, by id.
    #[serde(default)]
    pub hands: BTreeMap<String, u64>,
    /// Games per deck id (`plain` for none).
    #[serde(default)]
    pub decks: BTreeMap<String, DeckStats>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeckStats {
    /// Games ended: won, lost or abandoned.
    #[serde(default)]
    pub games: u64,
    #[serde(default)]
    pub won: u64,
    #[serde(default)]
    pub best_ante: u32,
}

/// How a game of the history ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    /// The winning ante's boss was beaten (endless mode may have gone on).
    Won,
    Lost,
    /// Left for a new game.
    Abandoned,
}

/// A game of the history. Items are kept by id, as in the collection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GameRecord {
    /// When it ended, in seconds since 1970.
    pub ended: u64,
    pub seed: u64,
    /// The deck id (`plain` for none).
    pub deck: String,
    pub stake: Stake,
    pub outcome: Outcome,
    pub ante: u32,
    /// The boss whose blind ended the game.
    #[serde(default)]
    pub lost_to: Option<String>,
    #[serde(default)]
    pub best_hand: f64,
    /// The hand type played most.
    #[serde(default)]
    pub main_hand: Option<String>,
    /// The jokers held at the end.
    #[serde(default)]
    pub jokers: Vec<String>,
}

fn id_of<T: Serialize>(item: T) -> Option<String> {
    serde_json::to_value(item)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
}

impl GameRecord {
    fn of(game: &Game, outcome: Outcome) -> GameRecord {
        GameRecord {
            ended: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |d| d.as_secs()),
            seed: game.seed(),
            deck: deck_id(game.run_deck()),
            stake: game.stake(),
            outcome,
            ante: game.ante(),
            lost_to: game.lost_to().and_then(id_of::<Boss>),
            best_hand: game.best_hand(),
            main_hand: game.main_hand().and_then(id_of::<HandKind>),
            jokers: game
                .joker_list()
                .into_iter()
                .filter_map(id_of::<Joker>)
                .collect(),
        }
    }
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
    /// The latest games, the last one first.
    #[serde(default)]
    history: VecDeque<GameRecord>,
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
                eprintln!(
                    "omatro: the profile {} can't be read, starting a new one",
                    path.display()
                );
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
            eprintln!(
                "omatro: could not save the profile to {}: {e}",
                path.display()
            );
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

    /// The latest games, the last one first.
    pub fn history(&self) -> &VecDeque<GameRecord> {
        &self.history
    }

    /// A game came to its end (or was left): into the history and the
    /// stats. A won game going on in endless mode keeps the record of its
    /// win, brought up to date when it ends.
    fn record(&mut self, game: &Game, outcome: Outcome) {
        let record = GameRecord::of(game, outcome);
        let deck = self.stats.decks.entry(record.deck.clone()).or_default();
        deck.best_ante = deck.best_ante.max(record.ante);
        self.stats.best_ante = self.stats.best_ante.max(record.ante);
        if outcome != Outcome::Won
            && game.won()
            && let Some(won) = self
                .history
                .iter_mut()
                .find(|r| r.seed == record.seed && r.outcome == Outcome::Won)
        {
            *won = GameRecord {
                outcome: Outcome::Won,
                ..record
            };
            return;
        }
        deck.games += 1;
        deck.won += u64::from(outcome == Outcome::Won);
        match outcome {
            Outcome::Won => {
                self.stats.streak += 1;
                self.stats.best_streak = self.stats.best_streak.max(self.stats.streak);
            }
            Outcome::Lost => {
                self.stats.games_lost += 1;
                self.stats.streak = 0;
            }
            Outcome::Abandoned => {
                self.stats.games_abandoned += 1;
                self.stats.streak = 0;
            }
        }
        self.history.push_front(record);
        self.history.truncate(HISTORY_LEN);
    }

    /// A new game replaces this one: abandoned, unless it was over or not
    /// started (no hand played). Games with cheats or a chosen seed count
    /// for nothing.
    pub fn abandon(&mut self, game: &Game) {
        if game.counts() && !game.is_over() && game.main_hand().is_some() {
            self.record(game, Outcome::Abandoned);
        }
    }

    /// The collection was opened.
    pub fn opened_collection(&mut self) -> Vec<News> {
        let mut news = Vec::new();
        self.achieve(Achievement::Rtfm, &mut news);
        news
    }

    /// Takes in what happened in the game since the last look. A game with
    /// cheats or a chosen seed fills the collection, but no achievement nor
    /// count.
    pub fn observe(&mut self, game: &mut Game) -> Vec<News> {
        let mut news = Vec::new();
        let fair = game.counts();
        for event in game.take_events() {
            match event {
                Event::Obtained(item) => self.found(item, &mut news),
                _ if !fair => {}
                Event::HandPlayed { kind, total } => {
                    self.stats.hands_played += 1;
                    if let Some(id) = id_of(kind) {
                        *self.stats.hands.entry(id).or_default() += 1;
                    }
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
                Event::BlindWon {
                    boss,
                    hands,
                    discards,
                } => {
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
                    self.record(game, Outcome::Won);
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
                Event::GameOver { won: false } => self.record(game, Outcome::Lost),
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
        text["found"]
            .as_array_mut()
            .unwrap()
            .push("joker:gone_since".into());
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
            news.iter()
                .map(|n| (n.kind, n.name.as_str()))
                .collect::<Vec<_>>(),
            [
                ("achievement", "Duplicate Detected"),
                ("unlocked", "Hash Collision")
            ]
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
        game.cheat(Cheat {
            win: true,
            ..Cheat::default()
        })
        .unwrap();
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
        assert!(
            news.iter()
                .any(|n| n.kind == "unlocked" && n.name == "Postmortem")
        );
    }

    #[test]
    fn winning_unlocks_the_next_level_of_the_deck() {
        let mut profile = Profile::default();
        assert_eq!(profile.top_stake(Some(Deck::ThinkPad)), Stake::DailyCommute);
        let mut game = Game::with_stake(1, vec![], Some(Deck::ThinkPad), Stake::TrackDay);
        game.events_for_tests(vec![Event::GameOver { won: true }]);
        let news = profile.observe(&mut game);
        assert_eq!(
            profile.best_stake(Some(Deck::ThinkPad)),
            Some(Stake::TrackDay)
        );
        assert_eq!(profile.top_stake(Some(Deck::ThinkPad)), Stake::ClubRacing);
        assert_eq!(profile.top_stake(Some(Deck::Basecamp)), Stake::DailyCommute);
        assert!(
            news.iter()
                .any(|n| n.kind == "unlocked" && n.name == "Club Racing")
        );
        // A lower level never lowers the best.
        let mut game = Game::with_stake(1, vec![], Some(Deck::ThinkPad), Stake::DailyCommute);
        game.events_for_tests(vec![Event::GameOver { won: true }]);
        assert!(
            profile
                .observe(&mut game)
                .iter()
                .all(|n| n.kind != "unlocked")
        );
        assert_eq!(
            profile.best_stake(Some(Deck::ThinkPad)),
            Some(Stake::TrackDay)
        );
        // The last level unlocks nothing more.
        profile.won_at(None, Stake::Hypercar);
        assert_eq!(profile.top_stake(None), Stake::Hypercar);
    }

    /// Plays the first card until the blind is lost.
    fn lose(game: &mut Game) {
        game.select_blind().unwrap();
        while game.state().phase == crate::game::Phase::Playing {
            game.play(&[0]).unwrap();
        }
    }

    #[test]
    fn games_go_to_the_history_and_the_stats() {
        let mut profile = Profile::default();
        let mut game = Game::with_deck(3, vec![], Some(Deck::ThinkPad));
        lose(&mut game);
        profile.observe(&mut game);
        let lost = &profile.history()[0];
        assert_eq!(
            (lost.outcome, lost.seed, lost.deck.as_str(), lost.ante),
            (Outcome::Lost, 3, "think_pad", 1)
        );
        assert_eq!(lost.main_hand.as_deref(), Some("high_card"));
        assert_eq!(lost.lost_to, None);
        assert!(lost.best_hand > 0.0);
        assert_eq!(profile.stats.hands.get("high_card"), Some(&4));
        assert_eq!(
            profile.stats.decks.get("think_pad"),
            Some(&DeckStats {
                games: 1,
                won: 0,
                best_ante: 1
            })
        );

        // A win, then the same game lost further on in endless mode: one
        // game, won.
        let mut game = Game::new(4, vec![]);
        game.won_for_tests();
        profile.observe(&mut game);
        assert_eq!((profile.stats.streak, profile.stats.best_streak), (1, 1));
        lose(&mut game);
        profile.observe(&mut game);
        assert_eq!(profile.history().len(), 2);
        assert_eq!(profile.history()[0].outcome, Outcome::Won);
        assert_eq!(profile.history()[0].main_hand.as_deref(), Some("high_card"));
        assert_eq!((profile.stats.games_won, profile.stats.games_lost), (1, 1));
        assert_eq!(profile.stats.streak, 1);
        assert_eq!(profile.stats.decks.get("plain").map(|d| d.won), Some(1));
    }

    #[test]
    fn games_left_for_another_are_abandoned() {
        let mut profile = Profile::default();
        // Before its first hand, a game leaves no trace.
        let mut game = Game::new(5, vec![]);
        profile.abandon(&game);
        assert!(profile.history().is_empty());
        game.select_blind().unwrap();
        game.play(&[0]).unwrap();
        profile.abandon(&game);
        assert_eq!(profile.history()[0].outcome, Outcome::Abandoned);
        assert_eq!(profile.stats.games_abandoned, 1);
        // Neither a lost game nor one with cheats.
        let mut lost = Game::new(6, vec![]);
        lose(&mut lost);
        profile.abandon(&lost);
        game.mark_cheated();
        profile.abandon(&game);
        assert_eq!(profile.history().len(), 1);
    }

    #[test]
    fn the_history_keeps_the_latest_games() {
        let mut profile = Profile::default();
        let mut game = Game::new(7, vec![]);
        game.select_blind().unwrap();
        game.play(&[0]).unwrap();
        for _ in 0..HISTORY_LEN + 5 {
            profile.abandon(&game);
        }
        assert_eq!(profile.history().len(), HISTORY_LEN);
        assert_eq!(profile.stats.games_abandoned, HISTORY_LEN as u64 + 5);
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
