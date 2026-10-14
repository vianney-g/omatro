//! How a game unfolds: antes of three blinds, hands and discards, the shop.

use crate::blind::{self, BlindKind, WINNING_ANTE};
use crate::card::{ACE, Card, Suit, standard_deck};
use crate::consumable::{CONSUMABLE_SLOTS, Command, Consumable, Theme};
use crate::hand::{Evaluated, HandCounts, HandKind, HandLevels, evaluate_with};
use crate::joker::{Bonus, Joker, OwnedJoker, Rarity};
use crate::modifier::{Edition, Enhancement, Seal};
use crate::rng::Rng;
use crate::score::{Breakdown, Table, copy_target, score};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
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
    ClipboardFull,
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
            GameError::ClipboardFull => write!(f, "the clipboard is full"),
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
    /// Always "joker", to tell shop offers apart.
    pub kind: &'static str,
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
    /// What a Fork copies.
    pub current: Option<String>,
    /// What a growing joker gives right now, for its badge.
    pub bonus: Option<Bonus>,
}

impl JokerInfo {
    fn new(owned: &OwnedJoker, current: Option<String>, bonus: Option<Bonus>) -> Self {
        let joker = owned.joker;
        JokerInfo {
            kind: "joker",
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
            bonus,
        }
    }
}

/// A consumable as the display shows it, in the clipboard or on offer.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ConsumableInfo {
    /// `kind` ("theme") and `id`.
    #[serde(flatten)]
    pub consumable: Consumable,
    pub name: &'static str,
    pub description: String,
    pub quip: &'static str,
    pub price: u32,
    pub sell_value: u32,
    /// Cards of the hand to select before using it (both 0 when none).
    pub min_cards: usize,
    pub max_cards: usize,
}

impl ConsumableInfo {
    fn new(consumable: Consumable, price: u32) -> Self {
        ConsumableInfo {
            consumable,
            name: consumable.name(),
            description: consumable.description(),
            quip: consumable.quip(),
            price,
            sell_value: consumable.sell_value(),
            min_cards: consumable.targets().map_or(0, |t| t.0),
            max_cards: consumable.targets().map_or(0, |t| t.1),
        }
    }
}

/// A shop offer: a joker or a consumable, told apart by `kind`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum OfferInfo {
    Joker(JokerInfo),
    Consumable(ConsumableInfo),
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
    /// Enterprise cards held in hand.
    pub cards: u32,
    pub total: u32,
    /// What happened to the jokers ("Beta Release broke.").
    pub notes: Vec<String>,
}

/// Name, effect and quip of an enhancement, a seal or an edition.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ModifierInfo {
    pub id: String,
    pub name: &'static str,
    pub description: &'static str,
    pub quip: &'static str,
}

/// Every enhancement, seal and edition, for the display to name what cards carry.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Modifiers {
    pub enhancements: Vec<ModifierInfo>,
    pub seals: Vec<ModifierInfo>,
    pub editions: Vec<ModifierInfo>,
}

fn id_of(value: impl Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

impl Modifiers {
    fn catalog() -> Self {
        Modifiers {
            enhancements: Enhancement::ALL
                .iter()
                .map(|&e| ModifierInfo {
                    id: id_of(e),
                    name: e.name(),
                    description: e.description(),
                    quip: e.quip(),
                })
                .collect(),
            seals: Seal::ALL
                .iter()
                .map(|&s| ModifierInfo {
                    id: id_of(s),
                    name: s.name(),
                    description: s.description(),
                    quip: s.quip(),
                })
                .collect(),
            editions: Edition::ALL
                .iter()
                .map(|&e| ModifierInfo {
                    id: id_of(e),
                    name: e.name(),
                    description: e.description(),
                    quip: e.quip(),
                })
                .collect(),
        }
    }
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
    /// The clipboard.
    pub consumables: Vec<ConsumableInfo>,
    pub consumable_slots: usize,
    /// Distinct themes applied this game.
    pub unique_themes: u32,
    /// Names of what cards may carry.
    pub modifiers: Modifiers,
    pub money: u32,
    /// Present in the `won` phase.
    pub reward: Option<Reward>,
    /// Shop offers (empty outside the shop).
    pub shop: Vec<OfferInfo>,
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
    /// Adds a random theme to the clipboard.
    #[serde(default)]
    pub random_theme: bool,
    /// Adds a random command to the clipboard.
    #[serde(default)]
    pub random_command: bool,
    /// Gives the cards in hand random enhancements, seals and editions.
    #[serde(default)]
    pub enhance_hand: bool,
}

/// Something for sale.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Offer {
    Joker {
        joker: Joker,
        edition: Option<Edition>,
    },
    Consumable(Consumable),
}

pub struct Game {
    seed: u64,
    deck_rng: Rng,
    shop_rng: Rng,
    /// Chances while scoring (Heartbleed, Segfault…).
    score_rng: Rng,
    /// Chances at the end of a blind (Beta Release…).
    round_rng: Rng,
    /// Cards added to the deck (Closed Source, Code Signing).
    card_rng: Rng,
    /// Id of the next card added to the deck.
    next_card_id: u32,
    /// What commands and jokers create (Up Arrow, omarchy-menu, yay -S…).
    command_rng: Rng,
    /// The last command or theme used, but Up Arrow (for Up Arrow).
    last_used: Option<Consumable>,
    /// The last hand type played (Theme Sync).
    last_hand: Option<HandKind>,
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
    consumables: Vec<Consumable>,
    /// Distinct themes applied (r/unixporn).
    themes_used: HashSet<Theme>,
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
            card_rng: Rng::stream(seed, "cards"),
            next_card_id: 52,
            command_rng: Rng::stream(seed, "commands"),
            last_used: None,
            last_hand: None,
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
            consumables: Vec::new(),
            themes_used: HashSet::new(),
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

