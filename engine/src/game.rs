//! How a game unfolds: antes of three blinds, hands and discards, the shop.

use crate::blind::{self, BlindKind, WINNING_ANTE};
use crate::card::{Card, standard_deck};
use crate::hand::{Evaluated, HandCounts, HandKind, HandLevels, evaluate_with};
use crate::joker::{Edition, Joker, OwnedJoker, Rarity};
use crate::rng::Rng;
use crate::score::{Breakdown, Table, copy_target, score};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const HAND_SIZE: usize = 8;
pub const MAX_PLAY: usize = 5;
pub const HANDS_PER_BLIND: u32 = 4;
pub const DISCARDS_PER_BLIND: u32 = 3;
pub const START_MONEY: u32 = 4;
pub const MAX_INTEREST: u32 = 5;
pub const MAX_JOKERS: usize = 5;
pub const SHOP_OFFERS: usize = 2;
pub const REROLL_COST: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    /// Before a blind: play it, or skip it if it is not the boss.
    BlindSelect,
    Playing,
    /// Blind won, reward not cashed out yet.
    Won,
    Shop,
    Lost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    NotSelecting,
    NotPlaying,
    NotWon,
    NotInShop,
    CannotSkipBoss,
    NoDiscardsLeft,
    NotEnoughMoney,
    JokerSlotsFull,
    BadSelection(String),
}

impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GameError::NotSelecting => write!(f, "no blind to choose right now"),
            GameError::NotPlaying => write!(f, "no blind in progress"),
            GameError::NotWon => write!(f, "the blind is not won yet"),
            GameError::NotInShop => write!(f, "the shop is closed"),
            GameError::CannotSkipBoss => write!(f, "the boss blind cannot be skipped"),
            GameError::NoDiscardsLeft => write!(f, "no discards left"),
            GameError::NotEnoughMoney => write!(f, "not enough money"),
            GameError::JokerSlotsFull => write!(f, "no room for another joker"),
            GameError::BadSelection(why) => write!(f, "invalid selection: {why}"),
        }
    }
}

impl std::error::Error for GameError {}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct EditionInfo {
    pub id: Edition,
    pub name: &'static str,
    pub description: &'static str,
    pub quip: &'static str,
}

impl From<Edition> for EditionInfo {
    fn from(edition: Edition) -> Self {
        EditionInfo {
            id: edition,
            name: edition.name(),
            description: edition.description(),
            quip: edition.quip(),
        }
    }
}

/// A joker as the display shows it, owned or on offer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JokerInfo {
    pub id: Joker,
    pub name: &'static str,
    pub description: &'static str,
    pub quip: &'static str,
    pub rarity: Rarity,
    pub rarity_name: &'static str,
    pub edition: Option<EditionInfo>,
    /// Price with the edition.
    pub price: u32,
    pub sell_value: u32,
    /// What a growing joker adds now, or what a Fork copies.
    pub current: Option<String>,
}

impl JokerInfo {
    fn new(owned: &OwnedJoker, current: Option<String>) -> Self {
        let joker = owned.joker;
        JokerInfo {
            id: joker,
            name: joker.name(),
            description: joker.description(),
            quip: joker.quip(),
            rarity: joker.rarity(),
            rarity_name: joker.rarity().name(),
            edition: owned.edition.map(EditionInfo::from),
            price: owned.price(),
            sell_value: owned.sell_value(),
            current,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BlindStatus {
    Upcoming,
    /// The next blind, waiting to be played or skipped.
    Select,
    Current,
    Defeated,
    Skipped,
}

/// One of the three blinds of the current ante.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BlindInfo {
    pub kind: BlindKind,
    pub name: &'static str,
    pub quip: &'static str,
    pub target: f64,
    pub reward: u32,
    pub status: BlindStatus,
}

/// A hand type's level and how often it was played, for the run info panel.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct HandInfo {
    pub kind: HandKind,
    pub name: &'static str,
    pub quip: &'static str,
    pub level: u32,
    pub chips: u64,
    pub mult: f64,
    pub played: u32,
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
    /// What happened to the jokers ("Beta Release broke.").
    pub notes: Vec<String>,
}

/// Snapshot of the game sent to the display.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct State {
    pub seed: u64,
    pub ante: u32,
    pub winning_ante: u32,
    /// The blind being chosen, played or just beaten.
    pub blind: BlindKind,
    /// The three blinds of the ante.
    pub blinds: Vec<BlindInfo>,
    pub target: f64,
    pub score: f64,
    pub hands_left: u32,
    pub discards_left: u32,
    pub phase: Phase,
    /// The winning ante's boss was just beaten; cashing out goes on in endless mode.
    pub victory: bool,
    pub hand: Vec<Card>,
    pub hand_size: usize,
    /// Cards left to draw during a blind, the whole deck otherwise; sorted by
    /// suit, then descending rank.
    pub remaining: Vec<Card>,
    pub deck_size: usize,
    /// Hand types weakest first, without the secret ones never played.
    pub hands: Vec<HandInfo>,
    pub jokers: Vec<JokerInfo>,
    pub max_jokers: usize,
    pub money: u32,
    /// Present in the `won` phase.
    pub reward: Option<Reward>,
    /// Shop offers (empty outside the shop).
    pub shop: Vec<JokerInfo>,
    pub reroll_cost: u32,
}

