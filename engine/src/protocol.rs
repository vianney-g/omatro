//! Protocol between the engine and the display: one JSON command per line
//! in, one JSON response per line out.

use crate::achievement::Achievement;
use crate::collection::{Collection, Item, collection};
use crate::game::{Cheat, DeckInfo, Game, SortBy, State};
use crate::glossary::{Term, glossary};
use crate::hand::HandKind;
use crate::i18n::{Catalog, Lang, translate_response};
use crate::joker::{Bonus, Joker};
use crate::profile::{News, Profile};
use crate::run_deck::Deck;
use crate::score::Breakdown;
use crate::stake::{Stake, StakeInfo};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Deserialize, PartialEq)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum Command {
    NewGame {
        seed: Option<u64>,
        #[serde(default)]
        jokers: Vec<Joker>,
        /// The starting deck (none: the plain one).
        deck: Option<Deck>,
        /// The difficulty level (none: the first).
        #[serde(default)]
        stake: Option<Stake>,
    },
    /// Lists the starting decks, for the new game screen.
    Decks,
    /// Lists the terms card texts mention, with what they mean.
    Glossary,
    /// The collection and the achievements, as the profile has them.
    Collection,
    /// Tells whether a saved game can be resumed, and its state.
    SavedGame,
    /// Resumes the saved game.
    Resume,
    /// Sets the language of the responses, and returns the catalog for the
    /// interface's own text.
    Language {
        lang: Lang,
    },
    State,
    SelectBlind,
    SkipBlind,
    Play {
        cards: Vec<usize>,
    },
    Discard {
        cards: Vec<usize>,
    },
    Preview {
        cards: Vec<usize>,
    },
    CashOut,
    Buy {
        offer: usize,
    },
    BuyAndUse {
        offer: usize,
    },
    Sell {
        joker: usize,
    },
    UseConsumable {
        index: usize,
        /// Cards of the hand a command changes.
        #[serde(default)]
        cards: Vec<usize>,
    },
    SellConsumable {
        index: usize,
    },
    BuyPack {
        pack: usize,
    },
    Pick {
        choice: usize,
        /// Cards of the drawn hand a command changes.
        #[serde(default)]
        cards: Vec<usize>,
    },
    SkipPack,
    BuyVoucher {
        #[serde(default)]
        index: usize,
    },
    RerollBoss,
    MoveJoker {
        from: usize,
        to: usize,
    },
    /// Moves a card of the hand.
    MoveCard {
        from: usize,
        to: usize,
    },
    /// Sorts the hand, and new hands from now on.
    SortHand {
        by: SortBy,
    },
    Reroll,
    NextBlind,
    /// Only accepted when the session allows debugging.
    Debug(Cheat),
}

#[derive(Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Response {
    State {
        state: State,
    },
    Scored {
        score: Breakdown,
        state: State,
    },
    /// Hand made by the selection (`hand` is null when nothing is chosen).
    Preview {
        cards: Vec<usize>,
        hand: Option<HandKind>,
        hand_name: Option<&'static str>,
        hand_level: Option<u32>,
        scoring: Vec<usize>,
        base_chips: u64,
        base_mult: f64,
        /// Each joker's badge if this hand were played (empty without a hand).
        bonuses: Vec<Option<Bonus>>,
    },
    Error {
        message: String,
    },
    Decks {
        decks: Vec<DeckInfo>,
        /// The difficulty levels, first to last.
        stakes: Vec<StakeInfo>,
    },
    Glossary {
        terms: Vec<Term>,
    },
    Collection {
        collection: Collection,
    },
    /// The language set, its catalog (English text → translation), and the
    /// languages to choose from.
    Language {
        lang: Lang,
        table: std::collections::HashMap<String, String>,
        languages: Vec<LanguageInfo>,
    },
    /// The saved game's state (none without one), or why it can't be resumed.
    SavedGame {
        state: Option<State>,
        problem: Option<String>,
    },
}

#[derive(Default)]
pub struct Session {
    game: Option<Game>,
    debug: bool,
    /// Jokers of every new game that names none (debug only).
    start_jokers: Vec<Joker>,
    /// Where the game in progress is saved after each command (none: never).
    save: Option<PathBuf>,
    /// The language of the responses (`handle_json` translates them).
    lang: Lang,
    /// The player's collection and achievements (none: everything is
    /// unlocked, nothing is kept), and the file it is written to.
    profile: Option<Profile>,
    profile_path: Option<PathBuf>,
    /// What the profile gained since the last response.
    news: Vec<News>,
}

