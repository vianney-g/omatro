# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

omatro is a Balatro-like card game for Omarchy: a standalone Qt Quick application written
in Rust with cxx-qt. The README covers the rules and the JSON protocol; the jokers are listed in
`docs/jokers.md`.

Game content (jokers and, later, consumables, bosses, decks…) is themed around Omarchy,
Arch, Hyprland and DHH, with humor: affectionate jabs, never mean. Names and texts are
original, never Balatro's (only the mechanics are borrowed). Each item has a `quip()`,
a one-line joke shown under its effect (50 characters at most, or it overflows the card).
New content names and quips are proposed to the maintainer for approval before coding.

The whole codebase is in English: code, comments, error messages, UI text as written in
the code, README and commit messages. The game is translated (English and French, see
Translations); English numbers read `12,345`, `2.5`, `$4`, French ones `12 345`,
`2,5`, `4 $`. Conversation with the maintainer may be in French.

## Commands

Requires Rust and Qt 6 (`qt6-base`, `qt6-declarative`; `qt6-multimedia` and
`qt6-multimedia-ffmpeg` for the music).

```sh
make test                          # cargo test --workspace
cargo test -p omatro-engine        # engine tests only (no Qt build needed)
cargo test -p omatro-engine hand:: # one module, or a specific test name
make run                           # release build, then runs target/release/omatro
make install                       # installs into ~/.local (binary, .desktop, icon)
make bot ARGS="2000 1 --chaos"     # bot games: crashes, stuck games, invariants, stats
cargo run -p omatro-engine         # engine alone: JSON commands on stdin, one per line
```

To poke at the engine by hand:
`printf '%s\n' '{"cmd":"new_game","seed":42}' '{"cmd":"play","cards":[0,1]}' | cargo run -q -p omatro-engine`

Releases: bump `version` in `app/Cargo.toml` and `engine/Cargo.toml`, then push a
`v<version>` tag; `.github/workflows/release.yml` builds on Arch Linux, runs the tests
and publishes the archive (binary, launcher entry, icon) with its checksum.

## Architecture

A Cargo workspace with two crates:

- `engine/` (`omatro-engine`): all the rules, with no Qt dependency. `Game` (`game.rs`)
  holds the state and the phases `blind_select` → `playing` → `won` → `shop` →
  `blind_select` (or `lost`); `blind.rs` has the antes (small, big, boss blinds), their
  targets and the endless-mode formula. Scores are `f64`, as endless targets outgrow
  integers.
  `hand.rs` recognizes the poker hand (including three secret hands that need
  duplicate cards) and holds the hand levels (`HandLevels`); `Game` keeps the whole
  `deck` for the game and a `draw_pile` per blind, `score.rs` scores it as a list of `steps` that the
  UI replays to animate chips × mult, `joker.rs` defines the jokers and their triggers.
  `rng.rs` is a hand-written generator, deterministic from the seed (no `rand` crate):
  the same `seed` gives the same game. Each use draws from its own stream
  (`Rng::stream(seed, "deck")`, `"shop"`…) so that, say, a shop reroll never changes
  the next deal; give new random features their own stream.
- `protocol.rs`: `Session::handle(&str) -> Response` is the only entry point into the
  engine. It is used both by the `omatro-engine` binary (stdin/stdout) and by the app.
  Commands and responses are serde enums (`tag = "cmd"` / `tag = "event"`,
  `snake_case`): a protocol change touches these enums, the QML that reads the
  responses, and the README table.
- `app/` (`omatro`): `bridge.rs` (cxx-qt bridge) exposes two QObjects to QML:
  `Engine.handle(json) -> json`, which wraps a `Session` (so the QML also goes through
  the JSON protocol, never through Rust types), and the `OmarchyTheme` singleton
  (colors, font size, corner radius). `theme.rs` reads the active Omarchy theme
  (`~/.local/state/omarchy/current/theme/colors.toml`, `~/.config/omarchy/shell.toml`,
  rounding via `hyprctl`); `Main.qml` calls `refresh()` on a timer, which only reloads
  when the fingerprint (modification times) has changed.
