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

The whole codebase and the interface are in English: code, comments, error messages,
UI text, README and commit messages. Numbers are displayed in English format
(`12,345`, `2.5`, `$4`). Conversation with the maintainer may be in French.

## Commands

Requires Rust and Qt 6 (`qt6-base`, `qt6-declarative`).

```sh
make test                          # cargo test --workspace
cargo test -p omatro-engine        # engine tests only (no Qt build needed)
cargo test -p omatro-engine hand:: # one module, or a specific test name
make run                           # release build, then runs target/release/omatro
make install                       # installs into ~/.local (binary, .desktop, icon)
cargo run -p omatro-engine         # engine alone: JSON commands on stdin, one per line
```

To poke at the engine by hand:
`printf '%s\n' '{"cmd":"new_game","seed":42}' '{"cmd":"play","cards":[0,1]}' | cargo run -q -p omatro-engine`

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
- Sizing: every size goes through `Style.space()` / `Style.fontPx()`, i.e. theme scale ×
  `Style.fit`. `Main.qml` sets `fit` so the whole UI shrinks together when the window is
  smaller than `designWidth` × `designHeight` (tiled Hyprland windows ignore the minimum
  size). The window's own size must use `Style.themeScale`, never `Style.space()`, or it
  makes a binding loop. If the layout gets wider or taller (the shop sets the height:
  joker row and offers), raise `designWidth` or `designHeight`. Rows whose length
  varies (hand size, joker slots) narrow their items instead (`handRow.cardWidth`,
  `jokerRow.jokerWidth`).

## Debugging and screenshots

`OMATRO_DEBUG=1` makes `Session::from_env()` accept the `debug` command (`Cheat` in
`game.rs`: money, jokers, jump to an ante or blind, win the blind in progress); the app
binds F5 / F6 / F7 / F8 / F9 / F10 to it (F8 raises every known hand one level, F9 adds a
random joker, F10 a random theme). `OMATRO_JOKERS=uptime,fork` (debug only) sets the
jokers new games start with, to screenshot specific jokers. To check the interface, run the app with it, float and size the
window, drive it with `wtype` and capture it with `grim`. Hyprland 0.56 dispatchers use
Lua: `hyprctl dispatch 'hl.dsp.window.float({ action = "toggle" })'`,
`'hl.dsp.window.resize({ x = 1100, y = 800 })'`, `'hl.dsp.window.center()'`; the window
geometry comes from `hyprctl clients -j`; `'hl.dsp.cursor.move({ x = …, y = … })'`
moves the pointer, to check hover tooltips. F11 gives the cards in hand random
modifiers, F12 adds a random command. Check that stderr has no QML warnings.

## Adding a joker

Add a variant to the `Joker` enum and to `Joker::ALL` (fixed-size array), then `name`,
`description`, `quip`, `rarity` and `price` in `engine/src/joker.rs`, and its behaviour:
- scoring triggers on `Joker`, which `score.rs` calls in this order: `retriggers` and
  `on_card_scored` per scoring card, `on_held` per card held in hand, `on_hand` once
  (with the joker's `counter`); and `blind_money` at the end of a won blind. Chances
  take the `Rng` passed in (the game's `score` or `round` stream), never a new one.
- jokers that grow or change: `OwnedJoker` keeps a `counter` (and `sell_bonus`),
  updated by `before_hand`, `after_hand` (false = used up), `on_discard`,
  `on_card_sold`, `end_of_blind` (false = broken); `initial_counter` and `current()`
  (the "Currently +n" line) go with them.
- rule changers (hand size, discards, four-card hands) are read by `Game` with
  `count(Joker::…)`; mark them, and anything else Fork must not copy, in `copyable`.

Every joker that grows, shrinks or depends on the game must show what it gives right
now: add it to `OwnedJoker::bonus()`, which feeds the badge on the joker card (the
state, and the preview for the selected hand). The player must always see the bonus
actually applied.

Fork copies through `score::copy_target`; `Game::abilities(joker)` counts a joker's
ability with the Forks copying it. Then run `make docs`.

## Card modifiers

`modifier.rs` holds the enhancements, seals and editions (the edition also decorates
jokers). A `Card` carries them for the whole game: change a card with
`Game::modify_card(id, …)`, which updates the deck, the draw pile and the hand alike.
Use `Card::is_suit`, `has_rank`, `is_face` and `chips` rather than reading `suit` or
`rank`: a Binary Blob has neither, a Symlink counts as every suit.

## Consumables

`consumable.rs` holds the consumables (`Consumable::Theme` and `Consumable::Command`;
sudo cards will join them), kept in the clipboard (`Game::consumables`, 2 slots) and
used by `Game::use_consumable(index, cards)`: `targets()` says how many hand cards a
command needs selected, `check_usable` refuses one that could do nothing, `run_command`
applies it. Add things to the clipboard with `Game::create`, which respects the slots.
The shop draws a joker (weight 20), a theme (4) or a command (4) for each slot
(`Offer`); the state sends offers as `OfferInfo`, told apart by `kind`. Random
creations draw from the `commands` stream; `roll_joker` and `roll_theme` take the
stream to draw from.

## Generated references

`docs/*.md` (jokers, poker hands, and each new content family) are generated by
`engine/src/docs.rs`, and the `docs_are_up_to_date` test fails whenever one no longer
matches the code: run `make docs` after changing content. Never edit them by hand; to
change a layout or intro text, or to add a family, edit `docs.rs`.

## Tests

Tests are `#[cfg(test)]` modules in each engine file. The `card::cards("AS KH 10D 2C")`
helper builds test cards (rank, then H/D/C/S).