    /// How many jokers use this ability, Forks copying it included.
    fn abilities(&self, joker: Joker) -> usize {
        (0..self.jokers.len())
            .filter_map(|i| copy_target(&self.jokers, i))
            .filter(|&t| self.jokers[t].joker == joker)
            .count()
    }

    /// Ultrawide jokers and Transparent cards in hand each add one card.
    fn hand_size(&self) -> usize {
        let transparent = self
            .hand
            .iter()
            .filter(|c| c.edition == Some(Edition::Transparent))
            .count();
        HAND_SIZE + self.count(Joker::Ultrawide) + transparent
    }

    /// Changes a card for good: in the deck, and in the draw pile or the hand.
    fn modify_card(&mut self, id: u32, change: impl Fn(&mut Card)) {
        for card in self
            .deck
            .iter_mut()
            .chain(&mut self.draw_pile)
            .chain(&mut self.hand)
            .filter(|c| c.id == id)
        {
            change(card);
        }
    }

    /// A new card for the deck, with an id of its own.
    fn new_card(&mut self) -> Card {
        let rank = 2 + self.card_rng.below(13) as u8;
        let suit = Suit::ALL[self.card_rng.below(4) as usize];
        self.next_card_id += 1;
        Card::new(rank, suit).with_id(self.next_card_id - 1)
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
        for _ in 0..self.abilities(Joker::ClosedSource) {
            let blob = self.new_card().with_enhancement(Enhancement::BinaryBlob);
            self.deck.push(blob);
            self.draw_pile.push(blob);
        }
        self.deck_rng.shuffle(&mut self.draw_pile);
        self.phase = Phase::Playing;
        self.draw();
        for _ in 0..self.abilities(Joker::AliasCollection) {
            let command = self.random_command();
            self.create(command);
        }
        for _ in 0..self.abilities(Joker::CodeSigning) {
            let seal = Seal::ALL[self.card_rng.below(Seal::ALL.len() as u64) as usize];
            let card = self.new_card().with_seal(seal);
            self.deck.push(card);
            self.hand.push(card);
        }
        self.sort_hand();
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
        self.sort_hand();
    }

    /// Hand sorted as at the table: descending rank, then suit; Binary Blobs last.
    fn sort_hand(&mut self) {
        self.hand
            .sort_by_key(|c| (!c.has_rank(), std::cmp::Reverse(c.rank), c.suit as u8));
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
        let money_before = self.money;
        let four_fingers = self.four_fingers();
        let evaluated = evaluate_with(&played, four_fingers);
        self.plays.add(evaluated.kind);
        let mut notes = Vec::new();
        let updates = (0..self.abilities(Joker::AutoUpdate))
            .filter(|_| self.score_rng.chance(4))
            .count() as u32;
        if updates > 0 {
            self.levels.raise(evaluated.kind, updates);
            notes.push(format!(
                "Auto Update: {} leveled up.",
                evaluated.kind.name()
            ));
        }
        self.last_hand = Some(evaluated.kind);
        let mut played = played;
        self.before_scoring(&evaluated, &mut played);
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
            deck: &self.deck,
        };
        let mut breakdown = score(&table, &mut self.score_rng);
        breakdown.notes = notes;
        self.money += breakdown.money;
        self.after_scoring(&mut breakdown, &scoring);
        self.create_from_hand(&evaluated, &scoring, money_before, &mut breakdown.notes);
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

    /// Jokers that change the played cards before they score, left to right:
    /// Enterprise Sales turns faces into Enterprise cards, Garbage Collector
    /// eats enhancements.
    fn before_scoring(&mut self, evaluated: &Evaluated, played: &mut [Card]) {
        for j in 0..self.jokers.len() {
            let joker = self.jokers[j].joker;
            for &i in &evaluated.scoring {
                let card = played[i];
                let change: Option<fn(&mut Card)> = match joker {
                    Joker::EnterpriseSales if card.is_face() => {
                        Some(|c| c.enhancement = Some(Enhancement::Enterprise))
                    }
                    Joker::GarbageCollector if card.enhancement.is_some() => {
                        self.jokers[j].counter += 0.1;
                        Some(|c| c.enhancement = None)
                    }
                    _ => None,
                };
                if let Some(change) = change {
                    change(&mut played[i]);
                    self.modify_card(card.id, change);
                }
            }
        }
    }

