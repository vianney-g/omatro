//! How a game unfolds: antes of three blinds, hands and discards, the shop.

use crate::blind::{self, BlindKind, WINNING_ANTE};
use crate::boss::Boss;
use crate::card::{ACE, Card, Suit, standard_deck};
use crate::consumable::{CONSUMABLE_SLOTS, Command, Consumable, Oligarchy, Theme};
use crate::hand::{Evaluated, HandCounts, HandKind, HandLevels, evaluate_with};
use crate::joker::{Bonus, Joker, OwnedJoker, Rarity};
use crate::modifier::{Edition, Enhancement, Seal};
use crate::pack::{Pack, PackKind, SHOP_PACKS};
use crate::rng::Rng;
use crate::score::{Breakdown, Table, copy_target, score};
use crate::tag::Tag;
use crate::voucher::{VOUCHER_PRICE, Voucher};
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
    /// A pack bought in the shop is open: pick its cards, or skip.
    Pack,
    Lost,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    NotSelecting,
    NotPlaying,
    NotWon,
    NotInShop,
    NotInPack,
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
            GameError::NotInPack => write!(f, "no pack is open"),
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
    /// A Transparent consumable takes no clipboard slot.
    pub transparent: bool,
    /// Works on the cards in hand: only during a blind, or in a pack with a hand.
    pub needs_hand: bool,
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
            transparent: false,
            needs_hand: consumable.needs_hand(),
        }
    }
}

/// A shop offer: a joker or a consumable, told apart by `kind`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum OfferInfo {
    Joker(JokerInfo),
    Consumable(ConsumableInfo),
    Card(CardOffer),
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

/// A pull request, merged or for sale.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct VoucherInfo {
    pub id: Voucher,
    pub name: &'static str,
    pub description: &'static str,
    pub quip: &'static str,
    pub price: u32,
}

impl From<Voucher> for VoucherInfo {
    fn from(voucher: Voucher) -> Self {
        VoucherInfo {
            id: voucher,
            name: voucher.name(),
            description: voucher.description(),
            quip: voucher.quip(),
            price: VOUCHER_PRICE,
        }
    }
}

/// A playing card for sale in the shop.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CardOffer {
    /// Always "card".
    pub kind: &'static str,
    pub card: Card,
    pub price: u32,
}

/// The boss of the ante.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BossInfo {
    pub id: Boss,
    pub name: &'static str,
    pub description: &'static str,
    pub quip: &'static str,
}

/// One of the three blinds of the current ante.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct BlindInfo {
    pub kind: BlindKind,
    pub name: &'static str,
    /// The boss's effect (none for the small and big blinds).
    pub description: &'static str,
    pub quip: &'static str,
    pub target: f64,
    pub reward: u32,
    pub status: BlindStatus,
    /// The badge skipping it gives (small and big blinds).
    pub tag: Option<TagInfo>,
}

/// A badge: "label: value", like a README badge.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TagInfo {
    pub id: Tag,
    pub label: &'static str,
    pub value: &'static str,
    pub name: String,
    pub description: &'static str,
    pub quip: &'static str,
}

impl From<Tag> for TagInfo {
    fn from(tag: Tag) -> Self {
        TagInfo {
            id: tag,
            label: tag.label(),
            value: tag.value(),
            name: tag.name(),
            description: tag.description(),
            quip: tag.quip(),
        }
    }
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
    /// funding: series A.
    pub badges: u32,
    pub total: u32,
    /// What happened to the jokers ("Beta Release broke.").
    pub notes: Vec<String>,
}

/// A pack for sale.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct PackInfo {
    #[serde(flatten)]
    pub pack: Pack,
    pub name: String,
    pub description: String,
    pub quip: &'static str,
    pub price: u32,
}

/// A playing card offered by a base-devel pack.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct CardChoice {
    /// Always "card".
    pub kind: &'static str,
    pub card: Card,
}

/// A card of an open pack: a consumable, a joker or a playing card, told
/// apart by `kind`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ChoiceInfo {
    Consumable(ConsumableInfo),
    Joker(JokerInfo),
    Card(CardChoice),
}

/// The pack being opened.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OpenedInfo {
    pub name: String,
    pub description: String,
    pub quip: &'static str,
    pub picks_left: usize,
    /// coreutils: commands apply to a hand drawn from the deck.
    pub with_hand: bool,
    pub choices: Vec<ChoiceInfo>,
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
    /// For each card in hand: dealt face down by the boss.
    pub face_down: Vec<bool>,
    /// For each card in hand: kept from scoring by the boss.
    pub debuffed: Vec<bool>,
    /// The Clippy: the hand card always selected.
    pub forced: Option<usize>,
    /// The boss of the ante, and whether its effect applies now.
    pub boss: BossInfo,
    pub boss_active: bool,
    /// The Kernel Panic: jokers are face down.
    pub jokers_hidden: bool,
    /// The Windows Update: the joker that does nothing this hand.
    pub disabled_joker: Option<usize>,
    /// Pull requests merged, and the one for sale this ante (in the shop).
    pub vouchers: Vec<VoucherInfo>,
    pub vouchers_for_sale: Vec<VoucherInfo>,
    /// Badges won and waiting for their moment.
    pub tags: Vec<TagInfo>,
    /// Change Request: what rerolling the boss costs, when allowed now.
    pub boss_reroll_cost: Option<u32>,
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
    /// Packs for sale in the shop.
    pub packs: Vec<PackInfo>,
    /// The pack being opened (`pack` phase).
    pub opened: Option<OpenedInfo>,
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
    /// Adds a random oligarchy card to the clipboard.
    #[serde(default)]
    pub random_oligarchy: bool,
    /// Sets this ante's boss.
    pub boss: Option<Boss>,
    /// Sets this ante's boss to the next one in the list.
    #[serde(default)]
    pub next_boss: bool,
    /// Merges pull requests.
    #[serde(default)]
    pub vouchers: Vec<Voucher>,
    /// Wins badges.
    #[serde(default)]
    pub tags: Vec<Tag>,
    /// Opens a random pack (in the shop).
    #[serde(default)]
    pub random_pack: bool,
    /// Gives the cards in hand random enhancements, seals and editions.
    #[serde(default)]
    pub enhance_hand: bool,
}

/// Why cards are drawn (some bosses deal them face down).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Draw {
    Deal,
    AfterHand,
    AfterDiscard,
}

/// A consumable in the clipboard; a Transparent one takes no slot (Copy-on-Write).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Held {
    pub consumable: Consumable,
    pub transparent: bool,
}

impl From<Consumable> for Held {
    fn from(consumable: Consumable) -> Self {
        Held {
            consumable,
            transparent: false,
        }
    }
}

/// A random oligarchy card: `card su` and `card pacman -Syu` are rare (0.3% each).
fn roll_oligarchy(rng: &mut Rng) -> Oligarchy {
    let roll = rng.unit();
    if roll < 0.003 {
        Oligarchy::AspiringBillionaire
    } else if roll < 0.006 {
        Oligarchy::PraiseTheOligarchy
    } else {
        let common: Vec<Oligarchy> = Oligarchy::ALL
            .into_iter()
            .filter(|s| {
                !matches!(
                    s,
                    Oligarchy::AspiringBillionaire | Oligarchy::PraiseTheOligarchy
                )
            })
            .collect();
        common[rng.below(common.len() as u64) as usize]
    }
}

/// A card of an open pack.
#[derive(Debug, Clone, Copy, PartialEq)]
enum PackItem {
    Consumable(Consumable),
    Joker {
        joker: Joker,
        edition: Option<Edition>,
    },
    /// A new card for the deck; it gets its id once picked.
    Card(Card),
}

#[derive(Debug, Clone, PartialEq)]
struct OpenedPack {
    pack: Pack,
    items: Vec<PackItem>,
    picks_left: usize,
    /// The shop, or the blind choice for a pack won by a badge.
    return_to: Phase,
}

