//! Déroulement d'une partie : blinds successives, mains et défausses.

use crate::card::{Card, standard_deck};
use crate::hand::{Evaluated, evaluate};
use crate::joker::Joker;
use crate::rng::Rng;
use crate::score::{Breakdown, score};
use serde::Serialize;
use std::fmt;

pub const HAND_SIZE: usize = 8;
pub const MAX_PLAY: usize = 5;
pub const HANDS_PER_BLIND: u32 = 4;
pub const DISCARDS_PER_BLIND: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Playing,
    Won,
    Lost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    NotPlaying,
    NotWon,
    NoDiscardsLeft,
    BadSelection(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameError::NotPlaying => write!(f, "la blind n'est plus en cours"),
            GameError::NotWon => write!(f, "la blind n'est pas encore gagnée"),
            GameError::NoDiscardsLeft => write!(f, "plus de défausse"),
            GameError::BadSelection(why) => write!(f, "sélection invalide : {why}"),
        }
    }
}

impl std::error::Error for GameError {}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JokerInfo {
    pub id: Joker,
    pub name: &'static str,
    pub description: &'static str,
}

impl From<Joker> for JokerInfo {
    fn from(joker: Joker) -> Self {
        JokerInfo {
            id: joker,
            name: joker.name(),
            description: joker.description(),
        }
    }
}

/// Instantané de la partie envoyé à l'affichage.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct State {
    pub seed: u64,
    pub blind: u32,
    pub target: u64,
    pub score: u64,
    pub hands_left: u32,
    pub discards_left: u32,
    pub phase: Phase,
    pub hand: Vec<Card>,
    pub deck_left: usize,
    pub jokers: Vec<JokerInfo>,
}

/// Score à atteindre pour la blind `n` (à partir de 1).
pub fn blind_target(n: u32) -> u64 {
    (300.0 * 1.5_f64.powi(n as i32 - 1)).round() as u64
}

pub struct Game {
    seed: u64,
    rng: Rng,
    deck: Vec<Card>,
    hand: Vec<Card>,
    jokers: Vec<Joker>,
    blind: u32,
    score: u64,
    hands_left: u32,
    discards_left: u32,
    phase: Phase,
}

impl Game {
    pub fn new(seed: u64, jokers: Vec<Joker>) -> Self {
        let mut game = Game {
            seed,
            rng: Rng::new(seed),
            deck: Vec::new(),
            hand: Vec::new(),
            jokers,
            blind: 1,
            score: 0,
            hands_left: 0,
            discards_left: 0,
            phase: Phase::Playing,
        };
        game.start_blind();
        game
    }

    fn start_blind(&mut self) {
        self.deck = standard_deck();
        self.rng.shuffle(&mut self.deck);
        self.hand.clear();
        self.score = 0;
        self.hands_left = HANDS_PER_BLIND;
        self.discards_left = DISCARDS_PER_BLIND;
        self.phase = Phase::Playing;
        self.draw();
    }

    fn draw(&mut self) {
        while self.hand.len() < HAND_SIZE
            && let Some(card) = self.deck.pop()
        {
            self.hand.push(card);
        }
        // Main rangée comme à la table : rang décroissant, puis couleur.
        self.hand
            .sort_by_key(|c| (std::cmp::Reverse(c.rank), c.suit as u8));
    }

    /// Vérifie une sélection et renvoie ses indices triés.
    fn validate(&self, indices: &[usize]) -> Result<Vec<usize>, GameError> {
        if self.phase != Phase::Playing {
            return Err(GameError::NotPlaying);
        }
        let mut sorted = indices.to_vec();
        sorted.sort();
        sorted.dedup();
        if sorted.len() != indices.len() {
            return Err(GameError::BadSelection("carte choisie deux fois".into()));
        }
        if !(1..=MAX_PLAY).contains(&sorted.len()) {
            return Err(GameError::BadSelection(format!(
                "choisir de 1 à {MAX_PLAY} cartes"
            )));
        }
        if let Some(&i) = sorted.iter().find(|&&i| i >= self.hand.len()) {
            return Err(GameError::BadSelection(format!("pas de carte n° {i}")));
        }
        Ok(sorted)
    }

    /// Retire de la main les cartes choisies et les rend dans l'ordre de la main.
    fn take(&mut self, indices: &[usize]) -> Result<Vec<Card>, GameError> {
        let sorted = self.validate(indices)?;
        let mut taken: Vec<Card> = sorted.iter().rev().map(|&i| self.hand.remove(i)).collect();
        taken.reverse();
        Ok(taken)
    }

    pub fn play(&mut self, indices: &[usize]) -> Result<Breakdown, GameError> {
        let played = self.take(indices)?;
        let breakdown = score(&played, &self.jokers);
        self.score += breakdown.total;
        self.hands_left -= 1;
        if self.score >= blind_target(self.blind) {
            self.phase = Phase::Won;
        } else if self.hands_left == 0 {
            self.phase = Phase::Lost;
        } else {
            self.draw();
        }
        Ok(breakdown)
    }

    /// La main que formeraient les cartes choisies, sans les jouer.
    pub fn preview(&self, indices: &[usize]) -> Result<Option<Evaluated>, GameError> {
        if indices.is_empty() {
            return Ok(None);
        }
        let cards: Vec<Card> = self
            .validate(indices)?
            .iter()
            .map(|&i| self.hand[i])
            .collect();
        Ok(Some(evaluate(&cards)))
    }