    /// After a hand scored: Flaky CI grows, Overclocked cards may have broken
    /// (RMA Department grows), Incremental Build adds chips to the cards.
    fn after_scoring(&mut self, breakdown: &mut Breakdown, scoring: &[Card]) {
        let broken = breakdown.broken.len() as f64;
        for joker in &mut self.jokers {
            match joker.joker {
                Joker::FlakyCi => joker.counter += 0.25 * breakdown.lucky_hits as f64,
                Joker::RmaDepartment => joker.counter += 0.75 * broken,
                _ => {}
            }
        }
        for &id in &breakdown.broken {
            if let Some(card) = self.deck.iter().find(|c| c.id == id) {
                breakdown
                    .notes
                    .push(format!("Overclocked {} broke.", card_label(card)));
            }
            self.deck.retain(|c| c.id != id);
        }
        let chips = 5 * self.abilities(Joker::IncrementalBuild) as u64;
        if chips > 0 {
            for card in scoring {
                self.modify_card(card.id, |c| c.bonus_chips += chips);
            }
        }
    }

    /// Jokers that create commands from the hand played: Vim Golf (an ace and a
    /// Straight), Free Tier ($4 or less), man 8 (1 in 4 per scoring 8).
    fn create_from_hand(
        &mut self,
        evaluated: &Evaluated,
        scoring: &[Card],
        money: u32,
        notes: &mut Vec<String>,
    ) {
        let ace = scoring.iter().any(|c| c.has_rank() && c.rank == ACE);
        if ace && evaluated.contains(HandKind::Straight) {
            self.create_commands(self.abilities(Joker::VimGolf), "Vim Golf", notes);
        }
        if money <= 4 {
            self.create_commands(self.abilities(Joker::FreeTier), "Free Tier", notes);
        }
        let eights = scoring
            .iter()
            .filter(|c| c.has_rank() && c.rank == 8)
            .count();
        let rolls = eights * self.abilities(Joker::Man8);
        let hits = (0..rolls).filter(|_| self.command_rng.chance(4)).count();
        self.create_commands(hits, "man 8", notes);
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
        let syncs = self
            .hand
            .iter()
            .filter(|c| c.seal == Some(Seal::ThemeSync))
            .count();
        if let Some(kind) = self.last_hand {
            let theme = Theme::ALL[kind as usize];
            for _ in 0..syncs {
                if self.consumables.len() < CONSUMABLE_SLOTS {
                    self.consumables.push(Consumable::Theme(theme));
                    reward.notes.push(format!(
                        "Theme Sync: {} added to the clipboard.",
                        theme.name()
                    ));
                }
            }
        }
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
        let discarded = self.take(indices)?;
        let histories = discarded
            .iter()
            .filter(|c| c.seal == Some(Seal::ShellHistory))
            .count();
        for _ in 0..histories {
            let command = self.random_command();
            self.create(command);
        }
        if self.discards_used == 0 {
            let reloads = self.abilities(Joker::ConfigReload) as u32;
            if reloads > 0 {
                let kind = evaluate_with(&discarded, self.four_fingers()).kind;
                self.levels.raise(kind, reloads);
            }
        }
        self.discards_left -= 1;
        self.discards_used += 1;
        for joker in &mut self.jokers {
            joker.on_discard();
        }
        self.draw();
        Ok(())
    }

    fn reward(&self) -> Reward {
        let cron_jobs = 1 + self.abilities(Joker::CronJob) as u32;
        let enterprise = self
            .hand
            .iter()
            .filter(|c| c.is(Enhancement::Enterprise))
            .count() as u32;
        let cards = 3 * enterprise * cron_jobs;
        let blind = self.blind.reward();
        let hands = self.hands_left;
        let interest = (self.money / 5).min(MAX_INTEREST);
        let jokers = (0..self.jokers.len())
            .filter_map(|i| copy_target(&self.jokers, i))
            .map(|t| {
                self.jokers[t].joker.blind_money(
                    self.discards_left,
                    self.discards_used,
                    self.themes_used.len() as u32,
                )
            })
            .sum();
        Reward {
            blind,
            hands,
            interest,
            jokers,
            cards,
            total: blind + hands + interest + jokers + cards,
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
        let offered = self
            .offers
            .iter()
            .any(|o| matches!(o, Offer::Joker { joker: j, .. } if *j == joker));
        let lifecycle = match joker {
            Joker::BetaRelease => !self.beta_broken,
            Joker::StableRelease => self.beta_broken,
            _ => true,
        };
        !owned && !offered && lifecycle && joker.rarity() != Rarity::Legendary
    }

    /// Draws new offers: for each slot, a joker (weight 20), a theme (4) or a
    /// command (4).
    fn roll_offers(&mut self) {
        self.offers.clear();
        let mut rng = self.shop_rng.clone();
        for _ in 0..SHOP_OFFERS {
            let roll = rng.below(28);
            let offer = if roll < 20 {
                self.roll_joker(&mut rng)
            } else if roll < 24 {
                self.roll_theme(&mut rng)
            } else {
                let command = Command::ALL[rng.below(Command::ALL.len() as u64) as usize];
                Some(Offer::Consumable(Consumable::Command(command)))
            };
            match offer {
                Some(offer) => self.offers.push(offer),
                None => break,
            }
        }
        self.shop_rng = rng;
    }

    /// A theme whose hand type is known (secret hands once played), not on
    /// offer yet.
    fn roll_theme(&self, rng: &mut Rng) -> Option<Offer> {
        let pool: Vec<Theme> = Theme::ALL
            .into_iter()
            .filter(|t| !t.hand().secret() || self.plays.get(t.hand()) > 0)
            .filter(|&t| {
                !self
                    .offers
                    .contains(&Offer::Consumable(Consumable::Theme(t)))
            })
            .collect();
        let theme = *pool.get(rng.below(pool.len() as u64) as usize)?;
        Some(Offer::Consumable(Consumable::Theme(theme)))
    }

    /// A rarity (70% core, 25% extra, 5% AUR), a joker of that rarity not owned
    /// yet, and sometimes an edition.
    fn roll_joker(&self, rng: &mut Rng) -> Option<Offer> {
        let roll = rng.unit();
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
            return None;
        }
        let joker = pool[rng.below(pool.len() as u64) as usize];
        let roll = rng.unit();
        let mut odds = 0.0;
        let edition = Edition::ALL.into_iter().rev().find(|e| {
            odds += e.odds();
            roll < odds
        });
        Some(Offer::Joker { joker, edition })
    }