#[derive(Debug, Serialize)]
pub struct LanguageInfo {
    pub id: Lang,
    pub name: &'static str,
}

/// Bumped when saves of older versions can no longer be read.
pub const SAVE_VERSION: u32 = 1;

/// A save file: the whole game, and the format it was written in.
#[derive(Serialize, Deserialize)]
struct Save {
    version: u32,
    game: Game,
}

/// Where the app keeps the game in progress: `$XDG_STATE_HOME/omatro/save.json`,
/// else `~/.local/state/omatro/save.json`.
pub fn default_save_path() -> Option<PathBuf> {
    let state = std::env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/state")))?;
    Some(state.join("omatro").join("save.json"))
}

/// Reads a save; the message says why it can't be resumed.
fn load(path: &Path) -> Result<Option<Game>, String> {
    let text = match std::fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("the saved game can't be read: {e}")),
    };
    let unreadable = || "the saved game comes from another version of omatro".to_string();
    let version: serde_json::Value = serde_json::from_str(&text).map_err(|_| unreadable())?;
    if version["version"] != SAVE_VERSION {
        return Err(unreadable());
    }
    let save: Save = serde_json::from_str(&text).map_err(|_| unreadable())?;
    Ok(Some(save.game))
}

impl Session {
    /// A session that accepts `debug` commands when `OMATRO_DEBUG=1`; then
    /// `OMATRO_JOKERS=uptime,fork` also sets the jokers new games start with.
    pub fn from_env() -> Self {
        let debug = std::env::var("OMATRO_DEBUG").is_ok_and(|v| v == "1");
        let start_jokers = match std::env::var("OMATRO_JOKERS") {
            Ok(list) if debug => list
                .split(',')
                .filter_map(|id| serde_json::from_value(id.trim().into()).ok())
                .collect(),
            _ => Vec::new(),
        };
        Session {
            game: None,
            debug,
            start_jokers,
            save: None,
            lang: Lang::En,
            profile: None,
            profile_path: None,
            news: Vec::new(),
        }
    }

    /// Saves the game in progress to this file after each command that
    /// changes it, and removes the file once the game is lost.
    pub fn with_save(mut self, path: PathBuf) -> Self {
        self.save = Some(path);
        self
    }

    /// Keeps a profile, read from this file and written back whenever it
    /// gains something.
    pub fn with_profile(mut self, path: PathBuf) -> Self {
        self.profile = Some(Profile::load(&path));
        self.profile_path = Some(path);
        self
    }

    /// Keeps a profile in memory only (tests, the bot).
    pub fn with_new_profile(mut self) -> Self {
        self.profile = Some(Profile::default());
        self
    }

    pub fn profile(&self) -> Option<&Profile> {
        self.profile.as_ref()
    }

    /// The game in progress (the bot scores its candidate hands on a copy).
    pub fn game(&self) -> Option<&Game> {
        self.game.as_ref()
    }

    /// What the profile gained since the last call: items found,
    /// achievements and what they unlocked.
    pub fn take_news(&mut self) -> Vec<News> {
        std::mem::take(&mut self.news)
    }

    /// Takes in what happened in the game, and writes the profile if it
    /// gained anything.
    fn observe(&mut self) {
        let (Some(profile), Some(game)) = (&mut self.profile, &mut self.game) else {
            return;
        };
        let before = profile.clone();
        let news = profile.observe(game);
        self.news.extend(news);
        if *profile != before
            && let Some(path) = &self.profile_path
        {
            profile.save(path);
        }
    }

    pub fn handle(&mut self, line: &str) -> Response {
        match serde_json::from_str::<Command>(line) {
            Ok(command) => {
                let changes = !matches!(
                    command,
                    Command::State
                        | Command::Preview { .. }
                        | Command::Decks
                        | Command::Glossary
                        | Command::Collection
                        | Command::SavedGame
                        | Command::Language { .. }
                );
                let response = self
                    .run(command)
                    .unwrap_or_else(|message| Response::Error { message });
                if changes && !matches!(response, Response::Error { .. }) {
                    self.persist();
                    self.observe();
                }
                response
            }
            Err(e) => Response::Error {
                message: format!("unreadable command: {e}"),
            },
        }
    }