- QML (`app/qml/`): embedded in the binary as the `omatro` module. Every new QML file
  must be declared in `app/build.rs` (singletons with `.singleton(true)`, `.js` files
  via `qrc_resources`). Displayed names come from the engine (`name()` of jokers,
  blinds, hands, sent in the state); `Labels.js` only formats ranks, suits, effects
  and numbers. `FastFetch.qml` is the run info panel (I key).
- Music: `Music.qml` plays the Opus tracks of `app/music/` (embedded, listed in
  `build.rs` and in its `tracks`, `song` for those with lyrics, kept out of blinds).
  `Main.qml` loads it through a `Loader`, so a missing Qt Multimedia leaves the
  game silent rather than broken; scripted runs (`OMATRO_SCRIPT`) play none until
  an `audio` step. New tracks: encode them to Opus at 96 kb/s, at −18 LUFS,
  trailing silence trimmed. `build.rs` embeds every file of `music/` and `sounds/`.
- Sound effects: `Sounds.qml` (loaded the same way) holds a `SoundEffect` per WAV
  of `app/sounds/` (mono, 44.1 kHz), with its own level in `levels`; Main plays
  them with `sound(name)`, `act(command, name)` (the sound if the state changed,
  `error` if refused) and `soundTransition` (phase changes). Kenney's CC0 packs
  give most of them; `app/sounds/synth.py` writes the others (chips steps, blind
  won, victory, game over).
- Glossary: the engine's `glossary` command lists every named term (`glossary.rs`:
  modifiers, badges, pull requests, consumables, jokers, packages; hand types only
  block matches) and its test forbids two things sharing a name. Card views
  underline the terms of their text (`Labels.highlight`) and, hovered, call
  `Hints.show`; `Glossary.qml` explains them beside the card. A new view with a
  description does both.
- Sizing: every size goes through `Style.space()` / `Style.fontPx()`, i.e. theme scale ×
  `Style.fit`. `Main.qml` sets `fit` so the whole UI shrinks together when the window is
  smaller than `designWidth` × `designHeight` (tiled Hyprland windows ignore the minimum
  size). The window's own size must use `Style.themeScale`, never `Style.space()`, or it
  makes a binding loop. If the layout gets wider or taller (the shop sets the height:
  joker row and offers), raise `designWidth` or `designHeight`. Rows whose length
  varies narrow their items (the hand, `handRow.cardWidth`) or overlap them
  (`Fan.qml`: the jokers and the clipboard, where the hovered, flashing or pinned
  card comes to the front with its buttons).

## Debugging and screenshots

`engine/examples/bot.rs` plays whole games through `Session::handle` on many seeds
(`[games] [first seed]`, `--endless`, `--chaos` for debug money, jokers and
consumables, `--trace` to follow one seed on stderr). It reports panics, stuck games,
broken invariants (`Bot::check`: add new ones there) and refused commands, then wins
per deck, joker and boss. Run it in both modes after changing rules: every problem
it lists is a bug, in the engine or in the bot.

`OMATRO_DEBUG=1` makes `Session::from_env()` accept the `debug` command (`Cheat` in
`game.rs`: money, jokers, jump to an ante or blind, win the blind in progress); the app
binds F5 / F6 / F7 / F8 / F9 / F10 to it (F8 raises every known hand one level, F9 adds a
random joker, F10 a random theme). `OMATRO_JOKERS=uptime,fork` (debug only) sets the
jokers new games start with, to screenshot specific jokers.
`OMATRO_SCRIPT` (debug only) plays steps through the app's own key handling and saves
screenshots, with no keyboard, pointer or focus, even with `QT_QPA_PLATFORM=offscreen`:
`OMATRO_SCRIPT="size:1100,700;sleep:1500;key:Return;key:Shift+Left;key:F5;shot:/tmp/a.png;quit"`
(see `runScript` in `Main.qml`); since Qt 6.12 the offscreen window's own background
comes out white or black, so check colours in a real window. Give the run its own `XDG_STATE_HOME` and
`XDG_CONFIG_HOME` so that the save and settings stay untouched. Hovers need a real window: float and size the window, drive it with `wtype` and capture it with `grim`. Hyprland 0.56 dispatchers use
Lua: `hyprctl dispatch 'hl.dsp.window.float({ action = "toggle" })'`,
`'hl.dsp.window.resize({ x = 1100, y = 800 })'`, `'hl.dsp.window.center()'`; the window
geometry comes from `hyprctl clients -j`; `'hl.dsp.cursor.move({ x = …, y = … })'`
moves the pointer, to check hover tooltips. F11 gives the cards in hand random
modifiers, F12 adds a random command, F2 a random oligarchy card, F3 opens a random pack (in
the shop), F4 switches this ante's boss to the next one, F1 makes every joker Welded, Worn
and Leased. Check that the app logs no QML
warnings: when stderr is not a terminal, Qt writes them to the journal, so read
`journalctl --user _PID=<pid> -o cat` (a QML error can keep the window from showing
at all). A simulated pointer only reaches the window when it enters it: move it out
before each hover.