    fn consumable_price(&self, consumable: Consumable) -> u32 {
        match consumable {
            Consumable::Theme(_) if self.count(Joker::FreeAsInBeer) > 0 => 0,
            _ => consumable.price(),
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
        let index = offer;
        let offer = *self
            .offers
            .get(index)
            .ok_or_else(|| GameError::BadSelection(format!("no offer #{index}")))?;
        match offer {
            Offer::Joker { joker, edition } => {
                if !self.has_room(edition) {
                    return Err(GameError::JokerSlotsFull);
                }
                let owned = OwnedJoker::new(joker, edition);
                self.spend(owned.price())?;
                self.jokers.push(owned);
            }
            Offer::Consumable(consumable) => {
                if self.consumables.len() >= CONSUMABLE_SLOTS {
                    return Err(GameError::ClipboardFull);
                }
                self.spend(self.consumable_price(consumable))?;
                self.consumables.push(consumable);
            }
        }
        self.offers.remove(index);
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

    fn consumable(&self, index: usize) -> Result<Consumable, GameError> {
        self.consumables
            .get(index)
            .copied()
            .ok_or_else(|| GameError::BadSelection(format!("no consumable #{index}")))
    }

    /// Adds a consumable to the clipboard if there is room.
    fn create(&mut self, consumable: Consumable) -> bool {
        let room = self.consumables.len() < CONSUMABLE_SLOTS;
        if room {
            self.consumables.push(consumable);
        }
        room
    }

    fn random_command(&mut self) -> Consumable {
        Consumable::Command(
            Command::ALL[self.command_rng.below(Command::ALL.len() as u64) as usize],
        )
    }

    /// Creates a random command for each of `times`, while there is room; says so.
    fn create_commands(&mut self, times: usize, source: &str, notes: &mut Vec<String>) {
        for _ in 0..times {
            let command = self.random_command();
            if self.create(command) {
                notes.push(format!(
                    "{source}: {} added to the clipboard.",
                    command.name()
                ));
            }
        }
    }

    /// Uses a consumable from the clipboard, at any time but after a loss;
    /// commands that change cards need them selected in the hand (`cards`).
    pub fn use_consumable(&mut self, index: usize, cards: &[usize]) -> Result<(), GameError> {
        if self.phase == Phase::Lost {
            return Err(GameError::NotPlaying);
        }
        let consumable = self.consumable(index)?;
        let targets = match consumable.targets() {
            Some((min, max)) => {
                let sorted = self.validate(cards)?;
                if !(min..=max).contains(&sorted.len()) {
                    return Err(GameError::BadSelection(format!(
                        "{} needs {min} to {max} cards",
                        consumable.name()
                    )));
                }
                sorted
            }
            None => Vec::new(),
        };
        self.check_usable(consumable)?;
        self.consumables.remove(index);
        match consumable {
            Consumable::Theme(theme) => {
                self.levels.raise(theme.hand(), 1);
                self.themes_used.insert(theme);
                for joker in &mut self.jokers {
                    joker.on_theme_used();
                }
            }
            Consumable::Command(command) => {
                self.run_command(command, &targets);
                for joker in &mut self.jokers {
                    joker.on_command_used();
                }
            }
        }
        if consumable != Consumable::Command(Command::UpArrow) {
            self.last_used = Some(consumable);
        }
        Ok(())
    }

    /// Refuses a consumable that could do nothing.
    fn check_usable(&self, consumable: Consumable) -> Result<(), GameError> {
        let refuse = |why: &str| Err(GameError::BadSelection(why.to_string()));
        match consumable {
            Consumable::Command(Command::UpArrow) if self.last_used.is_none() => {
                refuse("no command or theme used yet")
            }
            Consumable::Command(Command::HyprctlReload)
                if self.jokers.iter().all(|j| j.edition.is_some()) =>
            {
                refuse("no joker without an edition")
            }
            Consumable::Command(Command::YayS) if !self.has_room(None) => {
                Err(GameError::JokerSlotsFull)
            }
            _ => Ok(()),
        }
    }

    fn run_command(&mut self, command: Command, targets: &[usize]) {
        let ids: Vec<u32> = targets.iter().map(|&i| self.hand[i].id).collect();
        let enhance = |enhancement| move |card: &mut Card| card.enhancement = Some(enhancement);
        let suit = |suit| move |card: &mut Card| card.suit = suit;
        let change: Option<Box<dyn Fn(&mut Card)>> = match command {
            Command::NpmTest => Some(Box::new(enhance(Enhancement::FlakyTest))),
            Command::MakeJ => Some(Box::new(enhance(Enhancement::Multithreaded))),
            Command::CargoRelease => Some(Box::new(enhance(Enhancement::Optimized))),
            Command::LnS => Some(Box::new(enhance(Enhancement::Symlink))),
            Command::SystemctlEnable => Some(Box::new(enhance(Enhancement::Daemon))),
            Command::Cpupower => Some(Box::new(enhance(Enhancement::Overclocked))),
            Command::AcceptEula => Some(Box::new(enhance(Enhancement::Enterprise))),
            Command::Strip => Some(Box::new(enhance(Enhancement::BinaryBlob))),
            Command::MvDiamonds => Some(Box::new(suit(Suit::Diamonds))),
            Command::MvClubs => Some(Box::new(suit(Suit::Clubs))),
            Command::MvHearts => Some(Box::new(suit(Suit::Hearts))),
            Command::MvSpades => Some(Box::new(suit(Suit::Spades))),
            // The king becomes an ace, the ace a two.
            Command::NpmVersionPatch => Some(Box::new(|card: &mut Card| {
                card.rank = if card.rank == ACE { 2 } else { card.rank + 1 }
            })),
            _ => None,
        };
        if let Some(change) = change {
            for &id in &ids {
                self.modify_card(id, &change);
            }
            self.sort_hand();
            return;
        }
        match command {
            Command::UpArrow => {
                if let Some(last) = self.last_used {
                    self.create(last);
                }
            }
            Command::ThemeInstall => {
                for _ in 0..2 {
                    let mut rng = self.command_rng.clone();
                    let theme = self.roll_theme(&mut rng);
                    self.command_rng = rng;
                    if let Some(Offer::Consumable(theme)) = theme {
                        self.create(theme);
                    }
                }
            }
            Command::OmarchyMenu => {
                for _ in 0..2 {
                    let command = self.random_command();
                    self.create(command);
                }
            }
            Command::Rsync => self.money += self.money.min(20),
            Command::HyprctlReload => {
                let bare: Vec<usize> = (0..self.jokers.len())
                    .filter(|&j| self.jokers[j].edition.is_none())
                    .collect();
                if self.command_rng.chance(4) && !bare.is_empty() {
                    let j = bare[self.command_rng.below(bare.len() as u64) as usize];
                    let edition = Edition::ALL[self.command_rng.below(3) as usize];
                    self.jokers[j].edition = Some(edition);
                }
            }
            Command::RmRf => {
                self.deck.retain(|c| !ids.contains(&c.id));
                self.hand.retain(|c| !ids.contains(&c.id));
            }
            Command::CpF => {
                let (left, right) = (self.hand[targets[0]], self.hand[targets[1]]);
                self.modify_card(left.id, |card| *card = right.with_id(left.id));
                self.sort_hand();
            }
            Command::DuSh => {
                let value: u32 = self.jokers.iter().map(OwnedJoker::sell_value).sum();
                self.money += value.min(50);
            }
            Command::YayS => {
                let mut rng = self.command_rng.clone();
                if let Some(Offer::Joker { joker, .. }) = self.roll_joker(&mut rng) {
                    self.jokers.push(OwnedJoker::new(joker, None));
                }
                self.command_rng = rng;
            }
            _ => {}
        }
    }

    pub fn sell_consumable(&mut self, index: usize) -> Result<(), GameError> {
        if self.phase == Phase::Lost {
            return Err(GameError::NotPlaying);
        }
        let consumable = self.consumable(index)?;
        self.consumables.remove(index);
        self.money += consumable.sell_value();
        for joker in &mut self.jokers {
            joker.on_card_sold();
        }
        Ok(())
    }

    /// Each joker's badge if the hand type were played now (Muscle Memory
    /// depends on it).
    pub fn bonuses_for(&self, kind: HandKind) -> Vec<Option<Bonus>> {
        let plays = self.plays.get(kind) + 1;
        self.jokers
            .iter()
            .map(|j| j.bonus(Some(plays), self.themes_used.len() as u32, &self.deck))
            .collect()
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
        if cheat.enhance_hand {
            let ids: Vec<u32> = self.hand.iter().map(|c| c.id).collect();
            for id in ids {
                let roll = self.card_rng.below(8 * 5 * 4);
                self.modify_card(id, |card| {
                    card.enhancement = Some(Enhancement::ALL[roll as usize % 8]);
                    card.seal = Seal::ALL.get(roll as usize / 8 % 5).copied();
                    card.edition = Edition::ALL[..3].get(roll as usize / 40).copied();
                });
            }
        }
        if cheat.random_command {
            let command = self.random_command();
            if !self.create(command) {
                return Err(GameError::ClipboardFull);
            }
        }
        if cheat.random_theme {
            if self.consumables.len() >= CONSUMABLE_SLOTS {
                return Err(GameError::ClipboardFull);
            }
            let mut rng = Rng::stream(self.seed ^ self.consumables.len() as u64, "cheat-theme");
            let theme = Theme::ALL[rng.below(9) as usize];
            self.consumables.push(Consumable::Theme(theme));
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
            None
        };
        JokerInfo::new(
            owned,
            current,
            owned.bonus(None, self.themes_used.len() as u32, &self.deck),
        )
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
            consumables: self
                .consumables
                .iter()
                .map(|&c| ConsumableInfo::new(c, c.price()))
                .collect(),
            consumable_slots: CONSUMABLE_SLOTS,
            unique_themes: self.themes_used.len() as u32,
            modifiers: Modifiers::catalog(),
            shop: self
                .offers
                .iter()
                .map(|&o| match o {
                    Offer::Joker { joker, edition } => OfferInfo::Joker(JokerInfo::new(
                        &OwnedJoker::new(joker, edition),
                        None,
                        None,
                    )),
                    Offer::Consumable(c) => {
                        OfferInfo::Consumable(ConsumableInfo::new(c, self.consumable_price(c)))
                    }
                })
                .collect(),
            reroll_cost: self.reroll_cost,
        }
    }
}

/// "K♠", for messages.
fn card_label(card: &Card) -> String {
    let rank = match card.rank {
        11 => "J".to_string(),
        12 => "Q".to_string(),
        13 => "K".to_string(),
        14 => "A".to_string(),
        r => r.to_string(),
    };
    let suit = match card.suit {
        Suit::Hearts => "♥",
        Suit::Diamonds => "♦",
        Suit::Clubs => "♣",
        Suit::Spades => "♠",
    };
    format!("{rank}{suit}")
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
                cards: 0,
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
        game.offers = vec![
            Offer::Joker {
                joker: Joker::Hotwire,
                edition: None,
            },
            Offer::Consumable(Consumable::Theme(Theme::Nord)),
        ];
        let owned = OwnedJoker::new(Joker::Hotwire, None);
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
        game.offers = vec![Offer::Joker {
            joker: Joker::Fork,
            edition: None,
        }];
        assert_eq!(game.buy(0), Err(GameError::JokerSlotsFull));
        // A transparent joker brings its own slot.
        game.offers = vec![Offer::Joker {
            joker: Joker::Fork,
            edition: Some(Edition::Transparent),
        }];
        game.buy(0).unwrap();
        assert_eq!(game.state().max_jokers, MAX_JOKERS + 1);
        game.roll_offers();
        assert!(
            game.offers
                .iter()
                .filter_map(offered_joker)
                .all(|o| game.jokers.iter().all(|j| j.joker != o))
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
            game.state().jokers[0].bonus.as_ref().unwrap().text,
            "×1.25 mult"
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
            for joker in game.offers.iter().filter_map(offered_joker) {
                *counts.entry(joker.rarity()).or_insert(0) += 1;
            }
        }
        let total: usize = counts.values().sum();
        // Themes and commands take about 2 slots in 7.
        assert!(
            (total as f64 / 4000.0 - 20.0 / 28.0).abs() < 0.03,
            "{total}"
        );
        let share = |r| counts.get(&r).copied().unwrap_or(0) as f64 / total as f64;
        assert!((share(Rarity::Core) - 0.70).abs() < 0.03, "{counts:?}");
        assert!((share(Rarity::Aur) - 0.05).abs() < 0.02, "{counts:?}");
        assert_eq!(share(Rarity::Legendary), 0.0);
    }