    /// Handles a command and returns the response as JSON, translated into
    /// the session's language: what the app and the binary use.
    pub fn handle_json(&mut self, line: &str) -> String {
        let response = self.handle(line);
        let mut value = serde_json::to_value(response).expect("responses serialize");
        let news = self.take_news();
        if !news.is_empty() {
            value["news"] = serde_json::to_value(news).expect("news serialize");
        }
        translate_response(&mut value, self.lang);
        value.to_string()
    }

    /// Writes the game to the save file (to a temporary file first, so that a
    /// crash never leaves half a save), or removes it once the game is lost.
    fn persist(&self) {
        let (Some(path), Some(game)) = (&self.save, &self.game) else {
            return;
        };
        if game.is_over() {
            let _ = std::fs::remove_file(path);
            return;
        }
        let save = serde_json::json!({ "version": SAVE_VERSION, "game": game });
        let tmp = path.with_extension("json.tmp");
        let written = path
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| std::fs::write(&tmp, save.to_string()))
            .and_then(|_| std::fs::rename(&tmp, path));
        if let Err(e) = written {
            eprintln!("omatro: could not save the game to {}: {e}", path.display());
        }
    }

    fn run(&mut self, command: Command) -> Result<Response, String> {
        if let Command::NewGame {
            seed,
            mut jokers,
            deck,
            stake,
        } = command
        {
            let stake = stake.unwrap_or_default();
            if let Some(profile) = &self.profile
                && stake > profile.top_stake(deck)
            {
                return Err(format!(
                    "{} is locked: win at the level below with this deck first",
                    stake.name()
                ));
            }
            if let (Some(profile), Some(deck)) = (&self.profile, deck)
                && !profile.deck_unlocked(deck)
            {
                let a = Achievement::unlocking(Item::Deck(deck)).map_or("", Achievement::name);
                return Err(format!("{} is locked: {a} unlocks it", deck.name()));
            }
            if jokers.is_empty() {
                jokers = self.start_jokers.clone();
            }
            // The game it replaces, in the session or still in the save, goes
            // to the history.
            let previous = self
                .game
                .take()
                .or_else(|| self.save.as_deref().and_then(|p| load(p).ok().flatten()));
            if let (Some(profile), Some(previous)) = (&mut self.profile, &previous) {
                let before = profile.clone();
                profile.abandon(previous);
                if *profile != before
                    && let Some(path) = &self.profile_path
                {
                    profile.save(path);
                }
            }
            let helped = !jokers.is_empty();
            let seed = seed.unwrap_or_else(random_seed);
            let mut game = Game::with_stake(seed, jokers, deck, stake);
            if helped {
                game.mark_cheated();
            }
            if let Some(profile) = &self.profile {
                game.lock(profile.locked_jokers());
            }
            let game = self.game.insert(game);
            return Ok(Response::State {
                state: game.state(),
            });
        }
        if let Command::Decks = command {
            let decks = Deck::ALL
                .iter()
                .map(|&d| {
                    let mut info = DeckInfo::from(d);
                    if let Some(profile) = &self.profile {
                        if !profile.deck_unlocked(d) {
                            let a = Achievement::unlocking(Item::Deck(d));
                            info.achievement = a.map(Achievement::name);
                            info.condition = a.map(Achievement::description);
                        }
                        info.best_stake = profile.best_stake(Some(d));
                        info.top_stake = profile.top_stake(Some(d));
                    }
                    info
                })
                .collect();
            return Ok(Response::Decks {
                decks,
                stakes: Stake::ALL.iter().map(|&s| s.into()).collect(),
            });
        }
        if let Command::Glossary = command {
            return Ok(Response::Glossary { terms: glossary() });
        }
        if let Command::Collection = command {
            let profile = self.profile.as_mut().ok_or("no profile in this session")?;
            let news = profile.opened_collection();
            if !news.is_empty()
                && let Some(path) = &self.profile_path
            {
                profile.save(path);
            }
            self.news.extend(news);
            return Ok(Response::Collection {
                collection: collection(profile),
            });
        }
        if let Command::Language { lang } = command {
            self.lang = lang;
            return Ok(Response::Language {
                lang,
                table: Catalog::table(lang),
                languages: Lang::ALL
                    .iter()
                    .map(|&id| LanguageInfo {
                        id,
                        name: id.name(),
                    })
                    .collect(),
            });
        }
        if let Command::SavedGame = command {
            let Some(path) = &self.save else {
                return Ok(Response::SavedGame {
                    state: None,
                    problem: None,
                });
            };
            return Ok(match load(path) {
                Ok(game) => Response::SavedGame {
                    state: game.map(|g| g.state()),
                    problem: None,
                },
                Err(problem) => Response::SavedGame {
                    state: None,
                    problem: Some(problem),
                },
            });
        }
        if let Command::Resume = command {
            let path = self.save.as_ref().ok_or("no saved game")?;
            let game = load(path)?.ok_or("no saved game")?;
            let game = self.game.insert(game);
            return Ok(Response::State {
                state: game.state(),
            });
        }
        if matches!(command, Command::Debug(_)) && !self.debug {
            return Err("debug commands are off (set OMATRO_DEBUG=1)".into());
        }
        let game = self.game.as_mut().ok_or("no game in progress")?;
        match command {
            Command::NewGame { .. }
            | Command::Decks
            | Command::Glossary
            | Command::Collection
            | Command::SavedGame
            | Command::Resume
            | Command::Language { .. } => unreachable!(),
            Command::State => {}
            Command::SelectBlind => game.select_blind().map_err(|e| e.to_string())?,
            Command::SkipBlind => game.skip_blind().map_err(|e| e.to_string())?,
            Command::Play { cards } => {
                let score = game.play(&cards).map_err(|e| e.to_string())?;
                return Ok(Response::Scored {
                    score,
                    state: game.state(),
                });
            }
            Command::Preview { cards } => {
                let evaluated = game.preview(&cards).map_err(|e| e.to_string())?;
                let (base_chips, base_mult) = evaluated
                    .as_ref()
                    .map_or((0, 0.0), |e| game.levels().values(e.kind));
                return Ok(Response::Preview {
                    hand: evaluated.as_ref().map(|e| e.kind),
                    hand_name: evaluated.as_ref().map(|e| e.kind.name()),
                    hand_level: evaluated.as_ref().map(|e| game.levels().level(e.kind)),
                    bonuses: evaluated
                        .as_ref()
                        .map_or_else(Vec::new, |e| game.bonuses_for(e.kind)),
                    scoring: evaluated.map_or_else(Vec::new, |e| {
                        e.scoring.iter().map(|&i| sorted_nth(&cards, i)).collect()
                    }),
                    cards,
                    base_chips,
                    base_mult,
                });
            }
            Command::Discard { cards } => game.discard(&cards).map_err(|e| e.to_string())?,
            Command::CashOut => game.cash_out().map_err(|e| e.to_string())?,
            Command::Buy { offer } => game.buy(offer).map_err(|e| e.to_string())?,
            Command::BuyAndUse { offer } => game.buy_and_use(offer).map_err(|e| e.to_string())?,
            Command::Sell { joker } => game.sell(joker).map_err(|e| e.to_string())?,
            Command::UseConsumable { index, cards } => game
                .use_consumable(index, &cards)
                .map_err(|e| e.to_string())?,
            Command::SellConsumable { index } => {
                game.sell_consumable(index).map_err(|e| e.to_string())?
            }
            Command::BuyPack { pack } => game.buy_pack(pack).map_err(|e| e.to_string())?,
            Command::Pick { choice, cards } => {
                game.pick(choice, &cards).map_err(|e| e.to_string())?
            }
            Command::SkipPack => game.skip_pack().map_err(|e| e.to_string())?,
            Command::BuyVoucher { index } => game.buy_voucher(index).map_err(|e| e.to_string())?,
            Command::RerollBoss => game.reroll_boss().map_err(|e| e.to_string())?,
            Command::MoveJoker { from, to } => {
                game.move_joker(from, to).map_err(|e| e.to_string())?
            }
            Command::MoveCard { from, to } => {
                game.move_card(from, to).map_err(|e| e.to_string())?
            }
            Command::SortHand { by } => game.sort_hand_by(by),
            Command::Reroll => game.reroll().map_err(|e| e.to_string())?,
            Command::NextBlind => game.next_blind().map_err(|e| e.to_string())?,
            Command::Debug(cheat) => game.cheat(cheat).map_err(|e| e.to_string())?,
        }
        Ok(Response::State {
            state: game.state(),
        })
    }
}

