//! Protocol between the engine and the display: one JSON command per line
//! in, one JSON response per line out.

use crate::game::{Cheat, DeckInfo, Game, SortBy, State};
use crate::glossary::{Term, glossary};
use crate::hand::HandKind;
use crate::joker::{Bonus, Joker};
use crate::run_deck::Deck;
use crate::score::Breakdown;
use serde::{Deserialize, Serialize};
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
    },
    /// Lists the starting decks, for the new game screen.
    Decks,
    /// Lists the terms card texts mention, with what they mean.
    Glossary,
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
    },
    Glossary {
        terms: Vec<Term>,
    },
}

#[derive(Default)]
pub struct Session {
    game: Option<Game>,
    debug: bool,
    /// Jokers of every new game that names none (debug only).
    start_jokers: Vec<Joker>,
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
        }
    }

    pub fn handle(&mut self, line: &str) -> Response {
        match serde_json::from_str(line) {
            Ok(command) => self
                .run(command)
                .unwrap_or_else(|message| Response::Error { message }),
            Err(e) => Response::Error {
                message: format!("unreadable command: {e}"),
            },
        }
    }

    fn run(&mut self, command: Command) -> Result<Response, String> {
        if let Command::NewGame {
            seed,
            mut jokers,
            deck,
        } = command
        {
            if jokers.is_empty() {
                jokers = self.start_jokers.clone();
            }
            let seed = seed.unwrap_or_else(random_seed);
            let game = self.game.insert(Game::with_deck(seed, jokers, deck));
            return Ok(Response::State {
                state: game.state(),
            });
        }
        if let Command::Decks = command {
            return Ok(Response::Decks {
                decks: Deck::ALL.iter().map(|&d| d.into()).collect(),
            });
        }
        if let Command::Glossary = command {
            return Ok(Response::Glossary { terms: glossary() });
        }
        if matches!(command, Command::Debug(_)) && !self.debug {
            return Err("debug commands are off (set OMATRO_DEBUG=1)".into());
        }
        let game = self.game.as_mut().ok_or("no game in progress")?;
        match command {
            Command::NewGame { .. } | Command::Decks | Command::Glossary => unreachable!(),
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
        assert_eq!(r["decks"].as_array().unwrap().len(), 15);
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
}
