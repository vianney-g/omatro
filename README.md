# omatro

A Balatro-like card game for Omarchy: play poker hands to beat blinds, while jokers
multiply the score (chips × mult).

A standalone Qt Quick application, like omawrite or omacut, but written in Rust
(Qt binding with [cxx-qt](https://github.com/KDAB/cxx-qt)):

- `engine/`: the engine (rules, scoring, jokers, shop), with no Qt dependency. It also
  provides an `omatro-engine` binary that speaks the JSON protocol on stdin/stdout.
- `app/`: the application. The QML is embedded in the binary, and the engine is called
  directly, with the same JSON protocol. The active Omarchy theme (colors from
  `colors.toml`, font size from `shell.toml`, Hyprland rounding) is reloaded as soon as
  it changes.
- `pkgbuild/`: launcher entry and icon.

A personal project, unaffiliated with Balatro: the mechanics are borrowed, but not the
name, the artwork, the text or the code. The content is themed around Omarchy, Arch and
Hyprland, with affectionate jokes about DHH; it is unaffiliated with them too.

## Installation

Requires Rust and Qt 6 (`qt6-base`, `qt6-declarative`).

```sh
make install   # build and install into ~/.local (binary, launcher entry, icon)
omatro
make run       # build and run without installing
make test
```

Keys: 1 to 8 to choose cards, Enter to play, D to discard, N for a new game,
Ctrl+Q to quit. In the shop: 1 and 2 to buy, R to reroll, Enter for the next blind.

## How a game goes

Each blind requires reaching a score (300, then × 1.5) within 4 hands and 3 discards.
A won blind pays $4, plus $1 per unplayed hand, plus $1 of interest per $5 held
($5 at most). Cash out, then the shop offers 2 jokers you don't own yet (5 slots,
resold at half price; reroll for $5, then $1 more each time). You start with $4 and
no jokers.

## Engine alone

```sh
cargo run -p omatro-engine   # reads commands on stdin
```

## Protocol

One JSON command per line on standard input, one JSON response per line on output.

| Command | Effect |
|---|---|
| `{"cmd":"new_game","seed":42,"jokers":["dotfiles"]}` | New game (`seed` and `jokers` optional) |
| `{"cmd":"state"}` | Returns the state |
| `{"cmd":"play","cards":[0,2,3]}` | Plays 1 to 5 cards (indices into the hand) |
| `{"cmd":"discard","cards":[1]}` | Discards 1 to 5 cards |
| `{"cmd":"preview","cards":[0,2]}` | Hand the selection would make, without playing it |
| `{"cmd":"cash_out"}` | Cashes out a won blind's reward and opens the shop |
| `{"cmd":"buy","offer":0}` | Buys a shop offer |
| `{"cmd":"sell","joker":0}` | Sells a joker (in the shop) |
| `{"cmd":"reroll"}` | Rerolls the shop offers |
| `{"cmd":"next_blind"}` | Leaves the shop for the next blind |

Responses:

- `{"event":"state","state":{…}}`: game state (blind, target, score, hands and discards
  left, `phase` = `playing` / `won` / `shop` / `lost`, hand, jokers, `money`, `reward`
  in the `won` phase, `shop` offers and `reroll_cost` in the shop).
- `{"event":"scored","score":{…},"state":{…}}` after `play`: played cards, recognized
  hand, scoring cards, base chips and mult, then `steps` (each trigger, with chips and
  mult afterwards) to animate the calculation, and the `total`.
- `{"event":"preview","cards":[…],"hand":"pair","scoring":[…],"base_chips":10,"base_mult":2.0}`
  (`hand` is `null` when nothing is chosen, `scoring` holds hand indices).
- `{"event":"error","message":"…"}`.

The hand is always sorted by descending rank.

Cards are written `{"rank":14,"suit":"spades"}` (rank 2 to 14: 11 jack, 12 queen,
13 king, 14 ace).

## Jokers

See [docs/jokers.md](docs/jokers.md), generated from the engine code.