    fn offered_joker(offer: &Offer) -> Option<Joker> {
        match *offer {
            Offer::Joker { joker, .. } => Some(joker),
            Offer::Consumable(_) => None,
        }
    }

    #[test]
    fn themes_level_up_hands_and_feed_jokers() {
        let mut game = won_game(21, vec![Joker::ThemeCollector, Joker::Unixporn]);
        game.cash_out().unwrap();
        game.money = 20;
        let rose_pine = Offer::Consumable(Consumable::Theme(Theme::RosePine));
        game.offers = vec![rose_pine, rose_pine, rose_pine];
        game.buy(0).unwrap();
        game.buy(0).unwrap();
        assert_eq!(game.buy(0), Err(GameError::ClipboardFull));
        assert_eq!(game.money, 14);
        game.use_consumable(0, &[]).unwrap();
        game.use_consumable(0, &[]).unwrap();
        let s = game.state();
        assert_eq!(s.hands[1].level, 3);
        assert_eq!(s.jokers[0].bonus.as_ref().unwrap().text, "×1.2 mult");
        assert_eq!(s.jokers[1].bonus.as_ref().unwrap().text, "+$1 / blind");
        assert!(game.use_consumable(0, &[]).is_err());
    }

    #[test]
    fn free_as_in_beer_makes_themes_free() {
        let mut game = won_game(22, vec![Joker::FreeAsInBeer]);
        game.cash_out().unwrap();
        game.money = 0;
        game.offers = vec![Offer::Consumable(Consumable::Theme(Theme::Nord))];
        game.buy(0).unwrap();
        game.sell_consumable(0).unwrap();
        assert_eq!(game.money, 1);
    }