/// Shortcuts for testing the interface, only accepted with `OMATRO_DEBUG=1`.
#[derive(Debug, Default, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cheat {
    /// Sets the money.
    pub money: Option<u32>,
    /// Adds jokers, within the joker slots.
    #[serde(default)]
    pub jokers: Vec<Joker>,
    /// Edition of the added jokers.
    pub edition: Option<Edition>,
    /// Jumps to the choice of a blind (ante and blind default to the current ones).
    pub ante: Option<u32>,
    pub blind: Option<BlindKind>,
    /// Wins the blind in progress.
    #[serde(default)]
    pub win: bool,
    /// Raises each listed hand type by one level.
    #[serde(default)]
    pub level_up: Vec<HandKind>,
    /// Adds a random joker not owned yet, with a random edition (or none).
    #[serde(default)]
    pub random_joker: bool,
}

/// A joker for sale.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Offer {
    joker: Joker,
    edition: Option<Edition>,
}

pub struct Game {
    seed: u64,
    deck_rng: Rng,
    shop_rng: Rng,
    /// Chances while scoring (Heartbleed, Segfault…).
    score_rng: Rng,
    /// Chances at the end of a blind (Beta Release…).
    round_rng: Rng,
    /// The player's deck, which lasts the whole game.
    deck: Vec<Card>,
    /// Cards left to draw in the blind in progress.
    draw_pile: Vec<Card>,
    hand: Vec<Card>,
    levels: HandLevels,
    plays: HandCounts,
    jokers: Vec<OwnedJoker>,
    ante: u32,
    blind: BlindKind,
    /// Blinds skipped in the current ante.
    skipped: Vec<BlindKind>,
    score: f64,
    hands_left: u32,
    discards_left: u32,
    discards_used: u32,
    phase: Phase,
    victory: bool,
    money: u32,
    reward: Option<Reward>,
    offers: Vec<Offer>,
    reroll_cost: u32,
    /// Beta Release broke: it is gone from the shop, Stable Release shows up.
    beta_broken: bool,
}

impl Game {
    pub fn new(seed: u64, jokers: Vec<Joker>) -> Self {
        let mut game = Game {
            seed,
            deck_rng: Rng::stream(seed, "deck"),
            shop_rng: Rng::stream(seed, "shop"),
            score_rng: Rng::stream(seed, "score"),
            round_rng: Rng::stream(seed, "round"),
            deck: standard_deck(),
            draw_pile: Vec::new(),
            hand: Vec::new(),
            levels: HandLevels::default(),
            plays: HandCounts::default(),
            jokers: jokers
                .into_iter()
                .map(|j| OwnedJoker::new(j, None))
                .collect(),
            ante: 1,
            blind: BlindKind::Small,
            skipped: Vec::new(),
            score: 0.0,
            hands_left: 0,
            discards_left: 0,
            discards_used: 0,
            phase: Phase::BlindSelect,
            victory: false,
            money: START_MONEY,
            reward: None,
            offers: Vec::new(),
            reroll_cost: REROLL_COST,
            beta_broken: false,
        };
        game.await_blind();
        game
    }

    fn target(&self) -> f64 {
        blind::target(self.ante, self.blind)
    }

    fn count(&self, joker: Joker) -> usize {
        self.jokers.iter().filter(|j| j.joker == joker).count()
    }

    fn hand_size(&self) -> usize {
        HAND_SIZE + self.count(Joker::Ultrawide)
    }

    fn joker_slots(&self) -> usize {
        let transparent = self
            .jokers
            .iter()
            .filter(|j| j.edition == Some(Edition::Transparent))
            .count();
        MAX_JOKERS + transparent
    }

    fn four_fingers(&self) -> bool {
        self.count(Joker::Mvp) > 0
    }

    /// Waits for the player to choose the current blind; every card is back in
    /// the deck.
    fn await_blind(&mut self) {
        self.draw_pile = self.deck.clone();
        self.hand.clear();
        self.score = 0.0;
        self.hands_left = HANDS_PER_BLIND;
        self.discards_left = DISCARDS_PER_BLIND + self.count(Joker::UndoHistory) as u32;
        self.discards_used = 0;
        self.phase = Phase::BlindSelect;
    }

    /// Moves on to the next blind, or to the next ante after the boss.
    fn advance(&mut self) {
        match self.blind.next() {
            Some(next) => self.blind = next,
            None => {
                self.ante += 1;
                self.blind = BlindKind::Small;
                self.skipped.clear();
            }
        }
        self.await_blind();
    }

