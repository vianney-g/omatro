//! How a game unfolds: successive blinds, hands and discards.

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
pub const START_MONEY: u32 = 4;
pub const BLIND_REWARD: u32 = 4;
pub const MAX_INTEREST: u32 = 5;
pub const MAX_JOKERS: usize = 5;
pub const SHOP_OFFERS: usize = 2;
pub const REROLL_COST: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Playing,
    /// Blind won, reward not cashed out yet.
    Won,
    Shop,
    Lost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    NotPlaying,
    NotWon,
    NotInShop,
    NoDiscardsLeft,
    NotEnoughMoney,
    JokerSlotsFull,
    BadSelection(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameError::NotPlaying => write!(f, "the blind is over"),
            GameError::NotWon => write!(f, "the blind is not won yet"),
            GameError::NotInShop => write!(f, "the shop is closed"),
            GameError::NoDiscardsLeft => write!(f, "no discards left"),
            GameError::NotEnoughMoney => write!(f, "not enough money"),
            GameError::JokerSlotsFull => {
                write!(f, "no room for another joker ({MAX_JOKERS} at most)")
            }
            GameError::BadSelection(why) => write!(f, "invalid selection: {why}"),
        }
    }
}

impl std::error::Error for GameError {}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JokerInfo {
    pub id: Joker,
    pub name: &'static str,
    pub description: &'static str,
    pub price: u32,
    pub sell_value: u32,
}

impl From<Joker> for JokerInfo {
    fn from(joker: Joker) -> Self {
        JokerInfo {
            id: joker,
            name: joker.name(),
            description: joker.description(),
            price: joker.price(),
            sell_value: joker.sell_value(),
        }
    }
}

/// Money earned at the end of a blind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reward {
    pub blind: u32,
    /// $1 per unplayed hand.
    pub hands: u32,
    /// $1 per $5 held, at most `MAX_INTEREST`.
    pub interest: u32,
    pub jokers: u32,
    pub total: u32,
}

/// Snapshot of the game sent to the display.
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
    pub max_jokers: usize,
    pub money: u32,
    /// Present in the `won` phase.
    pub reward: Option<Reward>,
    /// Shop offers (empty outside the shop).
    pub shop: Vec<JokerInfo>,
    pub reroll_cost: u32,
}

