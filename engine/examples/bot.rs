//! A bot that plays whole games through the JSON protocol, on many seeds, to
//! find crashes, stuck games and broken invariants, and to get a first look at
//! the balance (wins per deck, joker and boss).
//!
//! cargo run --release -p omatro-engine --example bot -- [games] [first seed] [--endless] [--chaos]
//!
//! `--endless` goes on after the winning ante; `--chaos` uses the debug
//! commands to reach further: money and a random joker at each shop, random
//! commands, oligarchy cards and card modifiers; `--save` saves the game after
//! each command and, every 25 commands, resumes it in a new session, which
//! must find the very same state.

use omatro_engine::protocol::Session;
use omatro_engine::rng::Rng;
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};
use std::panic::{self, AssertUnwindSafe};
use std::sync::Mutex;
use std::time::Instant;

/// Commands a game may take before the bot calls it stuck.
const MAX_STEPS: usize = 20_000;
/// Endless mode stops at this ante.
const ENDLESS_LIMIT: u64 = 14;

/// What became of one game.
#[derive(Default)]
struct Outcome {
    seed: u64,
    deck: String,
    won: bool,
    ante: u64,
    /// The boss of the ante the game was lost at, when lost at a boss blind.
    lost_to: Option<String>,
    /// Jokers held at the end.
    jokers: Vec<String>,
    /// Command name and error message, for every refused command.
    errors: Vec<(String, String)>,
    /// Broken invariants.
    violations: Vec<String>,
    stuck: bool,
    steps: usize,
}

struct Bot {
    session: Session,
    rng: Rng,
    state: Value,
    endless: bool,
    chaos: bool,
    /// `--save`: the save file of this game.
    save: Option<std::path::PathBuf>,
    out: Outcome,
}

/// Commands between two resumed sessions, with `--save`.
const RESUME_EVERY: usize = 25;

/// A session as the bot plays it: with the debug commands in chaos mode.
fn new_session(chaos: bool, save: &Option<std::path::PathBuf>) -> Session {
    let session = if chaos {
        Session::from_env()
    } else {
        Session::default()
    };
    match save {
        Some(path) => session.with_save(path.clone()),
        None => session,
    }
}

/// `--trace`: prints every command (but previews) on stderr.
static TRACE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// A panic's message and where it happened, kept by the panic hook.
static PANIC_AT: Mutex<String> = Mutex::new(String::new());

impl Bot {
    /// Sends a command; a refusal is recorded as an error.
    fn send(&mut self, command: Value) -> Value {
        let response = self.probe(command.clone());
        if response["event"] == "error" {
            let name = command["cmd"].as_str().unwrap_or("?").to_string();
            let message = response["message"].as_str().unwrap_or("").to_string();
            self.out.errors.push((name, message));
        }
        response
    }

    /// Sends a command the bot is only trying (random targets, pack picks):
    /// a refusal is fine.
    fn probe(&mut self, command: Value) -> Value {
        // Previews change nothing: they don't count as steps.
        if command["cmd"] != "preview" {
            self.out.steps += 1;
        }
        let response = serde_json::to_value(self.session.handle(&command.to_string()))
            .expect("responses serialize");
        if let Some(state) = response.get("state") {
            self.state = state.clone();
            self.check(&command);
        }
        if self.save.is_some()
            && command["cmd"] != "preview"
            && self.out.steps % RESUME_EVERY == 0
            && self.state["phase"] != "lost"
        {
            self.resume_elsewhere(&command);
        }
        if TRACE.load(std::sync::atomic::Ordering::Relaxed) && command["cmd"] != "preview" {
            eprintln!(
                "{command} -> {} {} | ante {} {} hand {} pile {} hands {} discards {}",
                response["event"],
                response["message"].as_str().unwrap_or(""),
                self.state["ante"],
                self.state["phase"],
                self.len("hand"),
                self.len("remaining"),
                self.state["hands_left"],
                self.state["discards_left"],
            );
        }
        response
    }