    pub fn discard(&mut self, indices: &[usize]) -> Result<(), GameError> {
        if self.phase == Phase::Playing && self.discards_left == 0 {
            return Err(GameError::NoDiscardsLeft);
        }
        self.take(indices)?;
        self.discards_left -= 1;
        self.draw();
        Ok(())
    }

    pub fn next_blind(&mut self) -> Result<(), GameError> {
        if self.phase != Phase::Won {
            return Err(GameError::NotWon);
        }
        self.blind += 1;
        self.start_blind();
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn set_hand(&mut self, hand: Vec<Card>) {
        self.hand = hand;
    }

    pub fn state(&self) -> State {
        State {
            seed: self.seed,
            blind: self.blind,
            target: blind_target(self.blind),
            score: self.score,
            hands_left: self.hands_left,
            discards_left: self.discards_left,
            phase: self.phase,
            hand: self.hand.clone(),
            deck_left: self.deck.len(),
            jokers: self.jokers.iter().map(|&j| j.into()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::cards;

    #[test]
    fn same_seed_same_hand() {
        assert_eq!(
            Game::new(1, vec![]).state().hand,
            Game::new(1, vec![]).state().hand
        );
        assert_ne!(
            Game::new(1, vec![]).state().hand,
            Game::new(2, vec![]).state().hand
        );
    }

    #[test]
    fn new_game_deals_a_full_hand() {
        let s = Game::new(3, vec![]).state();
        assert_eq!((s.hand.len(), s.deck_left), (HAND_SIZE, 52 - HAND_SIZE));
        assert_eq!(
            (s.hands_left, s.discards_left, s.target),
            (HANDS_PER_BLIND, DISCARDS_PER_BLIND, 300)
        );
    }

    #[test]
    fn playing_scores_and_refills() {
        let mut game = Game::new(4, vec![]);
        let first = game.state().hand[0];
        let b = game.play(&[0]).unwrap();
        assert_eq!(b.cards, [first]);
        let s = game.state();
        assert_eq!(
            (s.score, s.hands_left, s.hand.len()),
            (b.total, HANDS_PER_BLIND - 1, HAND_SIZE)
        );
    }

    #[test]
    fn played_cards_keep_hand_order() {
        let mut game = Game::new(5, vec![]);
        let hand = game.state().hand;
        assert_eq!(game.play(&[4, 1]).unwrap().cards, [hand[1], hand[4]]);
    }

    #[test]
    fn bad_selections_are_refused_without_side_effects() {
        let mut game = Game::new(6, vec![]);
        let before = game.state();
        assert!(matches!(game.play(&[]), Err(GameError::BadSelection(_))));
        assert!(matches!(
            game.play(&[0, 1, 2, 3, 4, 5]),
            Err(GameError::BadSelection(_))
        ));
        assert!(matches!(
            game.play(&[2, 2]),
            Err(GameError::BadSelection(_))
        ));
        assert!(matches!(game.play(&[8]), Err(GameError::BadSelection(_))));
        assert_eq!(game.state(), before);
    }

    #[test]
    fn hand_is_sorted_by_rank() {
        let hand = Game::new(10, vec![]).state().hand;
        assert!(hand.windows(2).all(|w| w[0].rank >= w[1].rank));
    }

    #[test]
    fn preview_does_not_play() {
        let mut game = Game::new(11, vec![]);
        game.hand = cards("KH KD 9S 7C 5H 4D 3S 2C");
        let before = game.state();
        let preview = game.preview(&[0, 1]).unwrap().unwrap();
        assert_eq!(preview.kind, crate::hand::HandKind::Pair);
        assert_eq!(game.preview(&[]).unwrap(), None);
        assert!(game.preview(&[9]).is_err());
        assert_eq!(game.state(), before);
    }

    #[test]
    fn discards_run_out() {
        let mut game = Game::new(7, vec![]);
        for _ in 0..DISCARDS_PER_BLIND {
            game.discard(&[0, 1]).unwrap();
        }
        assert_eq!(game.discard(&[0]), Err(GameError::NoDiscardsLeft));
        assert_eq!(game.state().hand.len(), HAND_SIZE);
    }

    #[test]
    fn losing_after_last_hand() {
        let mut game = Game::new(8, vec![]);
        game.hand = cards("2H 3D 5S 7C 9H 2D 3S 5C");
        game.hands_left = 1;
        game.play(&[0]).unwrap();
        assert_eq!(game.state().phase, Phase::Lost);
        assert_eq!(game.play(&[0]).unwrap_err(), GameError::NotPlaying);
        assert_eq!(game.next_blind(), Err(GameError::NotWon));
    }

    #[test]
    fn winning_moves_to_next_blind() {
        let mut game = Game::new(9, vec![]);
        game.hand = cards("AS KS QS JS 10S 2H 3D 4C");
        game.play(&[0, 1, 2, 3, 4]).unwrap(); // quinte flush royale : (100 + 51) × 8
        assert_eq!(game.state().phase, Phase::Won);
        game.next_blind().unwrap();
        let s = game.state();
        assert_eq!(
            (s.blind, s.target, s.score, s.phase),
            (2, 450, 0, Phase::Playing)
        );
        assert_eq!(s.hand.len(), HAND_SIZE);
    }
}