    #[test]
    fn secret_themes_wait_for_their_hand() {
        let mut game = Game::new(23, vec![]);
        for seed in 0..300 {
            game.offers.clear();
            let roll = game.roll_theme(&mut Rng::new(seed));
            let Some(Offer::Consumable(Consumable::Theme(theme))) = roll else {
                panic!("no theme");
            };
            assert!(!theme.hand().secret());
        }
    }

    #[test]
    fn config_reload_levels_up_the_first_discard() {
        let mut game = playing(24, vec![Joker::ConfigReload]);
        game.hand = cards("KH KD 9S 7C 5H 4D 3S 2C");
        game.discard(&[0, 1]).unwrap();
        game.hand = cards("QH QD 9S 7C 5H 4D 3S 2C");
        game.discard(&[0, 1]).unwrap();
        assert_eq!(game.levels.level(HandKind::Pair), 2);
    }

    #[test]
    fn auto_update_sometimes_levels_up_the_hand() {
        let mut game = playing(25, vec![Joker::AutoUpdate]);
        // A huge target, so that the blind never ends.
        game.ante = 30;
        game.hands_left = 100;
        for _ in 0..40 {
            game.hand = cards("2H 3D 5S 7C 9H 2D 3S 5C");
            game.play(&[0]).unwrap();
        }
        // 1 in 4 over 40 hands: about 10 levels.
        let level = game.levels.level(HandKind::HighCard);
        assert!((4..=20).contains(&level), "{level}");
        assert_eq!(game.bonuses_for(HandKind::Pair), [None]);
    }