/// Score needed to beat blind `n` (starting at 1).
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
    money: u32,
    reward: Option<Reward>,
    offers: Vec<Joker>,
    reroll_cost: u32,
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
            money: START_MONEY,
            reward: None,
            offers: Vec::new(),
            reroll_cost: REROLL_COST,
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
        // Hand sorted as at the table: descending rank, then suit.
        self.hand
            .sort_by_key(|c| (std::cmp::Reverse(c.rank), c.suit as u8));
    }

    /// Checks a selection and returns its sorted indices.
    fn validate(&self, indices: &[usize]) -> Result<Vec<usize>, GameError> {
        if self.phase != Phase::Playing {
            return Err(GameError::NotPlaying);
        }
        let mut sorted = indices.to_vec();
        sorted.sort();
        sorted.dedup();
        if sorted.len() != indices.len() {
            return Err(GameError::BadSelection("card chosen twice".into()));
        }
        if !(1..=MAX_PLAY).contains(&sorted.len()) {
            return Err(GameError::BadSelection(format!(
                "choose 1 to {MAX_PLAY} cards"
            )));
        }
        if let Some(&i) = sorted.iter().find(|&&i| i >= self.hand.len()) {
            return Err(GameError::BadSelection(format!("no card #{i}")));
        }
        Ok(sorted)
    }

    /// Removes the chosen cards from the hand and returns them in hand order.
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
            self.reward = Some(self.reward());
        } else if self.hands_left == 0 {
            self.phase = Phase::Lost;
        } else {
            self.draw();
        }
        Ok(breakdown)
    }

    /// The hand the chosen cards would make, without playing them.
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

    fn reward(&self) -> Reward {
        let hands = self.hands_left;
        let interest = (self.money / 5).min(MAX_INTEREST);
        let jokers = self
            .jokers
            .iter()
            .map(|j| j.on_blind_won(self.discards_left))
            .sum();
        Reward {
            blind: BLIND_REWARD,
            hands,
            interest,
            jokers,
            total: BLIND_REWARD + hands + interest + jokers,
        }
    }

    /// Cashes out the reward and opens the shop.
    pub fn cash_out(&mut self) -> Result<(), GameError> {
        if self.phase != Phase::Won {
            return Err(GameError::NotWon);
        }
        self.money += self.reward.take().map_or(0, |r| r.total);
        self.phase = Phase::Shop;
        self.reroll_cost = REROLL_COST;
        self.roll_offers();
        Ok(())
    }

    /// Draws new offers among the jokers not owned yet.
    fn roll_offers(&mut self) {
        let mut pool: Vec<Joker> = Joker::ALL
            .into_iter()
            .filter(|j| !self.jokers.contains(j))
            .collect();
        self.rng.shuffle(&mut pool);
        pool.truncate(SHOP_OFFERS);
        self.offers = pool;
    }

    fn in_shop(&self) -> Result<(), GameError> {
        if self.phase == Phase::Shop {
            Ok(())
        } else {
            Err(GameError::NotInShop)
        }
    }

    fn spend(&mut self, amount: u32) -> Result<(), GameError> {
        self.money = self
            .money
            .checked_sub(amount)
            .ok_or(GameError::NotEnoughMoney)?;
        Ok(())
    }

    pub fn buy(&mut self, offer: usize) -> Result<(), GameError> {
        self.in_shop()?;
        let joker = *self
            .offers
            .get(offer)
            .ok_or_else(|| GameError::BadSelection(format!("no offer #{offer}")))?;
        if self.jokers.len() >= MAX_JOKERS {
            return Err(GameError::JokerSlotsFull);
        }
        self.spend(joker.price())?;
        self.offers.remove(offer);
        self.jokers.push(joker);
        Ok(())
    }

    pub fn sell(&mut self, index: usize) -> Result<(), GameError> {
        self.in_shop()?;
        if index >= self.jokers.len() {
            return Err(GameError::BadSelection(format!("no joker #{index}")));
        }
        self.money += self.jokers.remove(index).sell_value();
        Ok(())
    }

    /// New offers; each reroll costs $1 more than the previous one.
    pub fn reroll(&mut self) -> Result<(), GameError> {
        self.in_shop()?;
        self.spend(self.reroll_cost)?;
        self.reroll_cost += 1;
        self.roll_offers();
        Ok(())
    }

    pub fn next_blind(&mut self) -> Result<(), GameError> {
        self.in_shop()?;
        self.offers.clear();
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
            max_jokers: MAX_JOKERS,
            money: self.money,
            reward: self.reward.clone(),
            shop: self.offers.iter().map(|&j| j.into()).collect(),
            reroll_cost: self.reroll_cost,
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
        assert_eq!(game.next_blind(), Err(GameError::NotInShop));
    }

    /// A game whose first blind was just won with a royal flush.
    fn won_game(seed: u64, jokers: Vec<Joker>) -> Game {
        let mut game = Game::new(seed, jokers);
        game.hand = cards("AS KS QS JS 10S 2H 3D 4C");
        game.play(&[0, 1, 2, 3, 4]).unwrap();
        game
    }

    #[test]
    fn winning_pays_a_reward() {
        let game = won_game(9, vec![Joker::Tontine]);
        let s = game.state();
        assert_eq!(s.phase, Phase::Won);
        // $4 + 3 hands left + 0 interest ($4) + 3 discards (tontine).
        assert_eq!(
            s.reward,
            Some(Reward {
                blind: 4,
                hands: 3,
                interest: 0,
                jokers: 3,
                total: 10
            })
        );
    }

    #[test]
    fn interest_is_capped() {
        let mut game = won_game(9, vec![]);
        game.money = 100;
        assert_eq!(game.reward().interest, MAX_INTEREST);
    }

    #[test]
    fn cash_out_opens_the_shop_then_next_blind() {
        let mut game = won_game(9, vec![]);
        assert_eq!(game.next_blind(), Err(GameError::NotInShop));
        game.cash_out().unwrap();
        let s = game.state();
        assert_eq!(
            (s.phase, s.money, s.reward),
            (Phase::Shop, START_MONEY + 7, None)
        );
        assert_eq!(s.shop.len(), SHOP_OFFERS);
        assert_eq!(game.cash_out(), Err(GameError::NotWon));
        game.next_blind().unwrap();
        let s = game.state();
        assert_eq!(
            (s.blind, s.target, s.score, s.phase),
            (2, 450, 0, Phase::Playing)
        );
        assert_eq!((s.hand.len(), s.shop.len()), (HAND_SIZE, 0));
    }

    #[test]
    fn buying_and_selling() {
        let mut game = won_game(9, vec![]);
        game.cash_out().unwrap();
        game.money = 20;
        let offer = game.offers[0];
        game.buy(0).unwrap();
        assert_eq!(game.jokers, [offer]);
        assert_eq!(game.money, 20 - offer.price());
        assert_eq!(game.offers.len(), SHOP_OFFERS - 1);
        assert!(game.buy(5).is_err());
        game.sell(0).unwrap();
        assert!(game.jokers.is_empty());
        assert_eq!(game.money, 20 - offer.price() + offer.sell_value());
    }

    #[test]
    fn shop_limits() {
        let mut game = won_game(9, vec![]);
        game.cash_out().unwrap();
        game.money = 0;
        assert_eq!(game.buy(0), Err(GameError::NotEnoughMoney));
        assert_eq!(game.reroll(), Err(GameError::NotEnoughMoney));
        game.money = 100;
        game.jokers = Joker::ALL[..MAX_JOKERS].to_vec();
        game.roll_offers();
        assert_eq!(game.buy(0), Err(GameError::JokerSlotsFull));
        assert!(game.offers.iter().all(|o| !game.jokers.contains(o)));
    }

    #[test]
    fn reroll_cost_rises() {
        let mut game = won_game(9, vec![]);
        game.cash_out().unwrap();
        game.money = 100;
        game.reroll().unwrap();
        game.reroll().unwrap();
        assert_eq!((game.money, game.reroll_cost), (100 - 5 - 6, 7));
        assert_eq!(game.offers.len(), SHOP_OFFERS);
    }

    #[test]
    fn shop_actions_need_the_shop() {
        let mut game = Game::new(9, vec![Joker::Baobab]);
        assert_eq!(game.buy(0), Err(GameError::NotInShop));
        assert_eq!(game.sell(0), Err(GameError::NotInShop));
        assert_eq!(game.reroll(), Err(GameError::NotInShop));
    }
}