/// Hand index of the `n`-th chosen card, cards being taken in hand order.
fn sorted_nth(cards: &[usize], n: usize) -> usize {
    let mut sorted = cards.to_vec();
    sorted.sort();
    sorted[n]
}

fn random_seed() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn send(session: &mut Session, line: &str) -> Value {
        serde_json::to_value(session.handle(line)).unwrap()
    }

    #[test]
    fn parses_commands() {
        assert_eq!(
            serde_json::from_str::<Command>(r#"{"cmd":"new_game","seed":3,"jokers":["dotfiles"]}"#)
                .unwrap(),
            Command::NewGame {
                seed: Some(3),
                jokers: vec![Joker::Dotfiles],
                deck: None,
                stake: None,
            }
        );
        assert_eq!(
            serde_json::from_str::<Command>(r#"{"cmd":"play","cards":[0,2]}"#).unwrap(),
            Command::Play { cards: vec![0, 2] }
        );
    }

    #[test]
    fn preview_scoring_points_into_the_hand() {
        let mut session = Session::default();
        session.handle(r#"{"cmd":"new_game","seed":1}"#);
        session.handle(r#"{"cmd":"select_blind"}"#);
        let game = session.game.as_mut().unwrap();
        game.set_hand(crate::card::cards("KH 9D 9S 7C 5H 4D 3S 2C"));
        let r = send(&mut session, r#"{"cmd":"preview","cards":[2,0,1]}"#);
        assert_eq!(
            (&r["hand"], &r["hand_name"]),
            (&"pair".into(), &"Pair".into())
        );
        assert_eq!(r["scoring"], serde_json::json!([1, 2]));
        assert_eq!(
            (r["base_chips"].as_u64(), r["base_mult"].as_f64()),
            (Some(10), Some(2.0))
        );
    }

    #[test]
    fn decks_are_listed_and_chosen() {
        let mut session = Session::default();
        let r = send(&mut session, r#"{"cmd":"decks"}"#);
        assert_eq!(r["decks"].as_array().unwrap().len(), Deck::ALL.len());
        let r = send(
            &mut session,
            r#"{"cmd":"new_game","seed":1,"deck":"basecamp"}"#,
        );
        assert_eq!(
            (&r["state"]["money"], &r["state"]["run_deck"]["name"]),
            (&14.into(), &"Basecamp".into())
        );
    }

    #[test]
    fn errors_before_any_game() {
        let mut session = Session::default();
        assert_eq!(send(&mut session, r#"{"cmd":"state"}"#)["event"], "error");
        assert_eq!(send(&mut session, "not json")["event"], "error");
    }

    #[test]
    fn full_exchange() {
        let mut session = Session::default();
        let r = send(
            &mut session,
            r#"{"cmd":"new_game","seed":42,"jokers":["rails_world"]}"#,
        );
        assert_eq!(r["event"], "state");
        assert_eq!(r["state"]["seed"], 42);
        assert_eq!(r["state"]["phase"], "blind_select");
        assert_eq!(r["state"]["blinds"][0]["status"], "select");
        assert_eq!(r["state"]["target"], 300.0);
        assert_eq!(r["state"]["jokers"][0]["name"], "Rails World");
        assert_eq!(r["state"]["money"], 4);
        assert_eq!(r["state"]["shop"], serde_json::json!([]));

        let r = send(&mut session, r#"{"cmd":"skip_blind"}"#);
        assert_eq!(r["state"]["blind"], "big");
        let r = send(&mut session, r#"{"cmd":"select_blind"}"#);
        assert_eq!(r["state"]["phase"], "playing");
        assert_eq!(r["state"]["hand"].as_array().unwrap().len(), 8);

        let r = send(&mut session, r#"{"cmd":"play","cards":[0,1]}"#);
        assert_eq!(r["event"], "scored");
        assert_eq!(r["state"]["hands_left"], 3);
        assert_eq!(r["score"]["total"], r["state"]["score"]);

        let r = send(&mut session, r#"{"cmd":"discard","cards":[0]}"#);
        assert_eq!(r["state"]["discards_left"], 2);

        let r = send(&mut session, r#"{"cmd":"preview","cards":[]}"#);
        assert_eq!(
            (r["event"].as_str(), r["hand"].is_null()),
            (Some("preview"), true)
        );

        for cmd in ["next_blind", "cash_out", "reroll"] {
            let r = send(&mut session, &format!(r#"{{"cmd":"{cmd}"}}"#));
            assert_eq!(r["event"], "error", "{cmd}");
        }
        let r = send(&mut session, r#"{"cmd":"buy","offer":0}"#);
        assert_eq!(r["message"], "the shop is closed");
    }

    #[test]
    fn debug_needs_the_debug_session() {
        let cheat = r#"{"cmd":"debug","money":50,"ante":8,"blind":"boss"}"#;
        let mut session = Session::default();
        send(&mut session, r#"{"cmd":"new_game","seed":1}"#);
        assert_eq!(send(&mut session, cheat)["event"], "error");

        let mut session = Session {
            debug: true,
            ..Session::default()
        };
        send(&mut session, r#"{"cmd":"new_game","seed":1}"#);
        let r = send(&mut session, cheat);
        assert_eq!(
            (
                &r["state"]["money"],
                &r["state"]["ante"],
                &r["state"]["blind"]
            ),
            (
                &serde_json::json!(50),
                &serde_json::json!(8),
                &serde_json::json!("boss")
            )
        );
        send(&mut session, r#"{"cmd":"select_blind"}"#);
        let r = send(&mut session, r#"{"cmd":"debug","win":true}"#);
        assert_eq!(
            (&r["state"]["phase"], &r["state"]["victory"]),
            (&serde_json::json!("won"), &serde_json::json!(true))
        );
        let r = send(&mut session, r#"{"cmd":"debug","mony":1}"#);
        assert_eq!(r["event"], "error");
    }

    #[test]
    fn games_are_saved_and_resumed() {
        let dir = std::env::temp_dir().join(format!("omatro-save-{}", std::process::id()));
        let path = dir.join("save.json");
        let _ = std::fs::remove_dir_all(&dir);
        let mut first = Session::default().with_save(path.clone());
        let none = send(&mut first, r#"{"cmd":"saved_game"}"#);
        assert_eq!(
            (&none["state"], &none["problem"]),
            (&Value::Null, &Value::Null)
        );
        send(
            &mut first,
            r#"{"cmd":"new_game","seed":7,"deck":"think_pad"}"#,
        );
        send(&mut first, r#"{"cmd":"select_blind"}"#);
        send(&mut first, r#"{"cmd":"play","cards":[0,1]}"#);
        assert!(path.exists());
        let before = send(&mut first, r#"{"cmd":"state"}"#)["state"].clone();

        // The app restarted: it finds the game, and resumes it as it was.
        let mut again = Session::default().with_save(path.clone());
        let saved = send(&mut again, r#"{"cmd":"saved_game"}"#);
        assert_eq!(saved["state"]["score"], before["score"]);
        assert_eq!(send(&mut again, r#"{"cmd":"resume"}"#)["state"], before);
        // The same draws follow.
        let play = r#"{"cmd":"play","cards":[0]}"#;
        assert_eq!(
            send(&mut first, play)["state"],
            send(&mut again, play)["state"]
        );

        // A save it can't read is reported, never resumed.
        std::fs::write(&path, r#"{"version":0}"#).unwrap();
        let mut other = Session::default().with_save(path.clone());
        assert!(send(&mut other, r#"{"cmd":"saved_game"}"#)["problem"].is_string());
        assert_eq!(send(&mut other, r#"{"cmd":"resume"}"#)["event"], "error");

        // A lost game leaves nothing to resume.
        while send(&mut again, r#"{"cmd":"state"}"#)["state"]["phase"] == "playing" {
            send(&mut again, play);
        }
        assert!(!path.exists());
        // Sessions without a save file never write one.
        let mut plain = Session::default();
        send(&mut plain, r#"{"cmd":"new_game","seed":7}"#);
        assert!(!dir.join("save.json").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_profile_follows_games_and_locks_hidden_items() {
        let dir = std::env::temp_dir().join(format!("omatro-collection-{}", std::process::id()));
        let path = dir.join("profile.json");
        let _ = std::fs::remove_dir_all(&dir);
        let mut session = Session::default().with_profile(path.clone());
        let decks = send(&mut session, r#"{"cmd":"decks"}"#);
        let raid = decks["decks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|d| d["id"] == "raid1")
            .unwrap();
        assert_eq!(raid["achievement"], "It's Not a Phase");
        let r = send(
            &mut session,
            r#"{"cmd":"new_game","seed":1,"deck":"raid1"}"#,
        );
        assert_eq!(
            r["message"],
            "RAID 1 is locked: It's Not a Phase unlocks it"
        );

        let reply: Value = serde_json::from_str(
            &session.handle_json(r#"{"cmd":"new_game","seed":1,"deck":"think_pad"}"#),
        )
        .unwrap();
        assert_eq!(reply["news"][0]["name"], "ThinkPad");
        assert!(path.exists());
        assert!(
            session
                .game
                .as_ref()
                .unwrap()
                .locked()
                .contains(&Joker::HashCollision)
        );

        let reply: Value =
            serde_json::from_str(&session.handle_json(r#"{"cmd":"collection"}"#)).unwrap();
        assert_eq!(reply["news"][0]["name"], "RTFM");
        let collection = &reply["collection"];
        assert_eq!(collection["found"], 1);
        assert_eq!(collection["families"][1]["name"], "Decks");
        // Read again by the next session.
        let again = Session::default().with_profile(path.clone());
        assert!(again.profile().unwrap().is_done(Achievement::Rtfm));
        // Sessions without a profile lock nothing.
        let mut plain = Session::default();
        assert_eq!(
            send(&mut plain, r#"{"cmd":"collection"}"#)["event"],
            "error"
        );
        send(&mut plain, r#"{"cmd":"new_game","seed":1,"deck":"raid1"}"#);
        assert!(plain.game.as_ref().unwrap().locked().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_game_left_for_another_goes_to_the_history() {
        let dir = std::env::temp_dir().join(format!("omatro-history-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let (save, profile) = (dir.join("save.json"), dir.join("profile.json"));
        let session = || {
            Session::default()
                .with_save(save.clone())
                .with_profile(profile.clone())
        };
        let mut first = session();
        send(
            &mut first,
            r#"{"cmd":"new_game","seed":9,"deck":"think_pad"}"#,
        );
        send(&mut first, r#"{"cmd":"select_blind"}"#);
        send(&mut first, r#"{"cmd":"play","cards":[0]}"#);
        // The next start leaves the saved game for a new one.
        let mut next = session();
        send(&mut next, r#"{"cmd":"new_game","seed":10}"#);
        let r = send(&mut next, r#"{"cmd":"collection"}"#);
        let history = &r["collection"]["history"];
        assert_eq!(history.as_array().unwrap().len(), 1);
        assert_eq!(
            (
                &history[0]["outcome"],
                &history[0]["deck"],
                &history[0]["seed"]
            ),
            (&"abandoned".into(), &"ThinkPad".into(), &9.into())
        );
        let stats = &r["collection"]["stats"];
        assert_eq!(
            (&stats["games"], &stats["abandoned"]),
            (&1.into(), &1.into())
        );
        assert_eq!(stats["decks"][0]["deck"], "ThinkPad");
        assert_eq!(stats["hands"][0]["played"], 1);
        // Kept for the sessions after.
        assert_eq!(session().profile().unwrap().history().len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn levels_unlock_deck_by_deck() {
        let mut session = Session::default().with_new_profile();
        let r = send(&mut session, r#"{"cmd":"decks"}"#);
        assert_eq!(r["stakes"].as_array().unwrap().len(), Stake::ALL.len());
        assert_eq!(r["decks"][0]["top_stake"], "daily_commute");
        let r = send(
            &mut session,
            r#"{"cmd":"new_game","seed":1,"deck":"think_pad","stake":"track_day"}"#,
        );
        assert_eq!(
            r["message"],
            "Track Day is locked: win at the level below with this deck first"
        );
        let r = send(
            &mut session,
            r#"{"cmd":"new_game","seed":1,"deck":"think_pad"}"#,
        );
        assert_eq!(r["state"]["stake"]["name"], "Daily Commute");
        // Without a profile, every level is open.
        let mut plain = Session::default();
        let r = send(
            &mut plain,
            r#"{"cmd":"new_game","seed":1,"stake":"hypercar"}"#,
        );
        assert_eq!(r["state"]["stake"]["rules"].as_array().unwrap().len(), 7);
    }
}