    #[test]
    fn muscle_memory_badge_follows_the_selected_hand() {
        let mut game = playing(26, vec![Joker::MuscleMemory]);
        assert_eq!(game.state().jokers[0].bonus, None);
        game.hand = cards("KH KD 9S 7C 5H 4D 3S 2C");
        game.play(&[0, 1]).unwrap();
        assert_eq!(
            game.bonuses_for(HandKind::Pair)[0].as_ref().unwrap().text,
            "+2 mult"
        );
        assert_eq!(
            game.bonuses_for(HandKind::Flush)[0].as_ref().unwrap().text,
            "+1 mult"
        );
    }

    #[test]
    fn closed_source_and_code_signing_add_cards() {
        let game = playing(30, vec![Joker::ClosedSource, Joker::CodeSigning]);
        let s = game.state();
        assert_eq!(s.deck_size, 54);
        assert_eq!(s.hand.len(), HAND_SIZE + 1);
        assert!(s.hand.iter().any(|c| c.id == 53 && c.seal.is_some()));
        assert!(
            game.deck
                .iter()
                .any(|c| c.id == 52 && c.is(Enhancement::BinaryBlob))
        );
    }

    #[test]
    fn enterprise_sales_then_garbage_collector() {
        let mut game = playing(31, vec![Joker::EnterpriseSales, Joker::GarbageCollector]);
        game.hand = cards("KH KD 9S 7C 5H 4D 3S 2C");
        game.deck.extend(game.hand.clone());
        game.play(&[0, 1]).unwrap();
        // The kings became Enterprise cards, then lost it: ×1.2 for the collector.
        assert!((game.jokers[1].counter - 1.2).abs() < 1e-9);
        assert!(
            game.deck
                .iter()
                .filter(|c| c.id == 100 || c.id == 101)
                .all(|c| c.enhancement.is_none())
        );
    }

    #[test]
    fn enterprise_cards_pay_when_held_and_theme_sync_creates_themes() {
        let mut game = playing(32, vec![Joker::CronJob]);
        game.hand = cards("AS KS QS JS 10S 2H 3D 4C");
        game.hand[5] = game.hand[5].with_enhancement(Enhancement::Enterprise);
        game.hand[6] = game.hand[6].with_seal(Seal::ThemeSync);
        game.play(&[0, 1, 2, 3, 4]).unwrap();
        let s = game.state();
        // $3, twice with Cron Job.
        assert_eq!(s.reward.as_ref().unwrap().cards, 6);
        assert_eq!(s.consumables[0].name, "Tokyo Night");
    }

    #[test]
    fn incremental_build_adds_chips_for_good() {
        let mut game = playing(33, vec![Joker::IncrementalBuild]);
        let king = game.hand[0];
        game.play(&[0]).unwrap();
        let card = game.deck.iter().find(|c| c.id == king.id).unwrap();
        assert_eq!(card.bonus_chips, 5);
    }

    #[test]
    fn transparent_cards_enlarge_the_hand() {
        let mut game = playing(34, vec![]);
        let id = game.hand[0].id;
        game.modify_card(id, |c| c.edition = Some(Edition::Transparent));
        game.discard(&[7]).unwrap();
        assert_eq!(game.hand.len(), HAND_SIZE + 1);
    }

    #[test]
    fn broken_cards_leave_the_deck() {
        let mut game = playing(35, vec![Joker::RmaDepartment]);
        game.ante = 30;
        game.hands_left = 100;
        let mut broke = 0;
        for _ in 0..30 {
            let id = game.hand[0].id;
            game.modify_card(id, |c| c.enhancement = Some(Enhancement::Overclocked));
            let b = game.play(&[0]).unwrap();
            broke += b.broken.len();
            assert_eq!(game.deck.iter().any(|c| c.id == id), b.broken.is_empty());
            if game.hand.is_empty() {
                break;
            }
        }
        assert!(broke > 0);
        assert_eq!(game.jokers[0].counter, 1.0 + 0.75 * broke as f64);
    }