## Adding a joker

Add a variant to the `Joker` enum and to `Joker::ALL` (fixed-size array), then `name`,
`description`, `quip`, `rarity` and `price` in `engine/src/joker.rs`, and its behaviour:
- scoring triggers on `Joker`, which `score.rs` calls in this order: `retriggers` and
  `on_card_scored` per scoring card, `on_held` per card held in hand, `on_hand` once
  (with the joker's `counter`); and `blind_money` at the end of a won blind. Chances
  take the `Rng` passed in (the game's `score` or `round` stream), never a new one:
  Chaos Monkey doubles them through `Rng::set_luck` (`Game::update_luck`).
- what jokers see of the game (deck, money, slots, hands and discards left, the hand
  types played this blind) comes in a `Board`, built by `Game::board()`; `Targets`
  holds what Rolling Release, Daily Driver, Log Rotation, Bug Bounty and TODO.md aim
  at, drawn at each blind.
- rules on cards: `Game::in_play` flags the cards played, discarded or held with
  Fuzzy Finder (`smeared`) and Identicon (`any_face`), which `is_suit` and `is_face`
  read; MVP and Fast-Forward go through `hand::Rules`.
- jokers that grow or change: `OwnedJoker` keeps a `counter` (and `sell_bonus`),
  updated by `before_hand`, `after_hand` (false = used up), `on_discard`,
  `on_cards_discarded` (false = used up), `on_card_sold`, `on_reroll`,
  `on_cards_added` (call `Game::cards_added` wherever cards join the deck),
  `end_of_blind` (false = broken); `initial_counter` goes with them. Effects when a
  blind starts live in `Game::jokers_at_blind_start`.
- rule changers (hand size, discards, four-card hands) are read by `Game` with
  `count(Joker::…)`; mark them, and anything else Fork must not copy, in `copyable`.

Every joker that grows, shrinks or depends on the game must show what it gives right
now: add it to `OwnedJoker::bonus()`, which feeds the badge on the joker card (the
state, and the preview for the selected hand). The player must always see the bonus
actually applied.

Fork (right neighbour) and Upstream (leftmost joker) copy through `score::copy_target`,
which follows copies of copies; `Game::abilities(joker)` counts a joker's ability with
the jokers copying it. Then run `make docs`.

Money is an `i64`: Cloud Credits lets it go down to −$20 (`Game::spend`, `credit`).

## Card modifiers

`modifier.rs` holds the enhancements, seals and editions (the edition also decorates
jokers). A `Card` carries them for the whole game: change a card with
`Game::modify_card(id, …)`, which updates the deck, the draw pile and the hand alike.
Use `Card::is_suit`, `has_rank`, `is_face` and `chips` rather than reading `suit` or
`rank`: a Binary Blob has neither, a Symlink counts as every suit.

## Bosses

`boss.rs` lists the bosses (min ante, target multiplier, reward). `Game::roll_boss`
draws one per ante (final ones at the winning ante's multiples, no repeat until all
were met). `Game::boss_effect()` is the boss whose effect applies now: none outside the
boss blind, with kill -9, or once Rollback was sold. Effects live in `Game`: `debuffed`
(cards that don't score, sent to `score` as `Table::debuffed`), `draw(Draw::…)` (face-down
cards, The Rate Limiter), `play` (refused hands score nothing, money effects, The OOM
Killer), `select_blind` (discards, hands, The Kernel Panic), `with_forced` (The Clippy).
Report a boss that struck through `triggered`, for Incident Commander.

## Pull requests

`voucher.rs` lists the pull requests (vouchers), in pairs (`follows()`). `Game::vouchers`
holds the merged ones, `voucher_offers` those for sale this ante (`roll_voucher`, at
each new ante; PRs: welcome adds one), `merge` applies the immediate effects (ante, reroll cost). Rules read
them through `has` / `merged`: `hands_per_blind`, `discards_per_blind`, `hand_size`,
`joker_slots`, `consumable_slots`, `max_interest`, `discounted` (shop prices),
`edition_factor`, the shop weights in `roll_offers`, `boss_reroll_cost`.

## Badges

`tag.rs` lists the badges (tags). `Game::blind_tags` holds what the small and big blinds
give (`roll_blind_tags`, at each new ante); `skip_blind` calls `gain_tag`, where
`badges: 2x` copies the next one. `apply_tag` acts right away (money, jokers, levels,
boss reroll, free packs through `open_free_pack`, which come back to the phase they
were opened from) or keeps the badge in `Game::tags` until its moment: `shop_tags` at
cash-out (free offers in `free_offers`, `free_packs`), `select_blind` (`juggle`),
`win_blind` (funding). Take waiting badges with `take_tags`.

## Translations

`locales/fr.json` is the French catalog: English text → translation; keys with
`{name}` slots are templates (`"+{n} mult"`), whose slots are translated in turn (hand
types, nested messages) or written as French numbers. The engine stays in English:
`Session::handle_json` translates responses on the way out (`i18n.rs`: descriptions,
notes, messages, badges' `text`, `hand_name`, and `name` only word for word, so card
names stay), and the `language` command sends the catalog to the interface, whose own
text goes through `I18n.tr("English text", { slot: value })` (and `Labels.tr` in
`Labels.js`). Card names and quips are never translated. New content needs its
French text in the catalog (`the_catalog_covers_the_content` fails otherwise); new
notes or UI text too: `make bot ARGS="2000 1 --chaos --lang fr"` lists what remains
in English.

## Saves

The app's `Session` (`with_save(default_save_path())`, in `bridge.rs`) writes the whole
`Game` to `~/.local/state/omatro/save.json` after each command that changes it, and
removes it once the game is lost; `saved_game` and `resume` read it back. `Game` and
everything in it derive `Serialize`/`Deserialize`: a field added later needs
`#[serde(default)]` so that older saves still load, and a change older saves can't
follow bumps `SAVE_VERSION` (they are then reported, not resumed). `make bot ARGS="300
1 --chaos --save"` resumes every game in a new session every 25 commands and checks
the state is the same. Tests and the `omatro-engine` binary never save. When testing
the app, point `XDG_STATE_HOME` elsewhere for omatro only, or it replaces the
maintainer's game.

## Decks

`run_deck.rs` lists the decks (`Deck`), picked on the new game screen (`NewGame.qml`,
the last one remembered with `Settings`) and passed to `Game::with_deck`. `set_up_deck`
builds the cards and the starting items; the rules ask `deck_is(Deck::…)` where they
apply (`hands_per_blind`, `joker_slots`, `target_of`, `reward`, `roll_offers`…).
Hyprland Dwindle scores through a `Source::Deck` step (`Effect::Balance`).

## Packs

`pack.rs` defines the packs (kind × size, their names, prices and shop odds). The shop
rolls 2 of them at cash-out (`Game::packs`, kept by rerolls). `buy_pack` opens one
(`open_pack`): the `pack` phase, with `Game::opened` holding its items; a coreutils pack
also puts a hand drawn from the deck in `Game::hand`, so commands target it through
the usual `validate`. `pick` applies an item (`apply_consumable` for themes and
commands, shared with the clipboard), `skip_pack` and `close_pack` go back to the shop.
Play and discard check for the `playing` phase themselves.

## Consumables

`consumable.rs` holds the consumables (`Consumable::Theme`, `Command` and `Oligarchy`), kept
in the clipboard (`Game::consumables`, 2 slots; a `Held` entry may be Transparent and
take no slot) and used by `Game::use_consumable(index, cards)`: `targets()` says how
many hand cards it needs selected, `needs_hand()` whether it works on the hand at all,
`check_usable` refuses one that could do nothing, `run_command` / `run_oligarchy` apply it.
`buy_and_use` applies a shop theme or command at once, without the clipboard (not
those that need the hand). A consumable whose effect depends on the game says what
it would do now in `current` (`Game::consumable_current`: a theme's next level, money,
what Up Arrow recalls), as badges do (`Game::tag_info`); give new ones theirs there.
Add things to the clipboard with `Game::create`, which respects the slots. Destroy cards
with `Game::destroy_cards`, which keeps ThePrimeagen counting. Oligarchy cards (a running
joke of DHH's, see oligarchy.fyi) never show up in the shop (`roll_oligarchy` draws
them for packs and jokers).
The shop draws a joker (weight 20), a theme (4) or a command (4) for each slot
(`Offer`); the state sends offers as `OfferInfo`, told apart by `kind`. Random
creations draw from the `commands` stream; `roll_joker` and `roll_theme` take the
stream to draw from.

## Collection and achievements

`profile.rs` keeps what lasts from one game to the next (`Profile`, written to
`~/.local/state/omatro/profile.json` by the app's `Session::with_profile`; tests and
the bot use `with_new_profile`, in memory). `collection.rs` lists every `Item`, with
an id `family:id` that the profile stores as a string. After each command,
`Session::observe` feeds `Profile::observe` with `Game::take_events` (`Event`:
items obtained without staying in the game, hands, blinds won, sales, rerolls, the
end of the game) and `Game::owned_items` (what the game holds now); what it gains
comes back as `news` in the JSON response. A new content family goes in
`Item::all`, and gets an `Event::Obtained` wherever it is obtained without staying
in the game. The profile also keeps the stats and the history of the last games
(`Profile::record`, at a win, a loss, or `abandon` when `new_game` replaces a game
in progress); the `collection` command sends them for the collection's last two tabs.
A game whose seed the player chose (`seed_code` in `new_game`, `Game::seeded`) fills
the collection too, but counts for no achievement, stat nor history (`Game::counts`).
`achievement.rs` lists the achievements; `unlocks` names a hidden item,
which `Game::lock` keeps out of `on_sale` (jokers) and `Session` out of the decks
until it is done. A game with `cheated` set (cheats, `OMATRO_JOKERS`) fills the
collection but counts for no achievement.

## Difficulty levels

`stake.rs` lists the levels (`Stake`, each adding its rule to those below) and the
joker stickers (`Sticker`, kept in `OwnedJoker::stickers`). `Game::with_stake` starts
a game at a level; rules ask `self.stake >= Stake::…` where they apply (`reward`,
`discards_per_blind`, `target_of` through `Stake::scaling` and
`blind::ante_base_scaled`, `roll_stickers` for shop jokers, which draws nothing below
Quattro so that lower levels keep their shops). Welded jokers are left alone by
everything that destroys jokers (`Game::destructible`); a Worn joker past its blinds
is skipped by `copy_target`, `count` and the scoring loop. `Profile` keeps the
highest level won per deck (`best_stake`, `top_stake`); `Session` refuses a level
the deck hasn't unlocked. `Joker::sticker_compat` keeps stickers off jokers that run
out, break or are sold to work.

## Generated references

`docs/*.md` (jokers, poker hands, and each new content family) are generated by
`engine/src/docs.rs`, and the `docs_are_up_to_date` test fails whenever one no longer
matches the code: run `make docs` after changing content. Never edit them by hand; to
change a layout or intro text, or to add a family, edit `docs.rs`.

## Tests

Tests are `#[cfg(test)]` modules in each engine file. The `card::cards("AS KH 10D 2C")`
helper builds test cards (rank, then H/D/C/S).