/// Something for sale.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Offer {
    Joker {
        joker: Joker,
        edition: Option<Edition>,
    },
    Consumable(Consumable),
    /// A playing card for the deck (Card Marketplace); it gets its id once bought.
    Card(Card),
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
    /// Packs for sale in this shop.
    packs: Vec<Pack>,
    /// What packs hold, and which packs a shop sells.
    pack_rng: Rng,
    opened: Option<OpenedPack>,
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
    consumables: Vec<Held>,
    /// Distinct themes applied (r/unixporn).
    themes_used: HashSet<Theme>,
    /// Hand size lost to Consolidation and Offshore Account.
    hand_size_penalty: usize,
    /// This ante's boss, and the bosses already met (no repeat until all were).
    boss: Boss,
    bosses_seen: Vec<Boss>,
    /// Which boss each ante gets, and its chances (face-down cards…).
    boss_rng: Rng,
    /// Rollback was sold during the boss blind.
    boss_disabled: bool,
    /// Ids of the cards played this ante (The Stale Cache).
    played_this_ante: HashSet<u32>,
    /// Hand types played this blind (The Code Reviewer, The Style Guide).
    blind_hands: Vec<HandKind>,
    /// Ids of the cards in hand dealt face down.
    face_down: HashSet<u32>,
    /// The Clippy: id of the card always selected.
    forced: Option<u32>,
    /// The Windows Update: the joker that does nothing this hand.
    disabled_joker: Option<usize>,
    /// The Kernel Panic: jokers are face down.
    jokers_hidden: bool,
    /// The Vendor Lock-in: a joker was sold, cards score again.
    vendor_unlocked: bool,
    /// Pull requests merged, and the one for sale this ante.
    vouchers: Vec<Voucher>,
    voucher_offers: Vec<Voucher>,
    voucher_rng: Rng,
    /// Change Request: the boss was rerolled this ante.
    boss_rerolled: bool,
    /// The badges the small and big blinds give if skipped, and the badges won
    /// but not used yet.
    blind_tags: [Tag; 2],
    tags: Vec<Tag>,
    tag_rng: Rng,
    /// For the money badges: blinds skipped, hands played, discards left unused.
    skips: u32,
    hands_played: u32,
    unused_discards: u32,
    /// resolution: 4k: extra hand size for this blind.
    juggle: usize,
    /// Offers given for free (badges), aligned with `offers`; free packs.
    free_offers: Vec<bool>,
    free_packs: bool,
    /// Packs won by badges, opened one after the other.
    pending_packs: Vec<Pack>,
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
            packs: Vec::new(),
            pack_rng: Rng::stream(seed, "packs"),
            opened: None,
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
            hand_size_penalty: 0,
            boss: Boss::OomKiller,
            bosses_seen: Vec::new(),
            boss_rng: Rng::stream(seed, "boss"),
            boss_disabled: false,
            played_this_ante: HashSet::new(),
            blind_hands: Vec::new(),
            face_down: HashSet::new(),
            forced: None,
            disabled_joker: None,
            jokers_hidden: false,
            vendor_unlocked: false,
            vouchers: Vec::new(),
            voucher_offers: Vec::new(),
            voucher_rng: Rng::stream(seed, "vouchers"),
            boss_rerolled: false,
            blind_tags: [Tag::Stars100, Tag::Stars100],
            tags: Vec::new(),
            tag_rng: Rng::stream(seed, "tags"),
            skips: 0,
            hands_played: 0,
            unused_discards: 0,
            juggle: 0,
            free_offers: Vec::new(),
            free_packs: false,
            pending_packs: Vec::new(),
            beta_broken: false,
        };
        game.roll_boss();
        game.roll_voucher();
        game.roll_blind_tags();
        game.await_blind();
        game
    }

    /// The badges the small and big blinds of this ante give if skipped.
    fn roll_blind_tags(&mut self) {
        let pool: Vec<Tag> = Tag::ALL
            .into_iter()
            .filter(|t| t.min_ante() <= self.ante)
            .collect();
        for i in 0..2 {
            self.blind_tags[i] = pool[self.tag_rng.below(pool.len() as u64) as usize];
        }
    }

    /// Wins a badge: badges: 2x waits to copy the next one; the others act
    /// right away or wait for the next shop, blind or boss.
    fn gain_tag(&mut self, tag: Tag) {
        if tag == Tag::Badges2x {
            self.tags.push(tag);
            return;
        }
        let doubles = self.tags.iter().filter(|&&t| t == Tag::Badges2x).count();
        self.tags.retain(|&t| t != Tag::Badges2x);
        for _ in 0..=doubles {
            self.apply_tag(tag);
        }
    }

    fn apply_tag(&mut self, tag: Tag) {
        use crate::pack::PackSize;
        let pack = |kind, size| Pack { kind, size };
        match tag {
            Tag::Sponsors2x => self.money += self.money.min(40),
            Tag::Commits1k => self.money += self.hands_played,
            Tag::GcDone => self.money += self.unused_discards,
            Tag::SpeedBlazing => self.money += 5 * self.skips,
            Tag::Contributors2 => {
                for _ in 0..2 {
                    if self.has_room(None)
                        && let Some(joker) = self.random_joker_of(Rarity::Core)
                    {
                        self.jokers.push(OwnedJoker::new(joker, None));
                    }
                }
            }
            Tag::Version3 => {
                let known: Vec<HandKind> = HandKind::ALL
                    .into_iter()
                    .filter(|&k| !k.secret() || self.plays.get(k) > 0)
                    .collect();
                let kind = known[self.tag_rng.below(known.len() as u64) as usize];
                self.levels.raise(kind, 3);
            }
            Tag::Wontfix => self.roll_boss(),
            Tag::BuildPassing => self.open_free_pack(pack(PackKind::BaseDevel, PackSize::Meta)),
            Tag::DocsLatest => self.open_free_pack(pack(PackKind::Coreutils, PackSize::Meta)),
            Tag::ThemeDark => self.open_free_pack(pack(PackKind::Base16, PackSize::Meta)),
            Tag::FunYes => self.open_free_pack(pack(PackKind::JokersGit, PackSize::Meta)),
            Tag::FundingOligarchy => {
                self.open_free_pack(pack(PackKind::Oligarchy, PackSize::Normal))
            }
            // The others wait: next shop, next blind, next boss.
            _ => self.tags.push(tag),
        }
    }

    /// Opens a pack won by a badge, or queues it while another is open.
    fn open_free_pack(&mut self, pack: Pack) {
        if self.opened.is_some() {
            self.pending_packs.push(pack);
        } else {
            self.open_pack(pack);
        }
    }

    /// Takes the waiting badges this moment uses.
    fn take_tags(&mut self, wanted: &[Tag]) -> Vec<Tag> {
        let (used, kept) = self.tags.iter().partition(|t| wanted.contains(t));
        self.tags = kept;
        used
    }

    /// The shop opens: the badges waiting for it act.
    fn shop_tags(&mut self) {
        let used = self.take_tags(&[
            Tag::Stars100,
            Tag::Stars10k,
            Tag::Opacity0,
            Tag::BlurOn,
            Tag::ShadowOn,
            Tag::AnimationsOn,
            Tag::PrsWelcome,
            Tag::PriceFree,
            Tag::CacheHit,
        ]);
        for tag in used {
            match tag {
                Tag::Stars100 | Tag::Stars10k => {
                    let rarity = if tag == Tag::Stars100 {
                        Rarity::Extra
                    } else {
                        Rarity::Aur
                    };
                    if let Some(joker) = self.random_joker_of(rarity) {
                        self.offers.push(Offer::Joker {
                            joker,
                            edition: None,
                        });
                        self.free_offers.push(true);
                    }
                }
                Tag::Opacity0 | Tag::BlurOn | Tag::ShadowOn | Tag::AnimationsOn => {
                    let edition = match tag {
                        Tag::Opacity0 => Edition::Transparent,
                        Tag::BlurOn => Edition::Blur,
                        Tag::ShadowOn => Edition::Shadow,
                        _ => Edition::Animated,
                    };
                    let plain = self
                        .offers
                        .iter()
                        .position(|o| matches!(o, Offer::Joker { edition: None, .. }));
                    match plain {
                        Some(i) => {
                            if let Offer::Joker { edition: e, .. } = &mut self.offers[i] {
                                *e = Some(edition);
                            }
                            self.free_offers[i] = true;
                        }
                        None => {
                            if let Some(joker) = self.random_joker_of(Rarity::Core) {
                                self.offers.push(Offer::Joker {
                                    joker,
                                    edition: Some(edition),
                                });
                                self.free_offers.push(true);
                            }
                        }
                    }
                }
                Tag::PrsWelcome => self.add_voucher_offer(),
                Tag::PriceFree => {
                    self.free_offers.iter_mut().for_each(|f| *f = true);
                    self.free_packs = true;
                }
                Tag::CacheHit => self.reroll_cost = 0,
                _ => {}
            }
        }
    }

    pub fn has(&self, voucher: Voucher) -> bool {
        self.vouchers.contains(&voucher)
    }

    /// Of these pull requests, how many were merged.
    fn merged(&self, vouchers: &[Voucher]) -> usize {
        vouchers.iter().filter(|&&v| self.has(v)).count()
    }

    /// The pull request for sale this ante: one not merged yet, a follow-up
    /// only once its first part was.
    fn roll_voucher(&mut self) {
        self.voucher_offers.clear();
        self.add_voucher_offer();
    }

    /// One more pull request for sale (PRs: welcome), not already offered.
    fn add_voucher_offer(&mut self) {
        let pool: Vec<Voucher> = Voucher::ALL
            .into_iter()
            .filter(|&v| !self.has(v) && v.follows().is_none_or(|first| self.has(first)))
            .filter(|v| !self.voucher_offers.contains(v))
            .collect();
        if let Some(&voucher) = pool.get(self.voucher_rng.below(pool.len().max(1) as u64) as usize)
        {
            self.voucher_offers.push(voucher);
        }
    }

    /// Merges a pull request, with its immediate effects.
    fn merge(&mut self, voucher: Voucher) {
        self.vouchers.push(voucher);
        if matches!(voucher, Voucher::GitRevert | Voucher::GitResetHard) {
            self.ante = self.ante.saturating_sub(1).max(1);
        }
        if self.phase == Phase::Shop {
            self.reroll_cost = self.reroll_cost.saturating_sub(match voucher {
                Voucher::MirrorList | Voucher::FastestMirror => 2,
                _ => 0,
            });
        }
    }

    /// Shop prices after Early Bird (25% off) or Lifetime Deal (50% off).
    fn discounted(&self, price: u32) -> u32 {
        let factor = if self.has(Voucher::LifetimeDeal) {
            0.5
        } else if self.has(Voucher::EarlyBird) {
            0.75
        } else {
            1.0
        };
        if price == 0 {
            0
        } else {
            ((price as f64 * factor).round() as u32).max(1)
        }
    }

    /// Edition odds, raised by Compositor Tweaks (×2) and Eye Candy (×4).
    fn edition_factor(&self) -> f64 {
        if self.has(Voucher::EyeCandy) {
            4.0
        } else if self.has(Voucher::CompositorTweaks) {
            2.0
        } else {
            1.0
        }
    }

    fn roll_edition(&self, rng: &mut Rng) -> Option<Edition> {
        let roll = rng.unit();
        let mut odds = 0.0;
        let factor = self.edition_factor();
        Edition::ALL.into_iter().rev().find(|e| {
            odds += e.odds() * factor;
            roll < odds
        })
    }

    fn hands_per_blind(&self) -> u32 {
        let more = self.merged(&[Voucher::Overtime, Voucher::CrunchTime]) as u32;
        let less = self.merged(&[Voucher::GitRevert]) as u32;
        (HANDS_PER_BLIND + more).saturating_sub(less).max(1)
    }

    fn discards_per_blind(&self) -> u32 {
        let more =
            self.count(Joker::UndoHistory) + self.merged(&[Voucher::RmI, Voucher::EmptyTrash]);
        let less = self.merged(&[Voucher::GitResetHard]);
        (DISCARDS_PER_BLIND + more as u32).saturating_sub(less as u32)
    }

    fn max_interest(&self) -> u32 {
        if self.has(Voucher::IndexFund) {
            20
        } else if self.has(Voucher::HighYieldSavings) {
            10
        } else {
            MAX_INTEREST
        }
    }

    fn consumable_slots(&self) -> usize {
        CONSUMABLE_SLOTS + self.merged(&[Voucher::ClipboardManager])
    }

    fn base_reroll_cost(&self) -> u32 {
        REROLL_COST
            .saturating_sub(2 * self.merged(&[Voucher::MirrorList, Voucher::FastestMirror]) as u32)
    }

    /// The price of rerolling the boss, when Change Request allows it now.
    fn boss_reroll_cost(&self) -> Option<u32> {
        let allowed = self.phase == Phase::BlindSelect
            && self.has(Voucher::ChangeRequest)
            && (self.has(Voucher::EmergencyChange) || !self.boss_rerolled);
        allowed.then_some(VOUCHER_PRICE)
    }

    pub fn buy_voucher(&mut self, index: usize) -> Result<(), GameError> {
        self.in_shop()?;
        let voucher = *self
            .voucher_offers
            .get(index)
            .ok_or_else(|| GameError::BadSelection("no pull request for sale".into()))?;
        self.spend(VOUCHER_PRICE)?;
        self.voucher_offers.remove(index);
        self.merge(voucher);
        Ok(())
    }

    /// Change Request: draws another boss for this ante.
    pub fn reroll_boss(&mut self) -> Result<(), GameError> {
        let cost = self
            .boss_reroll_cost()
            .ok_or_else(|| GameError::BadSelection("no boss reroll available".into()))?;
        self.spend(cost)?;
        self.roll_boss();
        self.boss_rerolled = true;
        Ok(())
    }

    fn target(&self) -> f64 {
        self.target_of(self.blind)
    }

    /// The boss blind's target follows the boss (The Kubernetes Cluster ×4…).
    fn target_of(&self, kind: BlindKind) -> f64 {
        match kind {
            BlindKind::Boss => blind::ante_base(self.ante) * self.boss.multiplier(),
            _ => blind::target(self.ante, kind),
        }
    }

    /// The ante's boss: a final one at the winning ante and its multiples,
    /// otherwise one allowed at this ante; no repeat until all were met.
    fn roll_boss(&mut self) {
        let final_ante = self.ante % WINNING_ANTE == 0;
        let fits = |b: &Boss| {
            if final_ante {
                b.is_final()
            } else {
                !b.is_final() && b.min_ante() <= self.ante
            }
        };
        let mut pool: Vec<Boss> = Boss::ALL
            .into_iter()
            .filter(|b| fits(b) && !self.bosses_seen.contains(b))
            .collect();
        if pool.is_empty() {
            self.bosses_seen.retain(|b| !fits(b));
            pool = Boss::ALL.into_iter().filter(fits).collect();
        }
        self.boss = pool[self.boss_rng.below(pool.len() as u64) as usize];
        self.bosses_seen.push(self.boss);
    }

    /// The boss whose effect applies now: in the boss blind, unless kill -9
    /// is owned or Rollback was sold.
    fn boss_effect(&self) -> Option<Boss> {
        let on =
            self.blind == BlindKind::Boss && !self.boss_disabled && self.count(Joker::Kill9) == 0;
        on.then_some(self.boss)
    }

    fn boss_is(&self, boss: Boss) -> bool {
        self.boss_effect() == Some(boss)
    }

    /// Whether a boss keeps this card from scoring.
    fn debuffed(&self, card: &Card) -> bool {
        let Some(boss) = self.boss_effect() else {
            return false;
        };
        match boss {
            Boss::AnonymousAccount => card.is_face(),
            Boss::StaleCache => self.played_this_ante.contains(&card.id),
            Boss::VendorLockIn => !self.vendor_unlocked,
            _ => boss.debuffed_suit().is_some_and(|suit| card.is_suit(suit)),
        }
    }

    /// The Windows Update: picks the joker that does nothing this hand.
    fn pick_disabled_joker(&mut self) {
        self.disabled_joker = None;
        if self.boss_is(Boss::WindowsUpdate) && !self.jokers.is_empty() {
            self.disabled_joker = Some(self.boss_rng.below(self.jokers.len() as u64) as usize);
        }
    }

    /// The Clippy: picks a card of the hand to keep selected, if the last one left.
    fn pick_forced_card(&mut self) {
        let kept = self
            .forced
            .is_some_and(|id| self.hand.iter().any(|c| c.id == id));
        if !self.boss_is(Boss::Clippy) {
            self.forced = None;
        } else if !kept && !self.hand.is_empty() {
            let i = self.boss_rng.below(self.hand.len() as u64) as usize;
            self.forced = Some(self.hand[i].id);
        }
    }

    /// Adds The Clippy's card to a selection; refuses a full one without it.
    fn with_forced(&self, indices: &[usize]) -> Result<Vec<usize>, GameError> {
        let mut indices = indices.to_vec();
        if let Some(i) = self
            .forced
            .and_then(|id| self.hand.iter().position(|c| c.id == id))
            && !indices.contains(&i)
        {
            if indices.len() >= MAX_PLAY {
                return Err(GameError::BadSelection(
                    "The Clippy insists on one of your cards".into(),
                ));
            }
            indices.push(i);
        }
        Ok(indices)
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

    /// Ultrawide jokers and Transparent cards in hand each add one card;
    /// Consolidation, Offshore Account and The 4:3 Monitor take one away.
    fn hand_size(&self) -> usize {
        let transparent = self
            .hand
            .iter()
            .filter(|c| c.edition == Some(Edition::Transparent))
            .count();
        let monitor = usize::from(self.boss_is(Boss::Monitor43));
        let displays = self.merged(&[Voucher::Hidpi, Voucher::Display4k]);
        (HAND_SIZE + self.count(Joker::Ultrawide) + transparent + displays + self.juggle)
            .saturating_sub(self.hand_size_penalty + monitor)
            .max(1)
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
        MAX_JOKERS + transparent + self.merged(&[Voucher::FeatureFlag])
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
        self.hands_left = self.hands_per_blind();
        self.discards_left = self.discards_per_blind();
        self.discards_used = 0;
        self.blind_hands.clear();
        self.juggle = 0;
        self.face_down.clear();
        self.forced = None;
        self.disabled_joker = None;
        self.jokers_hidden = false;
        self.vendor_unlocked = false;
        self.boss_disabled = false;
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
                self.played_this_ante.clear();
                self.roll_boss();
                self.roll_voucher();
                self.roll_blind_tags();
                self.boss_rerolled = false;
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
        self.juggle = 3 * self.take_tags(&[Tag::Resolution4k]).len();
        // Jokers and pull requests merged since the blind was announced count too.
        self.hands_left = self.hands_per_blind();
        self.discards_left = self.discards_per_blind();
        for _ in 0..self.abilities(Joker::ClosedSource) {
            let blob = self.new_card().with_enhancement(Enhancement::BinaryBlob);
            self.deck.push(blob);
            self.draw_pile.push(blob);
        }
        self.deck_rng.shuffle(&mut self.draw_pile);
        self.phase = Phase::Playing;
        match self.boss_effect() {
            Some(Boss::ReadOnlyFilesystem) => self.discards_left = 0,
            Some(Boss::Deadline) => self.hands_left = 1,
            Some(Boss::KernelPanic) => {
                self.boss_rng.shuffle(&mut self.jokers);
                self.jokers_hidden = true;
            }
            _ => {}
        }
        self.draw(Draw::Deal);
        self.pick_disabled_joker();
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
        self.pick_forced_card();
        Ok(())
    }

    pub fn skip_blind(&mut self) -> Result<(), GameError> {
        self.selecting()?;
        if !self.blind.skippable() {
            return Err(GameError::CannotSkipBoss);
        }
        let tag = self.blind_tags[if self.blind == BlindKind::Small { 0 } else { 1 }];
        self.skipped.push(self.blind);
        self.skips += 1;
        for joker in &mut self.jokers {
            joker.on_blind_skipped();
        }
        self.advance();
        self.gain_tag(tag);
        Ok(())
    }

    /// Fills the hand up (The Rate Limiter: 3 cards after a hand or discard);
    /// some bosses deal cards face down.
    fn draw(&mut self, cause: Draw) {
        let limit = if self.boss_is(Boss::RateLimiter) && cause != Draw::Deal {
            self.hand.len() + 3
        } else {
            self.hand_size()
        };
        while self.hand.len() < limit
            && let Some(card) = self.draw_pile.pop()
        {
            let hidden = match self.boss_effect() {
                Some(Boss::BlackScreen) => cause == Draw::Deal,
                Some(Boss::ScreenLock) => cause == Draw::AfterHand,
                Some(Boss::FlakyWifi) => self.boss_rng.chance(7),
                Some(Boss::DarkPattern) => card.is_face(),
                _ => false,
            };
            if hidden {
                self.face_down.insert(card.id);
            }
            self.hand.push(card);
        }
        self.sort_hand();
    }

    /// Hand sorted as at the table: descending rank, then suit; Binary Blobs last.
    fn sort_hand(&mut self) {
        self.hand
            .sort_by_key(|c| (!c.has_rank(), std::cmp::Reverse(c.rank), c.suit as u8));
    }

    /// Checks a selection of the hand (the blind's, or an open coreutils
    /// pack's) and returns its sorted indices.
    fn validate(&self, indices: &[usize]) -> Result<Vec<usize>, GameError> {
        if self.phase != Phase::Playing && self.phase != Phase::Pack {
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
        if self.phase != Phase::Playing {
            return Err(GameError::NotPlaying);
        }
        let indices = self.with_forced(indices)?;
        let played = self.take(&indices)?;
        for card in &played {
            self.face_down.remove(&card.id);
        }
        let money_before = self.money;
        let four_fingers = self.four_fingers();
        let evaluated = evaluate_with(&played, four_fingers);
        let mut notes = Vec::new();
        let boss = self.boss_effect();
        let most_played = HandKind::ALL
            .iter()
            .all(|&k| self.plays.get(evaluated.kind) >= self.plays.get(k));
        let refused = match boss {
            Some(Boss::MandatoryMeeting) => played.len() < 5,
            Some(Boss::CodeReviewer) => self.blind_hands.contains(&evaluated.kind),
            Some(Boss::StyleGuide) => self
                .blind_hands
                .first()
                .is_some_and(|&k| k != evaluated.kind),
            _ => false,
        };
        self.plays.add(evaluated.kind);
        self.hands_played += 1;
        self.blind_hands.push(evaluated.kind);
        if refused {
            let breakdown = self.refused_hand(played, &evaluated, boss);
            return Ok(self.end_hand(breakdown, true));
        }
        let mut triggered = boss == Some(Boss::Clippy) || self.disabled_joker.is_some();
        if boss == Some(Boss::Regression) {
            self.levels.lower(evaluated.kind);
            notes.push(format!("{} lost a level.", evaluated.kind.name()));
            triggered = true;
        }
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
        let debuffed: Vec<u32> = played
            .iter()
            .chain(&self.hand)
            .filter(|c| self.debuffed(c))
            .map(|c| c.id)
            .collect();
        triggered |= played.iter().any(|c| debuffed.contains(&c.id));
        let showcase: Vec<usize> = if self.has(Voucher::ThemeShowcase) {
            let theme = Consumable::Theme(Theme::ALL[evaluated.kind as usize]);
            (0..self.consumables.len())
                .filter(|&i| self.consumables[i].consumable == theme)
                .collect()
        } else {
            Vec::new()
        };
        let table = Table {
            played: &played,
            held: &self.hand,
            jokers: &self.jokers,
            levels: &self.levels,
            plays: &self.plays,
            four_fingers,
            deck: &self.deck,
            debuffed: &debuffed,
            halve_base: boss == Some(Boss::ThermalThrottle),
            disabled_joker: self.disabled_joker,
            showcase: &showcase,
        };
        let mut breakdown = score(&table, &mut self.score_rng);
        breakdown.notes = notes;
        self.money += breakdown.money;
        match boss {
            Some(Boss::ThermalThrottle) => triggered = true,
            Some(Boss::PerSeatLicense) => {
                self.money = self.money.saturating_sub(played.len() as u32);
                triggered = true;
            }
            Some(Boss::CloudBill) if most_played => {
                self.money = 0;
                breakdown
                    .notes
                    .push("The Cloud Bill: money set to $0.".into());
                triggered = true;
            }
            _ => {}
        }
        self.after_scoring(&mut breakdown, &scoring);
        self.create_from_hand(
            &evaluated,
            &played,
            &scoring,
            money_before,
            &mut breakdown.notes,
        );
        let notes = &mut breakdown.notes;
        self.jokers.retain_mut(|joker| {
            let kept = joker.after_hand();
            if !kept {
                notes.push(format!("{} ran out.", joker.joker.name()));
            }
            kept
        });
        self.played_this_ante.extend(played.iter().map(|c| c.id));
        Ok(self.end_hand(breakdown, triggered))
    }

    /// A hand a boss does not allow: it counts, but scores nothing.
    fn refused_hand(
        &mut self,
        played: Vec<Card>,
        evaluated: &Evaluated,
        boss: Option<Boss>,
    ) -> Breakdown {
        self.played_this_ante.extend(played.iter().map(|c| c.id));
        Breakdown {
            hand: evaluated.kind,
            hand_name: evaluated.kind.name(),
            scoring: evaluated.scoring.clone(),
            cards: played,
            base_chips: 0,
            base_mult: 0.0,
            steps: Vec::new(),
            chips: 0,
            mult: 0.0,
            total: 0.0,
            money: 0,
            notes: vec![format!(
                "Not allowed by {}.",
                boss.map_or("the boss", Boss::name)
            )],
            lucky_hits: 0,
            broken: Vec::new(),
        }
    }

    /// After any hand: Incident Commander pays if the boss struck, The OOM
    /// Killer discards 2 cards, then the blind is won, lost or goes on.
    fn end_hand(&mut self, mut breakdown: Breakdown, triggered: bool) -> Breakdown {
        let commanders = self.abilities(Joker::IncidentCommander) as u32;
        if triggered && commanders > 0 {
            self.money += 8 * commanders;
            breakdown
                .notes
                .push(format!("Incident Commander: +${}.", 8 * commanders));
        }
        self.score += breakdown.total;
        self.hands_left -= 1;
        if self.score >= self.target() {
            self.win_blind();
        } else if self.hands_left == 0 {
            self.phase = Phase::Lost;
        } else {
            if self.boss_is(Boss::OomKiller) && !self.hand.is_empty() {
                let mut ids: Vec<u32> = self.hand.iter().map(|c| c.id).collect();
                self.boss_rng.shuffle(&mut ids);
                ids.truncate(2);
                self.hand.retain(|c| !ids.contains(&c.id));
                breakdown
                    .notes
                    .push(format!("The OOM Killer discarded {} cards.", ids.len()));
            }
            self.draw(Draw::AfterHand);
            self.pick_disabled_joker();
            self.pick_forced_card();
        }
        breakdown
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
        }
        let broken = breakdown.broken.clone();
        self.destroy_cards(&broken);
        let chips = 5 * self.abilities(Joker::IncrementalBuild) as u64;
        if chips > 0 {
            for card in scoring {
                self.modify_card(card.id, |c| c.bonus_chips += chips);
            }
        }
    }

    /// Jokers that create consumables from the hand played: Seed Round (a
    /// Straight Flush), Angel Investor (a lone 6 first), Vim Golf (an ace and a
    /// Straight), Free Tier ($4 or less), man 8 (1 in 4 per scoring 8).
    fn create_from_hand(
        &mut self,
        evaluated: &Evaluated,
        played: &[Card],
        scoring: &[Card],
        money: u32,
        notes: &mut Vec<String>,
    ) {
        if evaluated.contains(HandKind::StraightFlush) {
            self.create_oligarchy_cards(self.abilities(Joker::SeedRound), "Seed Round", notes);
        }
        let first = self.hands_left == self.hands_per_blind();
        let six = matches!(played, [card] if card.has_rank() && card.rank == 6);
        let chmods = self.abilities(Joker::AngelInvestor);
        if first && six && chmods > 0 {
            self.destroy_cards(&[played[0].id]);
            self.create_oligarchy_cards(chmods, "Angel Investor", notes);
        }
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
        self.unused_discards += self.discards_left;
        let mut reward = self.reward();
        if self.blind == BlindKind::Boss {
            let funded = 25 * self.take_tags(&[Tag::SeriesA]).len() as u32;
            if funded > 0 {
                reward.badges = funded;
                reward.total += funded;
                reward
                    .notes
                    .push(format!("funding: series A paid ${funded}."));
            }
        }
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
                if self.clipboard_room() {
                    self.consumables.push(Consumable::Theme(theme).into());
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
        if indices.is_empty() || self.phase == Phase::Pack {
            return Ok(None);
        }
        let hidden = indices.iter().any(|&i| {
            self.hand
                .get(i)
                .is_some_and(|c| self.face_down.contains(&c.id))
        });
        if hidden {
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
        if self.phase != Phase::Playing {
            return Err(GameError::NotPlaying);
        }
        if self.phase == Phase::Playing && self.discards_left == 0 {
            return Err(GameError::NoDiscardsLeft);
        }
        let indices = self.with_forced(indices)?;
        let discarded = self.take(&indices)?;
        for card in &discarded {
            self.face_down.remove(&card.id);
        }
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
            joker.on_cards_discarded(discarded.len());
        }
        self.draw(Draw::AfterDiscard);
        self.pick_forced_card();
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
        let blind = match self.blind {
            BlindKind::Boss => self.boss.reward(),
            kind => kind.reward(),
        };
        let hands = self.hands_left;
        let interest = (self.money / 5).min(self.max_interest());
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
            badges: 0,
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
        self.hand.clear();
        self.reroll_cost = self.base_reroll_cost();
        self.roll_offers();
        self.packs = (0..SHOP_PACKS).map(|_| self.roll_pack()).collect();
        self.shop_tags();
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
        self.free_offers.clear();
        let mut rng = self.shop_rng.clone();
        let factor = |base: Voucher, more: Voucher| {
            if self.has(more) {
                4.0
            } else if self.has(base) {
                2.0
            } else {
                1.0
            }
        };
        let theme = 4.0 * factor(Voucher::ThemeGallery, Voucher::ThemeMarketplace);
        let command = 4.0 * factor(Voucher::ShellAliases, Voucher::ShellPlugins);
        let card = if self.has(Voucher::CardMarketplace) {
            4.0
        } else {
            0.0
        };
        let slots =
            SHOP_OFFERS + self.merged(&[Voucher::MoreWorkspaces, Voucher::EvenMoreWorkspaces]);
        for _ in 0..slots {
            let roll = rng.unit() * (20.0 + theme + command + card);
            let offer = if roll < 20.0 {
                self.roll_joker(&mut rng)
            } else if roll < 20.0 + theme {
                self.roll_theme(&mut rng)
            } else if roll < 20.0 + theme + command {
                let command = Command::ALL[rng.below(Command::ALL.len() as u64) as usize];
                Some(Offer::Consumable(Consumable::Command(command)))
            } else {
                Some(Offer::Card(self.shop_card(&mut rng)))
            };
            match offer {
                Some(offer) => {
                    self.offers.push(offer);
                    self.free_offers.push(false);
                }
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
        let edition = self.roll_edition(rng);
        Some(Offer::Joker { joker, edition })
    }

    /// A playing card for the shop (Card Marketplace); with Premium Cards, it
    /// may come with modifiers.
    fn shop_card(&self, rng: &mut Rng) -> Card {
        let mut card = Card::new(2 + rng.below(13) as u8, Suit::ALL[rng.below(4) as usize]);
        if self.has(Voucher::PremiumCards) {
            if rng.unit() < 0.4 {
                card.enhancement = Some(Enhancement::ALL[rng.below(8) as usize]);
            }
            if rng.unit() < 0.2 {
                card.seal = Some(Seal::ALL[rng.below(Seal::ALL.len() as u64) as usize]);
            }
            card.edition = self.roll_edition(rng);
        }
        card
    }

    /// A playing card costs $1, plus $1 per modifier.
    fn card_price(card: &Card) -> u32 {
        1 + u32::from(card.enhancement.is_some())
            + u32::from(card.seal.is_some())
            + u32::from(card.edition.is_some())
    }

    /// What a shop offer costs: free when a badge says so.
    fn offer_price(&self, index: usize) -> u32 {
        if self.free_offers.get(index).copied().unwrap_or(false) {
            return 0;
        }
        match self.offers[index] {
            Offer::Joker { joker, edition } => {
                self.discounted(OwnedJoker::new(joker, edition).price())
            }
            Offer::Consumable(c) => self.consumable_price(c),
            Offer::Card(card) => self.discounted(Self::card_price(&card)),
        }
    }

    fn consumable_price(&self, consumable: Consumable) -> u32 {
        match consumable {
            Consumable::Theme(_) if self.count(Joker::FreeAsInBeer) > 0 => 0,
            _ => self.discounted(consumable.price()),
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
                self.spend(self.offer_price(index))?;
                self.jokers.push(owned);
            }
            Offer::Card(card) => {
                self.spend(self.offer_price(index))?;
                self.next_card_id += 1;
                self.deck.push(card.with_id(self.next_card_id - 1));
            }
            Offer::Consumable(consumable) => {
                if !self.clipboard_room() {
                    return Err(GameError::ClipboardFull);
                }
                self.spend(self.offer_price(index))?;
                self.consumables.push(consumable.into());
            }
        }
        self.offers.remove(index);
        self.free_offers.remove(index);
        Ok(())
    }

    /// Sells a joker: in the shop, before a blind or during it (Rollback, sold
    /// in the boss blind, disables the boss; any sale unlocks The Vendor Lock-in).
    pub fn sell(&mut self, index: usize) -> Result<(), GameError> {
        if !matches!(
            self.phase,
            Phase::Shop | Phase::BlindSelect | Phase::Playing
        ) {
            return Err(GameError::NotInShop);
        }
        if index >= self.jokers.len() {
            return Err(GameError::BadSelection(format!("no joker #{index}")));
        }
        let sold = self.jokers.remove(index);
        self.money += sold.sell_value();
        if self.phase == Phase::Playing {
            self.vendor_unlocked = true;
            if sold.joker == Joker::Rollback && self.blind == BlindKind::Boss {
                self.boss_disabled = true;
                self.face_down.clear();
                self.jokers_hidden = false;
                self.forced = None;
            }
            self.pick_disabled_joker();
        }
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
            .map(|held| held.consumable)
            .ok_or_else(|| GameError::BadSelection(format!("no consumable #{index}")))
    }

    /// Whether the clipboard has a free slot (Transparent consumables take none).
    fn clipboard_room(&self) -> bool {
        self.consumables.iter().filter(|h| !h.transparent).count() < self.consumable_slots()
    }

    /// Adds a consumable to the clipboard if there is room.
    fn create(&mut self, consumable: Consumable) -> bool {
        let room = self.clipboard_room();
        if room {
            self.consumables.push(consumable.into());
        }
        room
    }

    /// Creates a random oligarchy card for each of `times`, while there is room; says so.
    fn create_oligarchy_cards(&mut self, times: usize, source: &str, notes: &mut Vec<String>) {
        for _ in 0..times {
            let card = Consumable::Oligarchy(roll_oligarchy(&mut self.command_rng));
            if self.create(card) {
                notes.push(format!("{source}: {} added to the clipboard.", card.name()));
            }
        }
    }

    /// Destroys cards for good (deck, draw pile, hand); ThePrimeagen grows with
    /// the face cards among them.
    fn destroy_cards(&mut self, ids: &[u32]) {
        let faces = self
            .deck
            .iter()
            .filter(|c| ids.contains(&c.id) && c.is_face())
            .count();
        self.deck.retain(|c| !ids.contains(&c.id));
        self.draw_pile.retain(|c| !ids.contains(&c.id));
        self.hand.retain(|c| !ids.contains(&c.id));
        if faces > 0 {
            for joker in &mut self.jokers {
                joker.on_faces_destroyed(faces);
            }
        }
    }

    /// Destroys up to `count` random cards of the hand.
    fn destroy_random_in_hand(&mut self, count: usize) {
        let mut ids: Vec<u32> = self.hand.iter().map(|c| c.id).collect();
        self.card_rng.shuffle(&mut ids);
        ids.truncate(count);
        self.destroy_cards(&ids);
    }

    /// A new enhanced card of one of these ranks, added to the deck and the hand.
    fn add_enhanced_card(&mut self, ranks: &[u8]) {
        let rank = ranks[self.card_rng.below(ranks.len() as u64) as usize];
        let suit = Suit::ALL[self.card_rng.below(4) as usize];
        let enhancement = Enhancement::ALL[self.card_rng.below(8) as usize];
        self.next_card_id += 1;
        let card = Card::new(rank, suit)
            .with_id(self.next_card_id - 1)
            .with_enhancement(enhancement);
        self.deck.push(card);
        self.hand.push(card);
    }

    /// A random joker of this rarity not owned yet.
    fn random_joker_of(&mut self, rarity: Rarity) -> Option<Joker> {
        let pool: Vec<Joker> = Joker::ALL
            .into_iter()
            .filter(|&j| j.rarity() == rarity && !self.jokers.iter().any(|o| o.joker == j))
            .filter(|&j| rarity == Rarity::Legendary || self.on_sale(j))
            .collect();
        pool.get(self.command_rng.below(pool.len().max(1) as u64) as usize)
            .copied()
    }

    fn run_oligarchy(&mut self, oligarchy: Oligarchy, targets: &[usize]) {
        let target = targets.first().map(|&i| self.hand[i].id);
        let seal = |seal| move |card: &mut Card| card.seal = Some(seal);
        match oligarchy {
            Oligarchy::BoardAppointment | Oligarchy::HostileTakeover | Oligarchy::HiringSpree => {
                self.destroy_random_in_hand(1);
                let (count, ranks): (usize, Vec<u8>) = match oligarchy {
                    Oligarchy::BoardAppointment => (3, vec![11, 12, 13]),
                    Oligarchy::HostileTakeover => (2, vec![ACE]),
                    _ => (4, (2..=10).collect()),
                };
                for _ in 0..count {
                    self.add_enhanced_card(&ranks);
                }
            }
            Oligarchy::GoldenHandshake => {
                self.modify_card(target.unwrap_or_default(), seal(Seal::GoldStar))
            }
            Oligarchy::OligarchyHarness => {
                self.modify_card(target.unwrap_or_default(), seal(Seal::Retry))
            }
            Oligarchy::BrandGuidelines => {
                self.modify_card(target.unwrap_or_default(), seal(Seal::ThemeSync))
            }
            Oligarchy::Surveillance => {
                self.modify_card(target.unwrap_or_default(), seal(Seal::ShellHistory))
            }
            Oligarchy::OligarchyTheme => {
                let edition = Edition::ALL[self.card_rng.below(3) as usize];
                self.modify_card(target.unwrap_or_default(), |c| c.edition = Some(edition));
            }
            Oligarchy::LeveragedBuyout => {
                if let Some(joker) = self.random_joker_of(Rarity::Aur) {
                    self.jokers.push(OwnedJoker::new(joker, None));
                }
                self.money = 0;
            }
            Oligarchy::Monopoly => {
                let suit = Suit::ALL[self.card_rng.below(4) as usize];
                let ids: Vec<u32> = self.hand.iter().map(|c| c.id).collect();
                for id in ids {
                    self.modify_card(id, |c| c.suit = suit);
                }
            }
            Oligarchy::Consolidation => {
                let rank = 2 + self.card_rng.below(13) as u8;
                let ids: Vec<u32> = self.hand.iter().map(|c| c.id).collect();
                for id in ids {
                    self.modify_card(id, |c| c.rank = rank);
                }
                self.hand_size_penalty += 1;
            }
            Oligarchy::OffshoreAccount | Oligarchy::Restructuring => {
                let bare: Vec<usize> = (0..self.jokers.len())
                    .filter(|&j| self.jokers[j].edition.is_none())
                    .collect();
                let j = bare[self.command_rng.below(bare.len() as u64) as usize];
                if oligarchy == Oligarchy::OffshoreAccount {
                    self.jokers[j].edition = Some(Edition::Transparent);
                    self.hand_size_penalty += 1;
                } else {
                    self.jokers[j].edition = Some(Edition::Animated);
                    self.jokers = vec![self.jokers[j].clone()];
                }
            }
            Oligarchy::MassLayoffs => {
                self.destroy_random_in_hand(5);
                self.money += 20;
            }
            Oligarchy::Merger => {
                let j = self.command_rng.below(self.jokers.len() as u64) as usize;
                let original = self.jokers[j].clone();
                let mut copy = original.clone();
                if copy.edition == Some(Edition::Transparent) {
                    copy.edition = None;
                }
                self.jokers = vec![original, copy];
            }
            Oligarchy::StockSplit => {
                if let Some(card) = self.hand.iter().find(|c| Some(c.id) == target).copied() {
                    for _ in 0..2 {
                        self.next_card_id += 1;
                        let copy = card.with_id(self.next_card_id - 1);
                        self.deck.push(copy);
                        self.hand.push(copy);
                    }
                }
            }
            Oligarchy::AspiringBillionaire => {
                if let Some(joker) = self.random_joker_of(Rarity::Legendary) {
                    self.jokers.push(OwnedJoker::new(joker, None));
                }
            }
            Oligarchy::PraiseTheOligarchy => {
                for kind in HandKind::ALL {
                    self.levels.raise(kind, 1);
                }
            }
        }
        self.sort_hand();
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
        let targets = self.targets(consumable, cards)?;
        self.check_usable(consumable)?;
        self.consumables.remove(index);
        self.apply_consumable(consumable, &targets);
        Ok(())
    }

    /// Checks the hand cards a consumable needs selected.
    fn targets(&self, consumable: Consumable, cards: &[usize]) -> Result<Vec<usize>, GameError> {
        match consumable.targets() {
            Some((min, max)) => {
                let sorted = self.validate(cards)?;
                if !(min..=max).contains(&sorted.len()) {
                    return Err(GameError::BadSelection(format!(
                        "{} needs {min} to {max} cards",
                        consumable.name()
                    )));
                }
                Ok(sorted)
            }
            None => Ok(Vec::new()),
        }
    }

    fn apply_consumable(&mut self, consumable: Consumable, targets: &[usize]) {
        match consumable {
            Consumable::Theme(theme) => {
                self.levels.raise(theme.hand(), 1);
                self.themes_used.insert(theme);
                for joker in &mut self.jokers {
                    joker.on_theme_used();
                }
            }
            Consumable::Oligarchy(oligarchy) => self.run_oligarchy(oligarchy, targets),
            Consumable::Command(command) => {
                self.run_command(command, targets);
                for joker in &mut self.jokers {
                    joker.on_command_used();
                }
            }
        }
        // Up Arrow recalls themes and commands, but itself or oligarchy cards.
        if !matches!(
            consumable,
            Consumable::Command(Command::UpArrow) | Consumable::Oligarchy(_)
        ) {
            self.last_used = Some(consumable);
        }
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
            Consumable::Oligarchy(card)
                if card.needs_hand()
                    && (self.hand.is_empty()
                        || !matches!(self.phase, Phase::Playing | Phase::Pack)) =>
            {
                refuse("it needs cards in hand: use it during a blind")
            }
            Consumable::Oligarchy(Oligarchy::LeveragedBuyout | Oligarchy::AspiringBillionaire)
                if !self.has_room(None) =>
            {
                Err(GameError::JokerSlotsFull)
            }
            Consumable::Oligarchy(Oligarchy::OffshoreAccount | Oligarchy::Restructuring)
                if self.jokers.iter().all(|j| j.edition.is_some()) =>
            {
                refuse("no joker without an edition")
            }
            Consumable::Oligarchy(Oligarchy::Merger) if self.jokers.is_empty() => {
                refuse("no joker to copy")
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
            Command::RmRf => self.destroy_cards(&ids),
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

    /// A pack for the shop, drawn by the original's weights.
    fn roll_pack(&mut self) -> Pack {
        let roll = self.pack_rng.unit();
        let mut odds = 0.0;
        let all = Pack::odds();
        all.iter()
            .find(|(_, w)| {
                odds += w;
                roll < odds
            })
            .unwrap_or(&all[0])
            .0
    }

    fn pack_price(&self, pack: Pack) -> u32 {
        if self.free_packs || (pack.kind == PackKind::Base16 && self.count(Joker::FreeAsInBeer) > 0)
        {
            0
        } else {
            self.discounted(pack.price())
        }
    }

    pub fn buy_pack(&mut self, index: usize) -> Result<(), GameError> {
        self.in_shop()?;
        let pack = *self
            .packs
            .get(index)
            .ok_or_else(|| GameError::BadSelection(format!("no pack #{index}")))?;
        self.spend(self.pack_price(pack))?;
        self.packs.remove(index);
        self.open_pack(pack);
        Ok(())
    }

    /// A playing card for a base-devel pack: 40% enhanced, 20% sealed, and
    /// sometimes an edition.
    fn pack_card(&mut self) -> Card {
        let mut rng = self.pack_rng.clone();
        let mut card = Card::new(2 + rng.below(13) as u8, Suit::ALL[rng.below(4) as usize]);
        if rng.unit() < 0.4 {
            card.enhancement = Some(Enhancement::ALL[rng.below(8) as usize]);
        }
        if rng.unit() < 0.2 {
            card.seal = Some(Seal::ALL[rng.below(Seal::ALL.len() as u64) as usize]);
        }
        card.edition = self.roll_edition(&mut rng);
        self.pack_rng = rng;
        card
    }

    /// Draws a pack's cards, without duplicates; a coreutils pack also draws a
    /// hand from the deck for its commands.
    fn open_pack(&mut self, pack: Pack) {
        let mut items: Vec<PackItem> = Vec::new();
        // Theme Preview: the theme of the most played hand comes first.
        if pack.kind == PackKind::Base16 && self.has(Voucher::ThemePreview) {
            let most = HandKind::ALL
                .into_iter()
                .filter(|&k| self.plays.get(k) > 0)
                .max_by_key(|&k| self.plays.get(k));
            if let Some(kind) = most {
                items.push(PackItem::Consumable(Consumable::Theme(
                    Theme::ALL[kind as usize],
                )));
            }
        }
        for _ in 0..pack.shown() * 10 {
            if items.len() == pack.shown() {
                break;
            }
            let mut rng = self.pack_rng.clone();
            let item = match pack.kind {
                // Insider Access: 1 in 5 cards is an oligarchy card.
                PackKind::Coreutils if self.has(Voucher::InsiderAccess) && rng.chance(5) => Some(
                    PackItem::Consumable(Consumable::Oligarchy(roll_oligarchy(&mut rng))),
                ),
                PackKind::Coreutils => Some(PackItem::Consumable(Consumable::Command(
                    Command::ALL[rng.below(Command::ALL.len() as u64) as usize],
                ))),
                PackKind::Base16 => match self.roll_theme(&mut rng) {
                    Some(Offer::Consumable(theme)) => Some(PackItem::Consumable(theme)),
                    _ => None,
                },
                PackKind::JokersGit => match self.roll_joker(&mut rng) {
                    Some(Offer::Joker { joker, edition }) => {
                        Some(PackItem::Joker { joker, edition })
                    }
                    _ => None,
                },
                PackKind::Oligarchy => Some(PackItem::Consumable(Consumable::Oligarchy(
                    roll_oligarchy(&mut rng),
                ))),
                PackKind::BaseDevel => None,
            };
            self.pack_rng = rng;
            let item = match pack.kind {
                PackKind::BaseDevel => Some(PackItem::Card(self.pack_card())),
                _ => item,
            };
            let duplicate = |a: &PackItem, b: &PackItem| match (a, b) {
                (PackItem::Joker { joker: x, .. }, PackItem::Joker { joker: y, .. }) => x == y,
                (PackItem::Card(_), PackItem::Card(_)) => false,
                _ => a == b,
            };
            if let Some(item) = item
                && !items.iter().any(|i| duplicate(i, &item))
            {
                items.push(item);
            }
        }
        if pack.kind.with_hand() {
            let mut cards = self.deck.clone();
            self.pack_rng.shuffle(&mut cards);
            cards.truncate(self.hand_size());
            self.hand = cards;
            self.sort_hand();
        }
        for _ in 0..self.abilities(Joker::OptionalDependencies) {
            if self.command_rng.chance(2) {
                let command = self.random_command();
                self.create(command);
            }
        }
        self.opened = Some(OpenedPack {
            pack,
            picks_left: pack.picks().min(items.len()),
            items,
            return_to: self.phase,
        });
        self.phase = Phase::Pack;
    }

    /// Takes a card of the open pack: a theme or a command is used on the spot
    /// (commands on the selected `cards` of the drawn hand), a card joins the
    /// deck, a joker the row.
    pub fn pick(&mut self, choice: usize, cards: &[usize]) -> Result<(), GameError> {
        let opened = self.opened.as_ref().ok_or(GameError::NotInPack)?;
        let item = *opened
            .items
            .get(choice)
            .ok_or_else(|| GameError::BadSelection(format!("no choice #{choice}")))?;
        match item {
            PackItem::Consumable(consumable) => {
                let targets = self.targets(consumable, cards)?;
                self.check_usable(consumable)?;
                self.apply_consumable(consumable, &targets);
            }
            PackItem::Joker { joker, edition } => {
                if !self.has_room(edition) {
                    return Err(GameError::JokerSlotsFull);
                }
                self.jokers.push(OwnedJoker::new(joker, edition));
            }
            PackItem::Card(card) => {
                self.next_card_id += 1;
                self.deck.push(card.with_id(self.next_card_id - 1));
            }
        }
        let opened = self.opened.as_mut().ok_or(GameError::NotInPack)?;
        opened.items.remove(choice);
        opened.picks_left -= 1;
        if opened.picks_left == 0 || opened.items.is_empty() {
            self.close_pack();
        }
        Ok(())
    }

    /// Leaves the open pack without taking (more of) its cards.
    pub fn skip_pack(&mut self) -> Result<(), GameError> {
        if self.opened.is_none() {
            return Err(GameError::NotInPack);
        }
        for joker in &mut self.jokers {
            joker.on_pack_skipped();
        }
        self.close_pack();
        Ok(())
    }

    fn close_pack(&mut self) {
        let return_to = self.opened.take().map_or(Phase::Shop, |o| o.return_to);
        self.hand.clear();
        self.phase = return_to;
        if !self.pending_packs.is_empty() {
            let pack = self.pending_packs.remove(0);
            self.open_pack(pack);
        }
    }

    fn opened_info(&self) -> Option<OpenedInfo> {
        let opened = self.opened.as_ref()?;
        Some(OpenedInfo {
            name: opened.pack.name(),
            description: opened.pack.description(),
            quip: opened.pack.kind.quip(),
            picks_left: opened.picks_left,
            with_hand: opened.pack.kind.with_hand(),
            choices: opened
                .items
                .iter()
                .map(|&item| match item {
                    PackItem::Consumable(c) => ChoiceInfo::Consumable(ConsumableInfo::new(c, 0)),
                    PackItem::Joker { joker, edition } => ChoiceInfo::Joker(JokerInfo::new(
                        &OwnedJoker::new(joker, edition),
                        None,
                        None,
                    )),
                    PackItem::Card(card) => ChoiceInfo::Card(CardChoice { kind: "card", card }),
                })
                .collect(),
        })
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
        for _ in 0..self.abilities(Joker::CopyOnWrite) {
            if !self.consumables.is_empty() {
                let i = self.command_rng.below(self.consumables.len() as u64) as usize;
                self.consumables.push(Held {
                    consumable: self.consumables[i].consumable,
                    transparent: true,
                });
            }
        }
        self.offers.clear();
        self.free_offers.clear();
        self.free_packs = false;
        self.packs.clear();
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
                self.roll_boss();
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
        if cheat.random_pack {
            self.in_shop()?;
            let pack = self.roll_pack();
            self.open_pack(pack);
        }
        if let Some(boss) = cheat.boss {
            self.boss = boss;
        }
        for tag in cheat.tags {
            self.gain_tag(tag);
        }
        for voucher in cheat.vouchers {
            if !self.has(voucher) {
                self.merge(voucher);
            }
        }
        if cheat.next_boss {
            let i = Boss::ALL.iter().position(|&b| b == self.boss).unwrap_or(0);
            self.boss = Boss::ALL[(i + 1) % Boss::ALL.len()];
        }
        if cheat.random_oligarchy {
            let card = Consumable::Oligarchy(roll_oligarchy(&mut self.command_rng));
            if !self.create(card) {
                return Err(GameError::ClipboardFull);
            }
        }
        if cheat.random_command {
            let command = self.random_command();
            if !self.create(command) {
                return Err(GameError::ClipboardFull);
            }
        }
        if cheat.random_theme {
            if !self.clipboard_room() {
                return Err(GameError::ClipboardFull);
            }
            let mut rng = Rng::stream(self.seed ^ self.consumables.len() as u64, "cheat-theme");
            let theme = Theme::ALL[rng.below(9) as usize];
            self.consumables.push(Consumable::Theme(theme).into());
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
                Phase::Won | Phase::Shop | Phase::Pack => BlindStatus::Defeated,
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
                .map(|&kind| {
                    let boss = (kind == BlindKind::Boss).then_some(self.boss);
                    BlindInfo {
                        kind,
                        name: boss.map_or(kind.name(), Boss::name),
                        description: boss.map_or("", Boss::description),
                        quip: boss.map_or(kind.quip(), Boss::quip),
                        target: self.target_of(kind),
                        reward: boss.map_or(kind.reward(), Boss::reward),
                        status: self.blind_status(kind),
                        tag: match kind {
                            BlindKind::Small => Some(self.blind_tags[0].into()),
                            BlindKind::Big => Some(self.blind_tags[1].into()),
                            BlindKind::Boss => None,
                        },
                    }
                })
                .collect(),
            target: self.target(),
            score: self.score,
            hands_left: self.hands_left,
            discards_left: self.discards_left,
            phase: self.phase,
            victory: self.victory,
            hand: self.hand.clone(),
            face_down: self
                .hand
                .iter()
                .map(|c| self.face_down.contains(&c.id))
                .collect(),
            debuffed: self.hand.iter().map(|c| self.debuffed(c)).collect(),
            forced: self
                .forced
                .and_then(|id| self.hand.iter().position(|c| c.id == id)),
            boss: BossInfo {
                id: self.boss,
                name: self.boss.name(),
                description: self.boss.description(),
                quip: self.boss.quip(),
            },
            boss_active: self.boss_effect().is_some(),
            jokers_hidden: self.jokers_hidden,
            disabled_joker: self.disabled_joker,
            vouchers: self
                .vouchers
                .iter()
                .map(|&v| VoucherInfo::from(v))
                .collect(),
            vouchers_for_sale: if self.phase == Phase::Shop {
                self.voucher_offers
                    .iter()
                    .map(|&v| VoucherInfo::from(v))
                    .collect()
            } else {
                Vec::new()
            },
            boss_reroll_cost: self.boss_reroll_cost(),
            tags: self.tags.iter().map(|&t| t.into()).collect(),
            hand_size: self.hand_size(),
            remaining: {
                let mut cards = match self.phase {
                    Phase::Playing | Phase::Won | Phase::Lost => self.draw_pile.clone(),
                    Phase::BlindSelect | Phase::Shop | Phase::Pack => self.deck.clone(),
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
                .map(|held| ConsumableInfo {
                    transparent: held.transparent,
                    ..ConsumableInfo::new(held.consumable, held.consumable.price())
                })
                .collect(),
            consumable_slots: self.consumable_slots(),
            unique_themes: self.themes_used.len() as u32,
            modifiers: Modifiers::catalog(),
            packs: self
                .packs
                .iter()
                .map(|&pack| PackInfo {
                    pack,
                    name: pack.name(),
                    description: pack.description(),
                    quip: pack.kind.quip(),
                    price: self.pack_price(pack),
                })
                .collect(),
            opened: self.opened_info(),
            shop: self
                .offers
                .iter()
                .enumerate()
                .map(|(i, &o)| match o {
                    Offer::Joker { joker, edition } => {
                        let owned = OwnedJoker::new(joker, edition);
                        OfferInfo::Joker(JokerInfo {
                            price: self.offer_price(i),
                            ..JokerInfo::new(&owned, None, None)
                        })
                    }
                    Offer::Consumable(c) => {
                        OfferInfo::Consumable(ConsumableInfo::new(c, self.offer_price(i)))
                    }
                    Offer::Card(card) => OfferInfo::Card(CardOffer {
                        kind: "card",
                        card,
                        price: self.offer_price(i),
                    }),
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
    use crate::joker::Effect;
    use crate::score::Source;

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
                badges: 0,
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
            Offer::Consumable(_) | Offer::Card(_) => None,
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
        game.consumables = vec![Consumable::Command(command).into()];
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
        game.consumables = vec![Consumable::Command(Command::Rsync).into()];
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
        game.consumables
            .insert(0, Consumable::Theme(Theme::Nord).into());
        game.use_consumable(0, &[]).unwrap();
        game.use_consumable(0, &[]).unwrap();
        assert_eq!(
            game.consumables,
            [Held::from(Consumable::Theme(Theme::Nord))]
        );
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
        assert!(matches!(
            game.consumables[..],
            [Held {
                consumable: Consumable::Command(_),
                ..
            }]
        ));
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

    /// A game in the shop, with money.
    fn shopping(seed: u64, jokers: Vec<Joker>) -> Game {
        let mut game = won_game(seed, jokers);
        game.cash_out().unwrap();
        game.money = 50;
        game
    }

    fn pack(kind: PackKind, size: crate::pack::PackSize) -> Pack {
        Pack { kind, size }
    }

    #[test]
    fn the_shop_sells_two_packs_that_rerolls_keep() {
        let mut game = shopping(60, vec![]);
        let packs = game.packs.clone();
        assert_eq!(packs.len(), SHOP_PACKS);
        game.reroll().unwrap();
        assert_eq!(game.packs, packs);
        assert_eq!(game.state().packs.len(), 2);
    }

    #[test]
    fn coreutils_draws_a_hand_for_its_commands() {
        use crate::pack::PackSize;
        let mut game = shopping(61, vec![]);
        game.packs = vec![pack(PackKind::Coreutils, PackSize::Normal)];
        game.buy_pack(0).unwrap();
        assert_eq!(game.money, 46);
        let s = game.state();
        assert_eq!(s.phase, Phase::Pack);
        assert_eq!(
            (s.hand.len(), s.opened.as_ref().unwrap().choices.len()),
            (HAND_SIZE, 3)
        );
        assert!(game.play(&[0]).is_err());
        // Swap the first choice for a command that needs one card.
        game.opened.as_mut().unwrap().items[0] =
            PackItem::Consumable(Consumable::Command(Command::Strip));
        assert!(game.pick(0, &[]).is_err());
        let id = game.hand[0].id;
        game.pick(0, &[0]).unwrap();
        assert!(
            game.deck
                .iter()
                .find(|c| c.id == id)
                .unwrap()
                .is(Enhancement::BinaryBlob)
        );
        assert_eq!(game.phase, Phase::Shop);
        assert!(game.hand.is_empty() && game.opened.is_none());
    }

    #[test]
    fn base_devel_and_jokers_git() {
        use crate::pack::PackSize;
        let mut game = shopping(62, vec![]);
        game.packs = vec![
            pack(PackKind::BaseDevel, PackSize::Meta),
            pack(PackKind::JokersGit, PackSize::Normal),
        ];
        game.buy_pack(0).unwrap();
        assert_eq!(game.state().opened.unwrap().choices.len(), 5);
        game.pick(0, &[]).unwrap();
        assert_eq!(game.phase, Phase::Pack);
        game.pick(0, &[]).unwrap();
        assert_eq!((game.phase, game.deck.len()), (Phase::Shop, 54));
        game.buy_pack(0).unwrap();
        assert_eq!(game.state().opened.unwrap().choices.len(), 2);
        game.pick(1, &[]).unwrap();
        assert_eq!(game.jokers.len(), 1);
    }

    #[test]
    fn skipping_a_pack_grows_minimal_install() {
        use crate::pack::PackSize;
        let mut game = shopping(63, vec![Joker::MinimalInstall, Joker::FreeAsInBeer]);
        game.packs = vec![pack(PackKind::Base16, PackSize::Full)];
        game.money = 0;
        game.buy_pack(0).unwrap();
        game.skip_pack().unwrap();
        assert_eq!((game.phase, game.jokers[0].counter), (Phase::Shop, 3.0));
        assert_eq!(game.skip_pack(), Err(GameError::NotInPack));
    }

    /// A game in its first blind with this oligarchy card in the clipboard.
    fn with_oligarchy(seed: u64, card: Oligarchy, jokers: Vec<Joker>) -> Game {
        let mut game = playing(seed, jokers);
        game.consumables = vec![Consumable::Oligarchy(card).into()];
        game.hand = cards("KH KD 9S 7C 5H 4D 3S 2C");
        game.deck = game.hand.clone();
        game
    }

    #[test]
    fn oligarchy_cards_destroy_and_add_cards() {
        let mut game = with_oligarchy(70, Oligarchy::BoardAppointment, vec![Joker::ThePrimeagen]);
        game.use_consumable(0, &[]).unwrap();
        assert_eq!((game.hand.len(), game.deck.len()), (10, 10));
        assert_eq!(
            game.hand.iter().filter(|c| c.enhancement.is_some()).count(),
            3
        );

        let mut game = with_oligarchy(71, Oligarchy::MassLayoffs, vec![]);
        game.money = 0;
        game.use_consumable(0, &[]).unwrap();
        assert_eq!((game.hand.len(), game.deck.len(), game.money), (3, 3, 20));

        let mut game = with_oligarchy(72, Oligarchy::StockSplit, vec![]);
        game.use_consumable(0, &[0]).unwrap();
        assert_eq!(
            game.deck
                .iter()
                .filter(|c| c.rank == 13 && c.suit == Suit::Hearts)
                .count(),
            3
        );
    }

    #[test]
    fn oligarchy_cards_need_a_hand_and_the_right_targets() {
        let mut game = with_oligarchy(73, Oligarchy::GoldenHandshake, vec![]);
        assert!(game.use_consumable(0, &[]).is_err());
        game.use_consumable(0, &[2]).unwrap();
        assert_eq!(game.hand[2].seal, Some(Seal::GoldStar));

        let mut game = won_game(74, vec![]);
        game.cash_out().unwrap();
        game.consumables = vec![Consumable::Oligarchy(Oligarchy::Monopoly).into()];
        assert!(game.use_consumable(0, &[]).is_err());
    }

    #[test]
    fn consolidation_shrinks_the_hand() {
        let mut game = with_oligarchy(75, Oligarchy::Consolidation, vec![]);
        game.use_consumable(0, &[]).unwrap();
        let rank = game.hand[0].rank;
        assert!(game.hand.iter().all(|c| c.rank == rank));
        assert_eq!(game.hand_size(), HAND_SIZE - 1);
    }

    #[test]
    fn oligarchy_cards_on_jokers() {
        let mut game = with_oligarchy(
            76,
            Oligarchy::Merger,
            vec![Joker::Dotfiles, Joker::Hotwire, Joker::Omakase],
        );
        game.use_consumable(0, &[]).unwrap();
        assert_eq!(game.jokers.len(), 2);
        assert_eq!(game.jokers[0].joker, game.jokers[1].joker);

        let mut game = with_oligarchy(
            77,
            Oligarchy::Restructuring,
            vec![Joker::Dotfiles, Joker::Hotwire],
        );
        game.use_consumable(0, &[]).unwrap();
        assert_eq!(game.jokers.len(), 1);
        assert_eq!(game.jokers[0].edition, Some(Edition::Animated));

        let mut game = with_oligarchy(78, Oligarchy::AspiringBillionaire, vec![]);
        game.use_consumable(0, &[]).unwrap();
        assert_eq!(game.jokers[0].joker.rarity(), Rarity::Legendary);

        let mut game = with_oligarchy(79, Oligarchy::LeveragedBuyout, vec![]);
        game.use_consumable(0, &[]).unwrap();
        assert_eq!(
            (game.jokers[0].joker.rarity(), game.money),
            (Rarity::Aur, 0)
        );

        let mut game = with_oligarchy(80, Oligarchy::PraiseTheOligarchy, vec![]);
        game.use_consumable(0, &[]).unwrap();
        assert!(HandKind::ALL.iter().all(|&k| game.levels.level(k) == 2));
    }

    #[test]
    fn legendaries_grow() {
        let mut game = with_command(
            81,
            Command::RmRf,
            vec![Joker::ThePrimeagen, Joker::MailingList],
        );
        game.use_consumable(0, &[0, 1]).unwrap();
        assert_eq!(game.jokers[0].counter, 3.0);
        game.hand = cards("2H 3D 4S 5C 6H 7D 8S 9C");
        for _ in 0..3 {
            game.discard(&[0, 1, 2, 3, 4]).unwrap();
        }
        assert_eq!((game.jokers[1].counter, game.jokers[1].progress), (1.0, 15));
        game.jokers[1].on_cards_discarded(10);
        let s = game.state();
        assert_eq!(s.jokers[1].bonus.as_ref().unwrap().text, "×2 mult · 2/23");
    }

    #[test]
    fn dhh_doubles_kings_and_queens() {
        let mut game = playing(82, vec![Joker::Dhh]);
        game.hand = cards("KH QD 9S 7C 5H 4D 3S 2C");
        // Kings and queens are not a pair: high card king, which DHH doubles.
        let b = game.play(&[0]).unwrap();
        assert_eq!(b.mult, 2.0);
    }

    #[test]
    fn copy_on_write_adds_a_transparent_consumable() {
        let mut game = shopping(83, vec![Joker::CopyOnWrite]);
        game.consumables = vec![
            Consumable::Theme(Theme::Nord).into(),
            Consumable::Theme(Theme::Nord).into(),
        ];
        game.next_blind().unwrap();
        assert_eq!(game.consumables.len(), 3);
        assert!(game.consumables[2].transparent);
        assert!(!game.clipboard_room());
        assert!(game.state().consumables[2].transparent);
    }

    #[test]
    fn seed_round_and_angel_investor_create_oligarchy_cards() {
        let mut game = playing(84, vec![Joker::SeedRound]);
        game.hand = cards("9S 8S 7S 6S 5S 4D 3S 2C");
        game.play(&[0, 1, 2, 3, 4]).unwrap();
        assert!(matches!(
            game.consumables[0].consumable,
            Consumable::Oligarchy(_)
        ));

        let mut game = playing(85, vec![Joker::AngelInvestor]);
        game.hand = cards("6H 5D 4S 3C 2H 9D 8S 7C");
        game.deck = game.hand.clone();
        game.play(&[0]).unwrap();
        assert!(matches!(
            game.consumables[0].consumable,
            Consumable::Oligarchy(_)
        ));
        assert!(game.deck.iter().all(|c| c.rank != 6));
    }

    #[test]
    fn oligarchy_packs_draw_a_hand() {
        use crate::pack::PackSize;
        let mut game = shopping(86, vec![]);
        game.packs = vec![pack(PackKind::Oligarchy, PackSize::Full)];
        game.buy_pack(0).unwrap();
        let s = game.state();
        let opened = s.opened.unwrap();
        assert!(opened.with_hand);
        assert_eq!((opened.choices.len(), s.hand.len()), (4, HAND_SIZE));
    }

    /// A game in the boss blind of ante 1, against this boss.
    fn boss_game(seed: u64, boss: Boss, jokers: Vec<Joker>) -> Game {
        let mut game = Game::new(seed, jokers);
        game.cheat(Cheat {
            blind: Some(BlindKind::Boss),
            boss: Some(boss),
            ..Cheat::default()
        })
        .unwrap();
        game.select_blind().unwrap();
        game
    }

    #[test]
    fn bosses_follow_the_ante_and_do_not_repeat() {
        let mut game = Game::new(90, vec![]);
        assert!(!game.boss.is_final() && game.boss.min_ante() <= 1);
        game.ante = WINNING_ANTE;
        game.roll_boss();
        assert!(game.boss.is_final());
        let mut game = Game::new(91, vec![]);
        game.ante = 7;
        // The ante 1 boss counts as met: 22 more before any repeat.
        let mut seen = std::collections::HashSet::from([game.boss]);
        for _ in 0..22 {
            game.roll_boss();
            assert!(seen.insert(game.boss), "{:?} repeated", game.boss);
        }
    }

    #[test]
    fn suit_bosses_keep_cards_from_scoring() {
        let mut game = boss_game(92, Boss::Burnout, vec![]);
        game.hand = cards("KH KS 9S 7C 5H 4D 3S 2C");
        assert_eq!(game.state().debuffed[..2], [true, false]);
        let b = game.play(&[0, 1]).unwrap();
        // The heart king does not score: pair base 10 + 10 chips × 2.
        assert_eq!(b.total, 40.0);
        assert_eq!(b.steps[0].effect, Effect::Debuffed);
    }

    #[test]
    fn face_down_cards_hide_the_preview() {
        let game = boss_game(93, Boss::BlackScreen, vec![]);
        assert!(game.state().face_down.iter().all(|&f| f));
        assert_eq!(game.preview(&[0, 1]).unwrap(), None);
    }

    #[test]
    fn refused_hands_score_nothing() {
        let mut game = boss_game(94, Boss::MandatoryMeeting, vec![]);
        let b = game.play(&[0, 1]).unwrap();
        assert_eq!(
            (b.total, b.notes[0].as_str()),
            (0.0, "Not allowed by The Mandatory Meeting.")
        );
        assert_eq!(game.hands_left, HANDS_PER_BLIND - 1);

        let mut game = boss_game(95, Boss::CodeReviewer, vec![]);
        game.hand = cards("KH KS 9S 9C 5H 5D 3S 2C");
        assert!(game.play(&[0, 1]).unwrap().total > 0.0);
        assert_eq!(game.play(&[0, 1]).unwrap().total, 0.0);
    }

    #[test]
    fn deadline_read_only_and_kubernetes() {
        let game = boss_game(96, Boss::Deadline, vec![]);
        assert_eq!((game.hands_left, game.target()), (1, 300.0));
        let game = boss_game(97, Boss::ReadOnlyFilesystem, vec![]);
        assert_eq!(game.discards_left, 0);
        let game = boss_game(98, Boss::KubernetesCluster, vec![]);
        assert_eq!(game.state().blinds[2].target, 1200.0);
    }

    #[test]
    fn oom_killer_and_rate_limiter_shrink_the_hand() {
        let mut game = boss_game(99, Boss::OomKiller, vec![]);
        let left = game.draw_pile.len();
        game.play(&[0]).unwrap();
        // 2 cards discarded, then the hand refilled: 3 more cards drawn.
        assert_eq!(
            (game.hand.len(), game.draw_pile.len()),
            (HAND_SIZE, left - 3)
        );

        let mut game = boss_game(100, Boss::RateLimiter, vec![]);
        game.play(&[0, 1, 2, 3, 4]).unwrap();
        assert_eq!(game.hand.len(), HAND_SIZE - 5 + 3);
    }

    #[test]
    fn cloud_bill_and_incident_commander() {
        let mut game = boss_game(101, Boss::CloudBill, vec![Joker::IncidentCommander]);
        game.money = 30;
        game.hand = cards("KH KS 9S 7C 5H 4D 3S 2C");
        let b = game.play(&[0, 1]).unwrap();
        assert_eq!(game.money, 8);
        assert!(b.notes.iter().any(|n| n.contains("Cloud Bill")));
    }

    #[test]
    fn clippy_keeps_a_card_selected() {
        let mut game = boss_game(102, Boss::Clippy, vec![]);
        let forced = game.state().forced.unwrap();
        let other = (forced + 1) % HAND_SIZE;
        let b = game.play(&[other]).unwrap();
        assert_eq!(b.cards.len(), 2);
        assert!(game.state().forced.is_some());
    }

    #[test]
    fn windows_update_disables_a_joker() {
        let mut game = boss_game(103, Boss::WindowsUpdate, vec![Joker::Omakase]);
        assert_eq!(game.state().disabled_joker, Some(0));
        let b = game.play(&[0]).unwrap();
        assert!(
            b.steps
                .iter()
                .all(|s| s.source != Source::Joker { index: 0 })
        );
    }

    #[test]
    fn vendor_lock_in_until_a_joker_is_sold() {
        let mut game = boss_game(104, Boss::VendorLockIn, vec![Joker::Dotfiles]);
        assert!(game.state().debuffed.iter().all(|&d| d));
        game.sell(0).unwrap();
        assert!(game.state().debuffed.iter().all(|&d| !d));
    }

    #[test]
    fn kill_9_and_rollback_disable_bosses() {
        let game = boss_game(105, Boss::ReadOnlyFilesystem, vec![Joker::Kill9]);
        assert_eq!(game.discards_left, DISCARDS_PER_BLIND);
        assert!(!game.state().boss_active);

        let mut game = boss_game(106, Boss::Burnout, vec![Joker::Rollback]);
        assert!(game.state().boss_active);
        game.sell(0).unwrap();
        assert!(!game.state().boss_active);
        game.hand = cards("KH KS 9S 7C 5H 4D 3S 2C");
        assert_eq!(game.play(&[0, 1]).unwrap().total, 60.0);
    }

    fn merge(game: &mut Game, vouchers: &[Voucher]) {
        game.cheat(Cheat {
            vouchers: vouchers.to_vec(),
            ..Cheat::default()
        })
        .unwrap();
    }

    #[test]
    fn one_pull_request_per_ante_and_follow_ups_wait() {
        let mut game = shopping(110, vec![]);
        let offer = game.state().vouchers_for_sale[0].clone();
        assert!(offer.id.follows().is_none());
        game.buy_voucher(0).unwrap();
        assert_eq!(
            (game.money, game.state().vouchers_for_sale.len()),
            (50 - VOUCHER_PRICE, 0)
        );
        assert!(game.buy_voucher(0).is_err());
        for _ in 0..200 {
            game.roll_voucher();
            let v = game.voucher_offers[0];
            assert!(v.follows().is_none_or(|first| game.has(first)));
            assert!(!game.has(v));
        }
    }

    #[test]
    fn pull_requests_change_the_rules() {
        let mut game = Game::new(111, vec![]);
        merge(
            &mut game,
            &[
                Voucher::Overtime,
                Voucher::RmI,
                Voucher::Hidpi,
                Voucher::ClipboardManager,
                Voucher::LoremIpsum,
                Voucher::FeatureFlag,
                Voucher::HighYieldSavings,
            ],
        );
        game.select_blind().unwrap();
        let s = game.state();
        assert_eq!((s.hands_left, s.discards_left, s.hand_size), (5, 4, 9));
        assert_eq!((s.consumable_slots, s.max_jokers), (3, 6));
        game.money = 100;
        assert_eq!(game.reward().interest, 10);
    }

    #[test]
    fn discounts_and_rerolls() {
        let mut game = shopping(112, vec![]);
        merge(&mut game, &[Voucher::EarlyBird, Voucher::MirrorList]);
        game.offers = vec![Offer::Joker {
            joker: Joker::Fork,
            edition: None,
        }];
        assert_eq!(game.reroll_cost, REROLL_COST - 2);
        let s = game.state();
        let OfferInfo::Joker(fork) = &s.shop[0] else {
            panic!()
        };
        assert_eq!(fork.price, 8);
        game.buy(0).unwrap();
        assert_eq!(game.money, 42);
        merge(&mut game, &[Voucher::LifetimeDeal]);
        assert_eq!(game.discounted(10), 5);
    }

    #[test]
    fn more_workspaces_and_card_marketplace() {
        let mut game = shopping(113, vec![]);
        merge(
            &mut game,
            &[
                Voucher::MoreWorkspaces,
                Voucher::CardMarketplace,
                Voucher::PremiumCards,
            ],
        );
        game.reroll().unwrap();
        assert_eq!(game.offers.len(), SHOP_OFFERS + 1);
        let mut cards = 0;
        for _ in 0..200 {
            game.roll_offers();
            cards += game
                .offers
                .iter()
                .filter(|o| matches!(o, Offer::Card(_)))
                .count();
        }
        assert!(cards > 0);
        game.offers = vec![Offer::Card(
            Card::new(5, Suit::Hearts).with_seal(Seal::Retry),
        )];
        game.buy(0).unwrap();
        assert_eq!(game.deck.len(), 53);
    }

    #[test]
    fn git_revert_goes_back_an_ante() {
        let mut game = Game::new(114, vec![]);
        game.ante = 3;
        merge(&mut game, &[Voucher::GitRevert]);
        assert_eq!(
            (game.ante, game.hands_per_blind()),
            (2, HANDS_PER_BLIND - 1)
        );
    }

    #[test]
    fn change_request_rerolls_the_boss() {
        let mut game = Game::new(115, vec![]);
        game.money = 50;
        assert!(game.reroll_boss().is_err());
        merge(&mut game, &[Voucher::ChangeRequest]);
        let before = game.boss;
        game.reroll_boss().unwrap();
        assert_ne!(game.boss, before);
        assert_eq!(game.money, 40);
        assert!(game.reroll_boss().is_err());
        merge(&mut game, &[Voucher::EmergencyChange]);
        game.reroll_boss().unwrap();
    }

    #[test]
    fn theme_showcase_multiplies_matching_hands() {
        let mut game = playing(116, vec![]);
        merge(&mut game, &[Voucher::ThemePreview, Voucher::ThemeShowcase]);
        game.consumables = vec![Consumable::Theme(Theme::RosePine).into()];
        game.hand = cards("KH KD 9S 7C 5H 4D 3S 2C");
        // Pair: (10 + 20) × 2 × 1.5
        assert_eq!(game.play(&[0, 1]).unwrap().total, 90.0);
    }

    #[test]
    fn skipping_gives_the_blind_badge() {
        let mut game = Game::new(120, vec![Joker::SkipIntro]);
        let tag = game.blind_tags[0];
        let shown = game.state().blinds[0].tag.clone().unwrap();
        assert_eq!(shown.id, tag);
        game.blind_tags[0] = Tag::Sponsors2x;
        game.money = 10;
        game.skip_blind().unwrap();
        assert_eq!(
            (game.money, game.skips, game.jokers[0].counter),
            (20, 1, 1.25)
        );
    }

    #[test]
    fn badges_wait_for_their_moment() {
        let mut game = Game::new(121, vec![]);
        game.cheat(Cheat {
            tags: vec![
                Tag::Resolution4k,
                Tag::Stars10k,
                Tag::PriceFree,
                Tag::SeriesA,
            ],
            ..Cheat::default()
        })
        .unwrap();
        assert_eq!(game.state().tags.len(), 4);
        game.select_blind().unwrap();
        assert_eq!(game.hand.len(), HAND_SIZE + 3);
        win(&mut game);
        game.cash_out().unwrap();
        // The extra AUR joker is offered, and everything is free.
        let s = game.state();
        assert_eq!(s.shop.len(), SHOP_OFFERS + 1);
        assert!(s.packs.iter().all(|p| p.price == 0));
        assert!(game.free_offers.iter().all(|&f| f));
        assert_eq!(
            game.state().tags.iter().map(|t| t.id).collect::<Vec<_>>(),
            [Tag::SeriesA]
        );
        game.next_blind().unwrap();
        assert!(game.packs.is_empty() && !game.free_packs);
    }

    #[test]
    fn series_a_pays_after_the_boss() {
        let mut game = boss_game(122, Boss::Clippy, vec![Joker::Kill9]);
        game.gain_tag(Tag::SeriesA);
        win(&mut game);
        let reward = game.state().reward.unwrap();
        assert_eq!(reward.badges, 25);
    }

    #[test]
    fn badges_2x_doubles_the_next_badge() {
        let mut game = Game::new(123, vec![]);
        game.money = 0;
        game.gain_tag(Tag::Badges2x);
        game.skips = 2;
        game.gain_tag(Tag::SpeedBlazing);
        assert_eq!(game.money, 20);
        assert!(game.tags.is_empty());
    }

    #[test]
    fn pack_badges_open_a_free_pack_then_come_back() {
        let mut game = Game::new(124, vec![]);
        game.blind_tags[0] = Tag::BuildPassing;
        game.ante = 2;
        game.skip_blind().unwrap();
        let s = game.state();
        assert_eq!(
            (s.phase, s.opened.unwrap().name),
            (Phase::Pack, "base-devel-meta".to_string())
        );
        game.skip_pack().unwrap();
        assert_eq!(game.phase, Phase::BlindSelect);
    }

    #[test]
    fn edition_badges_decorate_a_shop_joker() {
        let mut game = won_game(125, vec![]);
        game.gain_tag(Tag::BlurOn);
        game.cash_out().unwrap();
        let free = game
            .offers
            .iter()
            .zip(&game.free_offers)
            .find(|(_, free)| **free)
            .map(|(o, _)| *o)
            .unwrap();
        assert!(matches!(
            free,
            Offer::Joker {
                edition: Some(Edition::Blur),
                ..
            }
        ));
    }

    #[test]
    fn shop_actions_need_the_shop() {
        let mut game = playing(9, vec![Joker::Dotfiles]);
        assert_eq!(game.buy(0), Err(GameError::NotInShop));
        assert_eq!(game.reroll(), Err(GameError::NotInShop));
        // Jokers may be sold during a blind (Rollback, The Vendor Lock-in).
        assert_eq!(game.sell(0), Ok(()));
    }
}