    /// A game in its first blind with this command in the clipboard.
    fn with_command(seed: u64, command: Command, jokers: Vec<Joker>) -> Game {
        let mut game = playing(seed, jokers);
        game.consumables = vec![Consumable::Command(command)];
        game.hand = cards("KH KD 9S 7C 5H 4D 3S 2C");
        game.deck = game.hand.clone();
        game
    }

    #[test]
    fn commands_change_the_selected_cards() {
        let mut game = with_command(40, Command::MakeJ, vec![Joker::ShellFu]);
        assert!(matches!(
            game.use_consumable(0, &[]),
            Err(GameError::BadSelection(_))
        ));
        assert!(matches!(
            game.use_consumable(0, &[0, 1, 2]),
            Err(GameError::BadSelection(_))
        ));
        game.use_consumable(0, &[0, 2]).unwrap();
        assert!(
            game.hand[0].is(Enhancement::Multithreaded)
                && game.hand[2].is(Enhancement::Multithreaded)
        );
        assert!(
            game.deck
                .iter()
                .filter(|c| c.is(Enhancement::Multithreaded))
                .count()
                == 2
        );
        assert_eq!(
            game.state().jokers[0].bonus.as_ref().unwrap().text,
            "+1 mult"
        );

        let mut game = with_command(41, Command::MvSpades, vec![]);
        game.use_consumable(0, &[0, 1, 4]).unwrap();
        assert!(game.hand.iter().filter(|c| c.suit == Suit::Spades).count() == 5);

        let mut game = with_command(42, Command::NpmVersionPatch, vec![]);
        game.use_consumable(0, &[0]).unwrap();
        assert_eq!(game.hand[0].rank, ACE);
    }

    #[test]
    fn rm_rf_and_cp_f() {
        let mut game = with_command(43, Command::RmRf, vec![]);
        game.use_consumable(0, &[0, 1]).unwrap();
        assert_eq!((game.hand.len(), game.deck.len()), (6, 6));

        let mut game = with_command(44, Command::CpF, vec![]);
        game.hand[7] = game.hand[7].with_seal(Seal::Retry);
        game.deck = game.hand.clone();
        game.use_consumable(0, &[2, 7]).unwrap();
        let copy = game.hand.iter().find(|c| c.id == 102).unwrap();
        assert_eq!(
            (copy.rank, copy.suit, copy.seal),
            (2, Suit::Clubs, Some(Seal::Retry))
        );
    }

    #[test]
    fn money_commands() {
        let mut game = with_command(45, Command::Rsync, vec![]);
        game.money = 15;
        game.use_consumable(0, &[]).unwrap();
        assert_eq!(game.money, 30);
        game.consumables = vec![Consumable::Command(Command::Rsync)];
        game.use_consumable(0, &[]).unwrap();
        assert_eq!(game.money, 50);

        let mut game = with_command(46, Command::DuSh, vec![Joker::Fork, Joker::Hotwire]);
        game.money = 0;
        game.use_consumable(0, &[]).unwrap();
        assert_eq!(game.money, 5 + 4);
    }

    #[test]
    fn up_arrow_recalls_the_last_consumable() {
        let mut game = with_command(47, Command::UpArrow, vec![]);
        assert!(game.use_consumable(0, &[]).is_err());
        game.consumables.insert(0, Consumable::Theme(Theme::Nord));
        game.use_consumable(0, &[]).unwrap();
        game.use_consumable(0, &[]).unwrap();
        assert_eq!(game.consumables, [Consumable::Theme(Theme::Nord)]);
    }

    #[test]
    fn creating_commands() {
        let mut game = with_command(48, Command::OmarchyMenu, vec![]);
        game.use_consumable(0, &[]).unwrap();
        assert_eq!(game.consumables.len(), 2);

        let mut game = with_command(49, Command::YayS, vec![]);
        game.use_consumable(0, &[]).unwrap();
        assert_eq!(game.jokers.len(), 1);

        let mut game = playing(50, vec![Joker::AliasCollection]);
        game.consumables.clear();
        game.cheat(Cheat {
            blind: Some(BlindKind::Big),
            ..Cheat::default()
        })
        .unwrap();
        game.select_blind().unwrap();
        assert_eq!(game.consumables.len(), 1);
    }

    #[test]
    fn shell_history_creates_a_command_when_discarded() {
        let mut game = playing(51, vec![]);
        let id = game.hand[0].id;
        game.modify_card(id, |c| c.seal = Some(Seal::ShellHistory));
        game.discard(&[0]).unwrap();
        assert!(matches!(game.consumables[..], [Consumable::Command(_)]));
    }

    #[test]
    fn vim_golf_and_free_tier() {
        let mut game = playing(52, vec![Joker::VimGolf, Joker::FreeTier]);
        game.money = 4;
        game.hand = cards("AH KD QS JC 10H 4D 3S 2C");
        let b = game.play(&[0, 1, 2, 3, 4]).unwrap();
        assert_eq!(game.consumables.len(), 2);
        assert_eq!(b.notes.len(), 2);
    }

    #[test]
    fn shop_actions_need_the_shop() {
        let mut game = playing(9, vec![Joker::Dotfiles]);
        assert_eq!(game.buy(0), Err(GameError::NotInShop));
        assert_eq!(game.sell(0), Err(GameError::NotInShop));
        assert_eq!(game.reroll(), Err(GameError::NotInShop));
    }
}