    fn selecting(&self) -> Result<(), GameError> {
        if self.phase == Phase::BlindSelect {
            Ok(())
        } else {
            Err(GameError::NotSelecting)
        }
    }

    /// Plays the blind on offer: shuffles the deck and deals a hand.
    pub fn select_blind(&mut self) -> Result<(), GameError> {
        self.selecting()?;
        // Jokers bought since the blind was announced count too.
        self.discards_left = DISCARDS_PER_BLIND + self.count(Joker::UndoHistory) as u32;
        self.deck_rng.shuffle(&mut self.draw_pile);
        self.phase = Phase::Playing;
        self.draw();
        Ok(())
    }

    pub fn skip_blind(&mut self) -> Result<(), GameError> {
        self.selecting()?;
        if !self.blind.skippable() {
            return Err(GameError::CannotSkipBoss);
        }
        self.skipped.push(self.blind);
        self.advance();
        Ok(())
    }

    fn draw(&mut self) {
        while self.hand.len() < self.hand_size()
            && let Some(card) = self.draw_pile.pop()
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
        let four_fingers = self.four_fingers();
        let evaluated = evaluate_with(&played, four_fingers);
        self.plays.add(evaluated.kind);
        let scoring: Vec<Card> = evaluated.scoring.iter().map(|&i| played[i]).collect();
        for joker in &mut self.jokers {
            joker.before_hand(&evaluated, &scoring);
        }
        let table = Table {
            played: &played,
            held: &self.hand,
            jokers: &self.jokers,
            levels: &self.levels,
            plays: &self.plays,
            four_fingers,
        };
        let mut breakdown = score(&table, &mut self.score_rng);
        self.money += breakdown.money;
        let notes = &mut breakdown.notes;
        self.jokers.retain_mut(|joker| {
            let kept = joker.after_hand();
            if !kept {
                notes.push(format!("{} ran out.", joker.joker.name()));
            }
            kept
        });

        self.score += breakdown.total;
        self.hands_left -= 1;
        if self.score >= self.target() {
            self.win_blind();
        } else if self.hands_left == 0 {
            self.phase = Phase::Lost;
        } else {
            self.draw();
        }
        Ok(breakdown)
    }

    fn win_blind(&mut self) {
        self.phase = Phase::Won;
        let mut reward = self.reward();
        let boss = self.blind == BlindKind::Boss;
        let rng = &mut self.round_rng;
        let mut broke_beta = false;
        self.jokers.retain_mut(|joker| {
            let kept = joker.end_of_blind(boss, rng);
            if !kept {
                reward.notes.push(format!("{} broke.", joker.joker.name()));
                broke_beta |= joker.joker == Joker::BetaRelease;
            }
            kept
        });
        self.beta_broken |= broke_beta;
        self.reward = Some(reward);
        self.victory = self.ante == WINNING_ANTE && boss;
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
        Ok(Some(evaluate_with(&cards, self.four_fingers())))
    }

    pub fn discard(&mut self, indices: &[usize]) -> Result<(), GameError> {
        if self.phase == Phase::Playing && self.discards_left == 0 {
            return Err(GameError::NoDiscardsLeft);
        }
        self.take(indices)?;
        self.discards_left -= 1;
        self.discards_used += 1;
        for joker in &mut self.jokers {
            joker.on_discard();
        }
        self.draw();
        Ok(())
    }

    fn reward(&self) -> Reward {
        let blind = self.blind.reward();
        let hands = self.hands_left;
        let interest = (self.money / 5).min(MAX_INTEREST);
        let jokers = (0..self.jokers.len())
            .filter_map(|i| copy_target(&self.jokers, i))
            .map(|t| {
                self.jokers[t]
                    .joker
                    .blind_money(self.discards_left, self.discards_used)
            })
            .sum();
        Reward {
            blind,
            hands,
            interest,
            jokers,
            total: blind + hands + interest + jokers,
            notes: Vec::new(),
        }
    }

    /// Cashes out the reward and opens the shop (after a victory, this goes on
    /// in endless mode).
    pub fn cash_out(&mut self) -> Result<(), GameError> {
        if self.phase != Phase::Won {
            return Err(GameError::NotWon);
        }
        self.money += self.reward.take().map_or(0, |r| r.total);
        self.victory = false;
        self.phase = Phase::Shop;
        self.reroll_cost = REROLL_COST;
        self.roll_offers();
        Ok(())
    }

    /// Whether a joker may show up in the shop.
    fn on_sale(&self, joker: Joker) -> bool {
        let owned = self.jokers.iter().any(|j| j.joker == joker);
        let offered = self.offers.iter().any(|o| o.joker == joker);
        let lifecycle = match joker {
            Joker::BetaRelease => !self.beta_broken,
            Joker::StableRelease => self.beta_broken,
            _ => true,
        };
        !owned && !offered && lifecycle && joker.rarity() != Rarity::Legendary
    }

