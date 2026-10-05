//! Protocole entre le moteur et l'affichage : une commande JSON par ligne en
//! entrée, une réponse JSON par ligne en sortie.

use crate::game::{Game, State};
use crate::joker::Joker;
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
    },
    State,
    Play {
        cards: Vec<usize>,
    },
    Discard {
        cards: Vec<usize>,
    },
    NextBlind,
}

#[derive(Debug, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum Response {
    State { state: State },
    Scored { score: Breakdown, state: State },
    Error { message: String },
}

#[derive(Default)]
pub struct Session {
    game: Option<Game>,
}

impl Session {
    pub fn handle(&mut self, line: &str) -> Response {
        match serde_json::from_str(line) {
            Ok(command) => self
                .run(command)
                .unwrap_or_else(|message| Response::Error { message }),
            Err(e) => Response::Error {
                message: format!("commande illisible : {e}"),
            },
        }
    }

    fn run(&mut self, command: Command) -> Result<Response, String> {
        if let Command::NewGame { seed, jokers } = command {
            let seed = seed.unwrap_or_else(random_seed);
            let game = self.game.insert(Game::new(seed, jokers));
            return Ok(Response::State {
                state: game.state(),
            });
        }
        let game = self.game.as_mut().ok_or("aucune partie en cours")?;
        match command {
            Command::NewGame { .. } => unreachable!(),
            Command::State => {}
            Command::Play { cards } => {
                let score = game.play(&cards).map_err(|e| e.to_string())?;
                return Ok(Response::Scored {
                    score,
                    state: game.state(),
                });
            }
            Command::Discard { cards } => game.discard(&cards).map_err(|e| e.to_string())?,
            Command::NextBlind => game.next_blind().map_err(|e| e.to_string())?,
        }
        Ok(Response::State {
            state: game.state(),
        })
    }
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
            serde_json::from_str::<Command>(r#"{"cmd":"new_game","seed":3,"jokers":["baobab"]}"#)
                .unwrap(),
            Command::NewGame {
                seed: Some(3),
                jokers: vec![Joker::Baobab]
            }
        );
        assert_eq!(
            serde_json::from_str::<Command>(r#"{"cmd":"play","cards":[0,2]}"#).unwrap(),
            Command::Play { cards: vec![0, 2] }
        );
    }

    #[test]
    fn errors_before_any_game() {
        let mut session = Session::default();
        assert_eq!(send(&mut session, r#"{"cmd":"state"}"#)["event"], "error");
        assert_eq!(send(&mut session, "pas du json")["event"], "error");
    }

    #[test]
    fn full_exchange() {
        let mut session = Session::default();
        let r = send(
            &mut session,
            r#"{"cmd":"new_game","seed":42,"jokers":["griot"]}"#,
        );
        assert_eq!(r["event"], "state");
        assert_eq!(r["state"]["seed"], 42);
        assert_eq!(r["state"]["hand"].as_array().unwrap().len(), 8);
        assert_eq!(r["state"]["jokers"][0]["name"], "Griot");

        let r = send(&mut session, r#"{"cmd":"play","cards":[0,1]}"#);
        assert_eq!(r["event"], "scored");
        assert_eq!(r["state"]["hands_left"], 3);
        assert_eq!(r["score"]["total"], r["state"]["score"]);

        let r = send(&mut session, r#"{"cmd":"discard","cards":[0]}"#);
        assert_eq!(r["state"]["discards_left"], 2);

        let r = send(&mut session, r#"{"cmd":"next_blind"}"#);
        assert_eq!(r["event"], "error");
    }
}