    /// Resumes the saved game in a new session, as the app does when it
    /// starts again, and checks it finds the state it left.
    fn resume_elsewhere(&mut self, after: &Value) {
        let mut session = new_session(self.chaos, &self.save);
        let resumed = serde_json::to_value(session.handle(r#"{"cmd":"resume"}"#)).unwrap();
        let left = serde_json::to_value(self.session.handle(r#"{"cmd":"state"}"#)).unwrap();
        if resumed["state"] != left["state"] {
            let field = left["state"]
                .as_object()
                .and_then(|o| o.keys().find(|k| resumed["state"][k.as_str()] != left["state"][k.as_str()]))
                .cloned()
                .unwrap_or_else(|| format!("{}", resumed["message"]));
            self.out
                .violations
                .push(format!("resumed game differs in {field} (after {})", after["cmd"]));
        }
        self.session = session;
    }

    fn phase(&self) -> &str {
        self.state["phase"].as_str().unwrap_or("")
    }

    fn money(&self) -> i64 {
        self.state["money"].as_i64().unwrap_or(0)
    }

    fn len(&self, field: &str) -> usize {
        self.state[field].as_array().map_or(0, Vec::len)
    }

    /// Invariants every state must keep.
    fn check(&mut self, command: &Value) {
        let s = &self.state;
        let mut broken = Vec::new();
        let money = s["money"].as_i64().unwrap_or(0);
        let credit = s["credit"].as_i64().unwrap_or(0);
        if money < -credit {
            broken.push(format!("money {money} below the credit of {credit}"));
        }
        if let Some(score) = s["score"].as_f64()
            && !score.is_finite()
        {
            broken.push(format!("score {score}"));
        }
        let mut ids = HashSet::new();
        for card in s["hand"]
            .as_array()
            .into_iter()
            .flatten()
            .chain(s["remaining"].as_array().into_iter().flatten())
        {
            if s["phase"] == "playing" && !ids.insert(card["id"].as_u64()) {
                broken.push(format!("card id {} twice in hand and draw pile", card["id"]));
            }
        }
        let jokers = s["jokers"].as_array().map_or(0, Vec::len);
        let slots = s["max_jokers"].as_u64().unwrap_or(0) as usize;
        if jokers > slots {
            broken.push(format!("{jokers} jokers for {slots} slots"));
        }
        let held = s["consumables"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|c| c["transparent"] != true)
            .count();
        let clipboard = s["consumable_slots"].as_u64().unwrap_or(0) as usize;
        if held > clipboard {
            broken.push(format!("{held} consumables for {clipboard} slots"));
        }
        if s["phase"] == "playing" && s["hand"].as_array().is_some_and(Vec::is_empty) {
            broken.push("playing with an empty hand".into());
        }
        let after = command["cmd"].as_str().unwrap_or("?");
        self.out
            .violations
            .extend(broken.into_iter().map(|b| format!("{b} (after {after})")));
    }

    /// Chips a card would score on its own (no enhancement but its own).
    fn card_chips(card: &Value) -> f64 {
        let enhancement = card["enhancement"].as_str().unwrap_or("");
        let rank = card["rank"].as_u64().unwrap_or(0);
        let base = if enhancement == "binary_blob" {
            50
        } else {
            match rank {
                11..=13 => 10,
                14 => 11,
                r => r,
            }
        };
        let optimized = if enhancement == "optimized" { 30 } else { 0 };
        (base + optimized + card["bonus_chips"].as_u64().unwrap_or(0)) as f64
    }

    /// The best selection by a rough estimate (hand base, scoring cards'
    /// chips), as the preview tells it.
    fn best_play(&mut self, sizes: &[usize]) -> Option<(Vec<usize>, Vec<usize>, f64)> {
        let hand = self.state["hand"].as_array()?.clone();
        let n = hand.len();
        // The Clippy: its card is always part of the selection.
        let forced = self.state["forced"].as_u64().map(|i| i as usize);
        let mut best: Option<(Vec<usize>, Vec<usize>, f64)> = None;
        for mask in 1u32..(1 << n.min(16)) {
            let size = mask.count_ones() as usize;
            if !sizes.contains(&size) {
                continue;
            }
            let cards: Vec<usize> = (0..n).filter(|i| mask & (1 << i) != 0).collect();
            if forced.is_some_and(|f| !cards.contains(&f)) {
                continue;
            }
            let preview = self.probe(json!({"cmd": "preview", "cards": cards}));
            if preview["hand"].is_null() {
                continue;
            }
            let scoring: Vec<usize> = preview["scoring"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(Value::as_u64)
                .map(|i| i as usize)
                .collect();
            let chips = preview["base_chips"].as_f64().unwrap_or(0.0)
                + scoring.iter().map(|&i| Self::card_chips(&hand[i])).sum::<f64>();
            let estimate = chips * preview["base_mult"].as_f64().unwrap_or(1.0);
            if best.as_ref().is_none_or(|b| estimate > b.2) {
                best = Some((cards, scoring, estimate));
            }
        }
        best
    }

    /// `n` distinct random hand indices.
    fn random_cards(&mut self, n: usize) -> Vec<usize> {
        let mut all: Vec<usize> = (0..self.len("hand")).collect();
        self.rng.shuffle(&mut all);
        all.truncate(n);
        all
    }

    /// Uses what can be used from the clipboard (themes right away, commands
    /// on random cards during a blind).
    fn use_consumables(&mut self) {
        let mut index = self.len("consumables");
        while index > 0 {
            index -= 1;
            let c = &self.state["consumables"][index];
            let min = c["min_cards"].as_u64().unwrap_or(0) as usize;
            let needs_hand = c["needs_hand"] == true;
            if needs_hand && self.phase() != "playing" {
                continue;
            }
            let cards = self.random_cards(min);
            self.probe(json!({"cmd": "use_consumable", "index": index, "cards": cards}));
            if self.phase() != "playing" && self.phase() != "shop" && self.phase() != "blind_select" {
                return;
            }
        }
    }

    fn play_blind(&mut self) {
        if self.chaos && self.rng.chance(6) {
            self.send(json!({"cmd": "debug", "enhance_hand": true}));
        }
        if self.rng.chance(8) {
            let by = if self.rng.chance(2) { "rank" } else { "suit" };
            self.send(json!({"cmd": "sort_hand", "by": by}));
        }
        if self.rng.chance(8) && self.len("hand") > 1 {
            let n = self.len("hand") as u64;
            let (from, to) = (self.rng.below(n), self.rng.below(n));
            self.send(json!({"cmd": "move_card", "from": from, "to": to}));
        }
        self.use_consumables();
        if self.phase() != "playing" {
            return;
        }
        let meeting = self.state["boss_active"] == true
            && self.state["boss"]["id"] == "mandatory_meeting";
        let sizes: &[usize] = if meeting { &[5] } else { &[1, 2, 3, 4, 5] };
        let Some((cards, scoring, estimate)) = self.best_play(sizes) else {
            // Everything face down: play blind.
            let n = 5.min(self.len("hand"));
            let cards = self.random_cards(n);
            self.send(json!({"cmd": "play", "cards": cards}));
            return;
        };
        let target = self.state["target"].as_f64().unwrap_or(f64::INFINITY);
        let score = self.state["score"].as_f64().unwrap_or(0.0);
        let hands = self.state["hands_left"].as_f64().unwrap_or(1.0).max(1.0);
        let discards = self.state["discards_left"].as_u64().unwrap_or(0);
        if discards > 0 && estimate < (target - score) / hands {
            // Throw the lowest cards that would not score.
            let hand = self.state["hand"].as_array().cloned().unwrap_or_default();
            let mut spare: Vec<usize> = (0..hand.len())
                .filter(|i| !scoring.iter().any(|&s| cards.get(s) == Some(i)))
                .collect();
            spare.sort_by(|&a, &b| Self::card_chips(&hand[a]).total_cmp(&Self::card_chips(&hand[b])));
            if let Some(f) = self.state["forced"].as_u64().map(|i| i as usize) {
                spare.retain(|&i| i != f);
                spare.truncate(4);
                spare.push(f);
            }
            spare.truncate(5);
            if !spare.is_empty() {
                self.send(json!({"cmd": "discard", "cards": spare}));
                return;
            }
        }
        self.send(json!({"cmd": "play", "cards": cards}));
    }

    /// Picks from an open pack until it closes.
    fn open_pack(&mut self) {
        for _ in 0..10 {
            if self.phase() != "pack" {
                return;
            }
            let choices = self.state["opened"]["choices"].as_array().cloned().unwrap_or_default();
            let mut picked = false;
            for (choice, item) in choices.iter().enumerate() {
                let min = item["min_cards"].as_u64().unwrap_or(0) as usize;
                let cards = self.random_cards(min);
                let r = self.probe(json!({"cmd": "pick", "choice": choice, "cards": cards}));
                if r["event"] != "error" {
                    picked = true;
                    break;
                }
            }
            if !picked {
                self.send(json!({"cmd": "skip_pack"}));
            }
        }
    }

    fn shop(&mut self) {
        if self.chaos {
            let money = self.money() + 40;
            let room = self.len("jokers") < self.state["max_jokers"].as_u64().unwrap_or(0) as usize;
            self.send(json!({"cmd": "debug", "money": money, "random_joker": room}));
            let clipboard = self.state["consumable_slots"].as_u64().unwrap_or(0) as usize;
            if self.len("consumables") < clipboard {
                if self.rng.chance(3) {
                    self.send(json!({"cmd": "debug", "random_oligarchy": true}));
                } else if self.rng.chance(2) {
                    self.send(json!({"cmd": "debug", "random_command": true}));
                }
            }
        }
        self.use_consumables();
        let mut rerolls = 0;
        loop {
            // Buy the first offer that is affordable and fits.
            let offers = self.state["shop"].as_array().cloned().unwrap_or_default();
            let jokers = self.len("jokers");
            let slots = self.state["max_jokers"].as_u64().unwrap_or(0) as usize;
            let clipboard = self.len("consumables");
            let clip_slots = self.state["consumable_slots"].as_u64().unwrap_or(0) as usize;
            let buy = offers.iter().position(|o| {
                let price = o["price"].as_i64().unwrap_or(0);
                let fits = match o["kind"].as_str() {
                    Some("joker") => jokers < slots || o["edition"]["id"] == "transparent",
                    Some("card") => true,
                    _ => clipboard < clip_slots,
                };
                fits && self.money() >= price
            });
            if let Some(offer) = buy {
                self.send(json!({"cmd": "buy", "offer": offer}));
                self.use_consumables();
                continue;
            }
            // Full row: make room for a rarer joker.
            let rare = offers.iter().position(|o| {
                o["kind"] == "joker"
                    && o["rarity"] == "aur"
                    && self.money() >= o["price"].as_i64().unwrap_or(0)
            });
            if let (Some(_), true) = (rare, jokers >= slots && jokers > 0) {
                let cheapest = (0..jokers)
                    .min_by_key(|&i| self.state["jokers"][i]["sell_value"].as_u64())
                    .unwrap_or(0);
                self.send(json!({"cmd": "sell", "joker": cheapest}));
                continue;
            }
            let price = self.state["reroll_cost"].as_i64().unwrap_or(99);
            if rerolls < 2 && self.money() >= price + 10 {
                rerolls += 1;
                self.send(json!({"cmd": "reroll"}));
                continue;
            }
            break;
        }
        for i in (0..self.len("vouchers_for_sale")).rev() {
            let price = self.state["vouchers_for_sale"][i]["price"].as_i64().unwrap_or(99);
            if self.money() >= price + 5 {
                self.send(json!({"cmd": "buy_voucher", "index": i}));
            }
        }
        for i in (0..self.len("packs")).rev() {
            let price = self.state["packs"][i]["price"].as_i64().unwrap_or(99);
            if i < self.len("packs") && self.money() >= price + 4 && self.rng.chance(2) {
                self.send(json!({"cmd": "buy_pack", "pack": i}));
                self.open_pack();
            }
        }
        if self.len("jokers") > 1 && self.rng.chance(4) {
            self.send(json!({"cmd": "move_joker", "from": 0, "to": 1}));
        }
        self.send(json!({"cmd": "next_blind"}));
    }

    fn play(mut self, deck: Option<&str>) -> Outcome {
        let seed = self.out.seed;
        self.send(json!({"cmd": "new_game", "seed": seed, "deck": deck}));
        while self.out.steps < MAX_STEPS {
            match self.phase() {
                "blind_select" => {
                    let boss = self.state["blind"] == "boss";
                    if !boss && self.rng.chance(5) {
                        self.send(json!({"cmd": "skip_blind"}));
                    } else {
                        let reroll = self.state["boss_reroll_cost"].as_i64();
                        if boss && reroll.is_some_and(|c| self.money() >= c) && self.rng.chance(3) {
                            self.send(json!({"cmd": "reroll_boss"}));
                        }
                        self.send(json!({"cmd": "select_blind"}));
                    }
                }
                "playing" => self.play_blind(),
                "won" => {
                    if self.state["victory"] == true {
                        self.out.won = true;
                        if !self.endless {
                            break;
                        }
                    }
                    if self.endless && self.state["ante"].as_u64() >= Some(ENDLESS_LIMIT) {
                        break;
                    }
                    self.send(json!({"cmd": "cash_out"}));
                }
                "shop" => self.shop(),
                "pack" => self.open_pack(),
                "lost" => {
                    if self.state["blind"] == "boss" {
                        self.out.lost_to = self.state["boss"]["id"].as_str().map(String::from);
                    }
                    break;
                }
                other => {
                    self.out.violations.push(format!("unknown phase {other:?}"));
                    break;
                }
            }
        }
        self.out.stuck = self.out.steps >= MAX_STEPS;
        self.out.ante = self.state["ante"].as_u64().unwrap_or(0);
        self.out.jokers = self.state["jokers"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|j| j["id"].as_str().map(String::from))
            .collect();
        self.out
    }
}

/// Plays one game; a panic becomes a violation.
fn run_game(seed: u64, deck: Option<String>, endless: bool, chaos: bool, save: bool) -> Outcome {
    let label = deck.clone().unwrap_or_else(|| "plain".into());
    let save = save.then(|| {
        std::env::temp_dir()
            .join(format!("omatro-bot-{}", std::process::id()))
            .join(format!("{seed}.json"))
    });
    let result = panic::catch_unwind(AssertUnwindSafe(|| {
        let bot = Bot {
            session: new_session(chaos, &save),
            save: save.clone(),
            rng: Rng::stream(seed, "bot"),
            state: Value::Null,
            endless,
            chaos,
            out: Outcome {
                seed,
                deck: label.clone(),
                ..Outcome::default()
            },
        };
        bot.play(deck.as_deref())
    }));
    if let Some(path) = &save {
        let _ = std::fs::remove_file(path);
    }
    result.unwrap_or_else(|_| Outcome {
        seed,
        deck: label,
        violations: vec![format!(
            "PANIC: {}",
            PANIC_AT.lock().map(|s| s.clone()).unwrap_or_default()
        )],
        ..Outcome::default()
    })
}

fn percent(part: usize, whole: usize) -> String {
    format!("{:.0}%", 100.0 * part as f64 / whole.max(1) as f64)
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let endless = args.iter().any(|a| a == "--endless");
    let chaos = args.iter().any(|a| a == "--chaos");
    let save = args.iter().any(|a| a == "--save");
    if args.iter().any(|a| a == "--trace") {
        TRACE.store(true, std::sync::atomic::Ordering::Relaxed);
    }
    if chaos {
        // SAFETY: set before any thread starts.
        unsafe { std::env::set_var("OMATRO_DEBUG", "1") };
        unsafe { std::env::remove_var("OMATRO_JOKERS") };
    }
    let numbers: Vec<u64> = args.iter().filter_map(|a| a.parse().ok()).collect();
    let games = numbers.first().copied().unwrap_or(200);
    let first = numbers.get(1).copied().unwrap_or(1);

    panic::set_hook(Box::new(|info| {
        if let Ok(mut at) = PANIC_AT.lock() {
            *at = info.to_string();
        }
    }));

    let decks: Vec<Option<String>> = {
        let mut session = Session::default();
        let r = serde_json::to_value(session.handle(r#"{"cmd":"decks"}"#)).unwrap();
        std::iter::once(None)
            .chain(
                r["decks"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|d| d["id"].as_str().map(|s| Some(s.to_string()))),
            )
            .collect()
    };

    let start = Instant::now();
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let seeds: Vec<u64> = (first..first + games).collect();
    let outcomes: Vec<Outcome> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..threads)
            .map(|t| {
                let seeds = &seeds;
                let decks = &decks;
                scope.spawn(move || {
                    seeds
                        .iter()
                        .skip(t)
                        .step_by(threads)
                        .map(|&seed| {
                            let deck = decks[(seed as usize) % decks.len()].clone();
                            run_game(seed, deck, endless, chaos, save)
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles.into_iter().flat_map(|h| h.join().unwrap()).collect()
    });

    let n = outcomes.len();
    let wins = outcomes.iter().filter(|o| o.won).count();
    let steps: usize = outcomes.iter().map(|o| o.steps).sum();
    println!(
        "{n} games in {:.1}s ({steps} commands), won {wins} ({}), average ante {:.2}",
        start.elapsed().as_secs_f64(),
        percent(wins, n),
        outcomes.iter().map(|o| o.ante as f64).sum::<f64>() / n.max(1) as f64
    );

    let mut antes: BTreeMap<u64, usize> = BTreeMap::new();
    for o in &outcomes {
        *antes.entry(o.ante).or_default() += 1;
    }
    println!("\nAnte reached:");
    for (ante, count) in &antes {
        println!("  {ante:>2}  {count:>5}  {}", "#".repeat((60 * count / n.max(1)).max(1)));
    }

    println!("\nDecks (games, wins):");
    let mut per_deck: BTreeMap<&str, (usize, usize, u64)> = BTreeMap::new();
    for o in &outcomes {
        let e = per_deck.entry(&o.deck).or_default();
        e.0 += 1;
        e.1 += usize::from(o.won);
        e.2 += o.ante;
    }
    for (deck, (games, wins, antes)) in &per_deck {
        println!(
            "  {deck:<18} {games:>4} games  {:>4} won  ante {:.1}",
            percent(*wins, *games),
            *antes as f64 / *games as f64
        );
    }

    let mut bosses: BTreeMap<&str, usize> = BTreeMap::new();
    for o in &outcomes {
        if let Some(boss) = &o.lost_to {
            *bosses.entry(boss).or_default() += 1;
        }
    }
    let mut bosses: Vec<_> = bosses.into_iter().collect();
    bosses.sort_by_key(|b| std::cmp::Reverse(b.1));
    println!("\nGames lost at a boss blind:");
    for (boss, count) in bosses.iter().take(10) {
        println!("  {boss:<22} {count:>4}");
    }

    let mut per_joker: BTreeMap<&str, (usize, u64)> = BTreeMap::new();
    for o in &outcomes {
        for joker in o.jokers.iter().collect::<HashSet<_>>() {
            let e = per_joker.entry(joker).or_default();
            e.0 += 1;
            e.1 += o.ante;
        }
    }
    let mut jokers: Vec<_> = per_joker
        .iter()
        .filter(|(_, (games, _))| *games >= 5)
        .map(|(j, (games, antes))| (*j, *games, *antes as f64 / *games as f64))
        .collect();
    jokers.sort_by(|a, b| b.2.total_cmp(&a.2));
    println!("\nJokers held at the end (at least 5 games), by average ante reached:");
    for (joker, games, ante) in jokers.iter().take(10) {
        println!("  top     {joker:<24} {games:>4} games  ante {ante:.1}");
    }
    for (joker, games, ante) in jokers.iter().rev().take(10).rev() {
        println!("  bottom  {joker:<24} {games:>4} games  ante {ante:.1}");
    }
    let never = omatro_engine::joker::Joker::ALL
        .iter()
        .filter_map(|j| serde_json::to_value(j).ok())
        .filter_map(|v| v.as_str().map(String::from))
        .filter(|id| !per_joker.contains_key(id.as_str()))
        .collect::<Vec<_>>();
    println!("  never held at the end: {}", never.join(", "));

    let mut errors: BTreeMap<(String, String), (usize, u64)> = BTreeMap::new();
    for o in &outcomes {
        for e in &o.errors {
            errors.entry(e.clone()).or_insert((0, o.seed)).0 += 1;
        }
    }
    println!("\nRefused commands (the bot thought they were fine):");
    for ((cmd, message), (count, seed)) in &errors {
        println!("  {count:>6}  {cmd:<14} {message}   (first: seed {seed})");
    }

    // Problems grouped by kind (numbers left out), with the games they hit.
    let mut problems: BTreeMap<String, (Vec<u64>, String)> = BTreeMap::new();
    for o in &outcomes {
        let stuck = o
            .stuck
            .then(|| format!("stuck after {MAX_STEPS} commands"));
        for v in o.violations.iter().cloned().chain(stuck) {
            let kind: String = v.chars().filter(|c| !c.is_ascii_digit()).collect();
            let entry = problems.entry(kind).or_insert_with(|| (Vec::new(), v.clone()));
            if !entry.0.contains(&o.seed) {
                entry.0.push(o.seed);
            }
        }
    }
    println!("\nProblems (games hit, first example):");
    for (seeds, example) in problems.values() {
        let shown: Vec<String> = seeds.iter().take(5).map(u64::to_string).collect();
        println!("  {:>5}  {example}   (seeds {})", seeds.len(), shown.join(", "));
    }
    if problems.is_empty() {
        println!("  none");
    }
}