    /// Draws new offers: a rarity for each slot (70% core, 25% extra, 5% AUR),
    /// a joker of that rarity not owned yet, and sometimes an edition.
    fn roll_offers(&mut self) {
        self.offers.clear();
        for _ in 0..SHOP_OFFERS {
            let roll = self.shop_rng.unit();
            let mut odds = 0.0;
            let rarity = Rarity::ALL
                .into_iter()
                .find(|r| {
                    odds += r.odds();
                    roll < odds
                })
                .unwrap_or(Rarity::Core);
            let of_rarity: Vec<Joker> = Joker::ALL
                .into_iter()
                .filter(|&j| j.rarity() == rarity && self.on_sale(j))
                .collect();
            let pool = if of_rarity.is_empty() {
                Joker::ALL
                    .into_iter()
                    .filter(|&j| self.on_sale(j))
                    .collect()
            } else {
                of_rarity
            };
            if pool.is_empty() {
                break;
            }
            let joker = pool[self.shop_rng.below(pool.len() as u64) as usize];
            let roll = self.shop_rng.unit();
            let mut odds = 0.0;
            let edition = Edition::ALL.into_iter().rev().find(|e| {
                odds += e.odds();
                roll < odds
            });
            self.offers.push(Offer { joker, edition });
        }
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

    /// Whether a joker of this edition fits: a Transparent one brings its own slot.
    fn has_room(&self, edition: Option<Edition>) -> bool {
        edition == Some(Edition::Transparent) || self.jokers.len() < self.joker_slots()
    }

    pub fn buy(&mut self, offer: usize) -> Result<(), GameError> {
        self.in_shop()?;
        let Offer { joker, edition } = *self
            .offers
            .get(offer)
            .ok_or_else(|| GameError::BadSelection(format!("no offer #{offer}")))?;
        if !self.has_room(edition) {
            return Err(GameError::JokerSlotsFull);
        }
        let owned = OwnedJoker::new(joker, edition);
        self.spend(owned.price())?;
        self.offers.remove(offer);
        self.jokers.push(owned);
        Ok(())
    }

    pub fn sell(&mut self, index: usize) -> Result<(), GameError> {
        self.in_shop()?;
        if index >= self.jokers.len() {
            return Err(GameError::BadSelection(format!("no joker #{index}")));
        }
        self.money += self.jokers.remove(index).sell_value();
        for joker in &mut self.jokers {
            joker.on_card_sold();
        }
        Ok(())
    }

    /// Moves a joker within the row (order matters: left to right, and Fork
    /// copies its right neighbour).
    pub fn move_joker(&mut self, from: usize, to: usize) -> Result<(), GameError> {
        if from >= self.jokers.len() || to >= self.jokers.len() {
            return Err(GameError::BadSelection(format!(
                "no joker #{}",
                from.max(to)
            )));
        }
        let joker = self.jokers.remove(from);
        self.jokers.insert(to, joker);
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

    /// Leaves the shop for the choice of the next blind.
    pub fn next_blind(&mut self) -> Result<(), GameError> {
        self.in_shop()?;
        self.offers.clear();
        self.advance();
        Ok(())
    }

    pub fn cheat(&mut self, cheat: Cheat) -> Result<(), GameError> {
        if let Some(money) = cheat.money {
            self.money = money;
        }
        for joker in cheat.jokers {
            if !self.has_room(cheat.edition) {
                return Err(GameError::JokerSlotsFull);
            }
            self.jokers.push(OwnedJoker::new(joker, cheat.edition));
        }
        if cheat.ante.is_some() || cheat.blind.is_some() {
            if let Some(ante) = cheat.ante {
                self.ante = ante.max(1);
                self.blind = BlindKind::Small;
            }
            if let Some(blind) = cheat.blind {
                self.blind = blind;
            }
            self.skipped.clear();
            self.reward = None;
            self.victory = false;
            self.offers.clear();
            self.await_blind();
        }
        if cheat.random_joker {
            let mut rng = Rng::stream(self.seed ^ self.jokers.len() as u64, "cheat");
            let pool: Vec<Joker> = Joker::ALL
                .into_iter()
                .filter(|&j| self.on_sale(j))
                .collect();
            let joker = pool[rng.below(pool.len() as u64) as usize];
            let edition = Edition::ALL.get(rng.below(5) as usize).copied();
            if !self.has_room(edition) {
                return Err(GameError::JokerSlotsFull);
            }
            self.jokers.push(OwnedJoker::new(joker, edition));
        }
        for kind in cheat.level_up {
            self.levels.raise(kind, 1);
        }
        if cheat.win {
            if self.phase != Phase::Playing {
                return Err(GameError::NotPlaying);
            }
            self.score = self.score.max(self.target());
            self.win_blind();
        }
        Ok(())
    }

    pub fn levels(&self) -> &HandLevels {
        &self.levels
    }

    #[cfg(test)]
    pub(crate) fn set_hand(&mut self, hand: Vec<Card>) {
        self.hand = hand;
    }

    fn blind_status(&self, kind: BlindKind) -> BlindStatus {
        let order = |k| BlindKind::ALL.iter().position(|&b| b == k);
        if order(kind) < order(self.blind) {
            if self.skipped.contains(&kind) {
                BlindStatus::Skipped
            } else {
                BlindStatus::Defeated
            }
        } else if kind == self.blind {
            match self.phase {
                Phase::BlindSelect => BlindStatus::Select,
                Phase::Won | Phase::Shop => BlindStatus::Defeated,
                Phase::Playing | Phase::Lost => BlindStatus::Current,
            }
        } else {
            BlindStatus::Upcoming
        }
    }

    fn joker_info(&self, index: usize) -> JokerInfo {
        let owned = &self.jokers[index];
        let current = if owned.joker == Joker::Fork {
            Some(match copy_target(&self.jokers, index) {
                Some(t) => format!("Copying {}", self.jokers[t].joker.name()),
                None => "Nothing to copy".to_string(),
            })
        } else {
            owned.current()
        };
        JokerInfo::new(owned, current)
    }

    pub fn state(&self) -> State {
        State {
            seed: self.seed,
            ante: self.ante,
            winning_ante: WINNING_ANTE,
            blind: self.blind,
            blinds: BlindKind::ALL
                .iter()
                .map(|&kind| BlindInfo {
                    kind,
                    name: kind.name(),
                    quip: kind.quip(),
                    target: blind::target(self.ante, kind),
                    reward: kind.reward(),
                    status: self.blind_status(kind),
                })
                .collect(),
            target: self.target(),
            score: self.score,
            hands_left: self.hands_left,
            discards_left: self.discards_left,
            phase: self.phase,
            victory: self.victory,
            hand: self.hand.clone(),
            hand_size: self.hand_size(),
            remaining: {
                let mut cards = match self.phase {
                    Phase::Playing | Phase::Won | Phase::Lost => self.draw_pile.clone(),
                    Phase::BlindSelect | Phase::Shop => self.deck.clone(),
                };
                cards.sort_by_key(|c| (c.suit as u8, std::cmp::Reverse(c.rank)));
                cards
            },
            deck_size: self.deck.len(),
            hands: HandKind::ALL
                .iter()
                .filter(|k| !k.secret() || self.plays.get(**k) > 0)
                .map(|&kind| {
                    let (chips, mult) = self.levels.values(kind);
                    HandInfo {
                        kind,
                        name: kind.name(),
                        quip: kind.quip(),
                        level: self.levels.level(kind),
                        chips,
                        mult,
                        played: self.plays.get(kind),
                    }
                })
                .collect(),
            jokers: (0..self.jokers.len()).map(|i| self.joker_info(i)).collect(),
            max_jokers: self.joker_slots(),
            money: self.money,
            reward: self.reward.clone(),
            shop: self
                .offers
                .iter()
                .map(|o| JokerInfo::new(&OwnedJoker::new(o.joker, o.edition), None))
                .collect(),
            reroll_cost: self.reroll_cost,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::cards;

    /// A game in its first blind, hand dealt.
    fn playing(seed: u64, jokers: Vec<Joker>) -> Game {
        let mut game = Game::new(seed, jokers);
        game.select_blind().unwrap();
        game
    }

    fn statuses(game: &Game) -> Vec<BlindStatus> {
        game.state().blinds.iter().map(|b| b.status).collect()
    }

    #[test]
    fn new_game_waits_for_the_small_blind() {
        let game = Game::new(3, vec![]);
        let s = game.state();
        assert_eq!(
            (s.phase, s.ante, s.blind, s.target),
            (Phase::BlindSelect, 1, BlindKind::Small, 300.0)
        );
        assert!(s.hand.is_empty());
        use BlindStatus::*;
        assert_eq!(statuses(&game), [Select, Upcoming, Upcoming]);
    }

    #[test]
    fn same_seed_same_hand() {
        assert_eq!(
            playing(1, vec![]).state().hand,
            playing(1, vec![]).state().hand
        );
        assert_ne!(
            playing(1, vec![]).state().hand,
            playing(2, vec![]).state().hand
        );
    }

    #[test]
    fn selecting_deals_a_full_hand() {
        let s = playing(3, vec![]).state();
        assert_eq!(s.phase, Phase::Playing);
        assert_eq!(
            (s.hand.len(), s.remaining.len()),
            (HAND_SIZE, 52 - HAND_SIZE)
        );
        assert_eq!(
            (s.hands_left, s.discards_left),
            (HANDS_PER_BLIND, DISCARDS_PER_BLIND)
        );
        assert_eq!(s.blinds[0].status, BlindStatus::Current);
    }

    #[test]
    fn skipping_moves_to_the_next_blind_but_not_past_the_boss() {
        let mut game = Game::new(3, vec![]);
        game.skip_blind().unwrap();
        game.skip_blind().unwrap();
        let s = game.state();
        assert_eq!((s.blind, s.target), (BlindKind::Boss, 600.0));
        use BlindStatus::*;
        assert_eq!(statuses(&game), [Skipped, Skipped, Select]);
        assert_eq!(game.skip_blind(), Err(GameError::CannotSkipBoss));
        game.select_blind().unwrap();
        assert_eq!(game.skip_blind(), Err(GameError::NotSelecting));
    }

    #[test]
    fn the_deck_lasts_and_every_card_comes_back() {
        let mut game = playing(12, vec![]);
        game.play(&[0, 1]).unwrap();
        game.discard(&[0]).unwrap();
        let s = game.state();
        assert_eq!((s.remaining.len(), s.deck_size), (52 - HAND_SIZE - 3, 52));
        let ids: std::collections::HashSet<u32> =
            s.hand.iter().chain(&s.remaining).map(|c| c.id).collect();
        assert_eq!(ids.len(), 52 - 3);
        game.cheat(Cheat {
            blind: Some(BlindKind::Big),
            ..Cheat::default()
        })
        .unwrap();
        assert_eq!(game.state().remaining, {
            let mut deck = standard_deck();
            deck.sort_by_key(|c| (c.suit as u8, std::cmp::Reverse(c.rank)));
            deck
        });
    }

    #[test]
    fn hands_are_counted_and_secret_ones_revealed_once_played() {
        let mut game = playing(13, vec![]);
        let kinds = |game: &Game| {
            game.state()
                .hands
                .iter()
                .map(|h| h.kind)
                .collect::<Vec<_>>()
        };
        assert_eq!(kinds(&game), HandKind::ALL[..9]);
        game.hand = cards("7S 7S 7S 7S 7S 2H 3D 4C");
        game.play(&[0, 1, 2, 3, 4]).unwrap();
        let s = game.state();
        let fork_bomb = s
            .hands
            .iter()
            .find(|h| h.kind == HandKind::ForkBomb)
            .unwrap();
        assert_eq!((fork_bomb.played, fork_bomb.level), (1, 1));
        assert_eq!(kinds(&game).len(), 10);
    }

    #[test]
    fn levelled_hands_score_more() {
        let mut game = playing(14, vec![]);
        game.cheat(Cheat {
            level_up: vec![HandKind::Pair, HandKind::Pair],
            ..Cheat::default()
        })
        .unwrap();
        let pair = game.state().hands[1].clone();
        assert_eq!((pair.level, pair.chips, pair.mult), (3, 40, 4.0));
        game.hand = cards("KH KD 9S 7C 5H 4D 3S 2C");
        assert_eq!(game.play(&[0, 1]).unwrap().total, (40.0 + 20.0) * 4.0);
    }

    #[test]
    fn playing_scores_and_refills() {
        let mut game = playing(4, vec![]);
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
        let mut game = playing(5, vec![]);
        let hand = game.state().hand;
        assert_eq!(game.play(&[4, 1]).unwrap().cards, [hand[1], hand[4]]);
    }

    #[test]
    fn bad_selections_are_refused_without_side_effects() {
        let mut game = playing(6, vec![]);
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
    fn no_play_before_selecting() {
        let mut game = Game::new(6, vec![]);
        assert_eq!(game.play(&[0]).unwrap_err(), GameError::NotPlaying);
    }

    #[test]
    fn hand_is_sorted_by_rank() {
        let hand = playing(10, vec![]).state().hand;
        assert!(hand.windows(2).all(|w| w[0].rank >= w[1].rank));
    }

    #[test]
    fn preview_does_not_play() {
        let mut game = playing(11, vec![]);
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
        let mut game = playing(7, vec![]);
        for _ in 0..DISCARDS_PER_BLIND {
            game.discard(&[0, 1]).unwrap();
        }
        assert_eq!(game.discard(&[0]), Err(GameError::NoDiscardsLeft));
        assert_eq!(game.state().hand.len(), HAND_SIZE);
    }

    #[test]
    fn losing_after_last_hand() {
        let mut game = playing(8, vec![]);
        game.hand = cards("2H 3D 5S 7C 9H 2D 3S 5C");
        game.hands_left = 1;
        game.play(&[0]).unwrap();
        assert_eq!(game.state().phase, Phase::Lost);
        assert_eq!(game.play(&[0]).unwrap_err(), GameError::NotPlaying);
        assert_eq!(game.next_blind(), Err(GameError::NotInShop));
    }

    /// Wins the blind in progress with a royal flush (1,208 points).
    fn win(game: &mut Game) {
        game.hand = cards("AS KS QS JS 10S 2H 3D 4C");
        game.play(&[0, 1, 2, 3, 4]).unwrap();
    }

    /// A game whose first blind was just won.
    fn won_game(seed: u64, jokers: Vec<Joker>) -> Game {
        let mut game = playing(seed, jokers);
        win(&mut game);
        game
    }

    #[test]
    fn winning_pays_a_reward() {
        let game = won_game(9, vec![Joker::CloudExit]);
        let s = game.state();
        assert_eq!(s.phase, Phase::Won);
        assert!(!s.victory);
        // $3 (small blind) + 3 hands left + 0 interest ($4) + 3 discards (Cloud Exit).
        assert_eq!(
            s.reward,
            Some(Reward {
                blind: 3,
                hands: 3,
                interest: 0,
                jokers: 3,
                total: 9,
                notes: vec![],
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
    fn cash_out_opens_the_shop_then_the_next_blind() {
        let mut game = won_game(9, vec![]);
        assert_eq!(game.next_blind(), Err(GameError::NotInShop));
        game.cash_out().unwrap();
        let s = game.state();
        assert_eq!(
            (s.phase, s.money, s.reward),
            (Phase::Shop, START_MONEY + 6, None)
        );
        assert_eq!(s.shop.len(), SHOP_OFFERS);
        assert_eq!(game.cash_out(), Err(GameError::NotWon));
        game.next_blind().unwrap();
        let s = game.state();
        assert_eq!(
            (s.ante, s.blind, s.target, s.score, s.phase),
            (1, BlindKind::Big, 450.0, 0.0, Phase::BlindSelect)
        );
        assert_eq!((s.hand.len(), s.shop.len()), (0, 0));
        use BlindStatus::*;
        assert_eq!(statuses(&game), [Defeated, Select, Upcoming]);
    }

    #[test]
    fn the_boss_leads_to_the_next_ante() {
        let mut game = Game::new(9, vec![]);
        game.skip_blind().unwrap();
        game.skip_blind().unwrap();
        game.select_blind().unwrap();
        win(&mut game);
        game.cash_out().unwrap();
        game.next_blind().unwrap();
        let s = game.state();
        assert_eq!((s.ante, s.blind, s.target), (2, BlindKind::Small, 800.0));
        use BlindStatus::*;
        assert_eq!(statuses(&game), [Select, Upcoming, Upcoming]);
    }

    #[test]
    fn beating_the_winning_boss_is_a_victory_then_endless() {
        let mut game = Game::new(9, vec![]);
        game.cheat(Cheat {
            ante: Some(WINNING_ANTE),
            blind: Some(BlindKind::Boss),
            ..Cheat::default()
        })
        .unwrap();
        game.select_blind().unwrap();
        game.cheat(Cheat {
            win: true,
            ..Cheat::default()
        })
        .unwrap();
        assert!(game.state().victory);
        game.cash_out().unwrap();
        assert!(!game.state().victory);
        game.next_blind().unwrap();
        let s = game.state();
        assert_eq!((s.ante, s.target), (WINNING_ANTE + 1, 110_000.0));
    }

    #[test]
    fn cheats_set_money_jokers_and_blind() {
        let mut game = Game::new(9, vec![]);
        game.cheat(Cheat {
            money: Some(99),
            jokers: vec![Joker::Hotwire],
            ante: Some(3),
            ..Cheat::default()
        })
        .unwrap();
        let s = game.state();
        assert_eq!((s.money, s.jokers.len(), s.ante), (99, 1, 3));
        assert_eq!((s.blind, s.phase), (BlindKind::Small, Phase::BlindSelect));
        assert_eq!(
            game.cheat(Cheat {
                win: true,
                ..Cheat::default()
            }),
            Err(GameError::NotPlaying)
        );
    }

    #[test]
    fn buying_and_selling() {
        let mut game = won_game(9, vec![]);
        game.cash_out().unwrap();
        game.money = 20;
        let offer = game.offers[0];
        let owned = OwnedJoker::new(offer.joker, offer.edition);
        game.buy(0).unwrap();
        assert_eq!(game.jokers, [owned.clone()]);
        assert_eq!(game.money, 20 - owned.price());
        assert_eq!(game.offers.len(), SHOP_OFFERS - 1);
        assert!(game.buy(5).is_err());
        game.sell(0).unwrap();
        assert!(game.jokers.is_empty());
        assert_eq!(game.money, 20 - owned.price() + owned.sell_value());
    }

    #[test]
    fn shop_limits() {
        let mut game = won_game(9, vec![]);
        game.cash_out().unwrap();
        game.money = 0;
        assert_eq!(game.buy(0), Err(GameError::NotEnoughMoney));
        assert_eq!(game.reroll(), Err(GameError::NotEnoughMoney));
        game.money = 100;
        game.jokers = Joker::ALL[..MAX_JOKERS]
            .iter()
            .map(|&j| OwnedJoker::new(j, None))
            .collect();
        game.offers = vec![Offer {
            joker: Joker::Fork,
            edition: None,
        }];
        assert_eq!(game.buy(0), Err(GameError::JokerSlotsFull));
        // A transparent joker brings its own slot.
        game.offers[0].edition = Some(Edition::Transparent);
        game.buy(0).unwrap();
        assert_eq!(game.state().max_jokers, MAX_JOKERS + 1);
        game.roll_offers();
        assert!(
            game.offers
                .iter()
                .all(|o| game.jokers.iter().all(|j| j.joker != o.joker))
        );
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
    fn rerolling_does_not_change_the_next_deal() {
        let next_hand = |rerolls| {
            let mut game = won_game(9, vec![]);
            game.cash_out().unwrap();
            game.money = 100;
            for _ in 0..rerolls {
                game.reroll().unwrap();
            }
            game.next_blind().unwrap();
            game.select_blind().unwrap();
            game.state().hand
        };
        assert_eq!(next_hand(0), next_hand(3));
    }

    #[test]
    fn jokers_change_hand_size_and_discards() {
        let game = playing(15, vec![Joker::Ultrawide, Joker::UndoHistory]);
        let s = game.state();
        assert_eq!((s.hand.len(), s.hand_size, s.discards_left), (9, 9, 4));
    }

    #[test]
    fn jokers_pay_at_the_end_of_the_blind() {
        let game = won_game(
            16,
            vec![
                Joker::Sponsorware,
                Joker::DataHoarder,
                Joker::Fork,
                Joker::Sponsorware,
            ],
        );
        // 4 + 2×3 (no discard used) + 4 (Fork copying Sponsorware) + 4.
        assert_eq!(game.state().reward.unwrap().jokers, 18);
    }

    #[test]
    fn beta_release_breaks_then_stable_release_shows_up() {
        let mut game = Game::new(17, vec![]);
        for _ in 0..200 {
            game.jokers = vec![OwnedJoker::new(Joker::BetaRelease, None)];
            game.cheat(Cheat {
                blind: Some(BlindKind::Small),
                ..Cheat::default()
            })
            .unwrap();
            game.select_blind().unwrap();
            win(&mut game);
            if game.jokers.is_empty() {
                break;
            }
        }
        assert!(game.beta_broken);
        assert_eq!(game.state().reward.unwrap().notes, ["Beta Release broke."]);
        assert!(!game.on_sale(Joker::BetaRelease) && game.on_sale(Joker::StableRelease));
    }

    #[test]
    fn selling_grows_orphan_cleanup() {
        let mut game = won_game(18, vec![Joker::OrphanCleanup, Joker::Dotfiles]);
        game.cash_out().unwrap();
        game.sell(1).unwrap();
        assert_eq!(game.jokers[0].counter, 1.25);
        assert_eq!(
            game.state().jokers[0].current.as_deref(),
            Some("Currently ×1.25 mult")
        );
    }

    #[test]
    fn jokers_can_be_reordered() {
        let mut game = playing(19, vec![Joker::Fork, Joker::Dotfiles, Joker::Omakase]);
        assert_eq!(
            game.state().jokers[0].current.as_deref(),
            Some("Copying Dotfiles")
        );
        game.move_joker(0, 2).unwrap();
        let ids: Vec<Joker> = game.state().jokers.iter().map(|j| j.id).collect();
        assert_eq!(ids, [Joker::Dotfiles, Joker::Omakase, Joker::Fork]);
        assert_eq!(
            game.state().jokers[2].current.as_deref(),
            Some("Nothing to copy")
        );
        assert!(game.move_joker(0, 3).is_err());
    }

    #[test]
    fn shop_rarities_follow_the_odds() {
        let mut game = won_game(20, vec![]);
        game.cash_out().unwrap();
        let mut counts = std::collections::HashMap::new();
        for _ in 0..2000 {
            game.roll_offers();
            for offer in &game.offers {
                *counts.entry(offer.joker.rarity()).or_insert(0) += 1;
            }
        }
        let share = |r| counts.get(&r).copied().unwrap_or(0) as f64 / 4000.0;
        assert!((share(Rarity::Core) - 0.70).abs() < 0.03, "{counts:?}");
        assert!((share(Rarity::Aur) - 0.05).abs() < 0.02, "{counts:?}");
        assert_eq!(share(Rarity::Legendary), 0.0);
    }

    #[test]
    fn shop_actions_need_the_shop() {
        let mut game = playing(9, vec![Joker::Dotfiles]);
        assert_eq!(game.buy(0), Err(GameError::NotInShop));
        assert_eq!(game.sell(0), Err(GameError::NotInShop));
        assert_eq!(game.reroll(), Err(GameError::NotInShop));
    }
}
