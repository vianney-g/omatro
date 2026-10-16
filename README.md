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
Names of real people, products and companies belong to their owners; they appear
as affectionate parody.

The code is under the MIT license (see [LICENSE](LICENSE)); the license covers the
code only, not the names and trademarks the game mentions.

## Installation

Requires Rust and Qt 6 (`qt6-base`, `qt6-declarative`).

```sh
make install   # build and install into ~/.local (binary, launcher entry, icon)
omatro
make run       # build and run without installing
make test
```

A new game starts by picking a deck, the machine you play on
([docs/decks.md](docs/decks.md)): more hands or discards, money, slots, a deck without
face cards, starting pull requests… The last one picked is remembered.

Keys: 1 to 9 and 0 to choose cards, Enter to play, D to discard, Tab to sort the hand
by rank or suit, Shift+←/→ to move the selected card (or drag it), J to bring the next
joker to the front when they overlap (or hover or click one), N for a new game, I for
fastfetch (run info: hand levels, cards left), Ctrl+Q to quit. Before a blind: Enter to
play it, S to skip it. Ctrl+1 and Ctrl+2 apply the consumables of the clipboard. In the
shop: 1 to 6 to buy the cards then the packs, in order, M to merge the pull request, R
to reroll, Enter for the next blind; the buttons under each joker sell it or move it
left and right. Before a boss, B rerolls it (with Change Request). In an open pack: Install
takes a card, Ctrl+C (or Escape) skips.

## How a game goes

The game is played in antes of three blinds: the small blind, the big blind and the
boss blind. Each blind requires reaching a score within 4 hands and 3 discards: the
ante's base score (300, 800, 2,000, 5,000, 11,000, 20,000, 35,000, 50,000) × 1, 1.5 or
2. Before the small and big blinds, you may skip them, for the badge each one shows
([docs/tags.md](docs/tags.md)): money, free jokers or packs, editions, levels… Some act
right away, others wait for the next shop, blind or boss. Each ante's boss
([docs/bosses.md](docs/bosses.md)) has an effect, announced before the blind: cards that
don't score, cards dealt face down, hands it refuses, a bigger target… Beating the
boss blind of ante 8 wins the game; you may then go on in endless mode, where targets
explode.

Each poker hand type has a level, which raises its starting chips and mult (see
[docs/hands.md](docs/hands.md)); three secret hands need duplicate cards. Themes
([docs/themes.md](docs/themes.md)) level up a hand type: bought in the shop, they wait
in the clipboard (2 slots) until applied, at any time. Commands
([docs/commands.md](docs/commands.md)) change the selected cards, make money or create
things; those that change cards are applied during a blind, with the cards selected.
Oligarchy cards ([docs/oligarchy.md](docs/oligarchy.md)): elite capital, public code.
Powerful, at a price, they only come from oligarchy packs and a few jokers, and
Aspiring Billionaire is the only way to a legendary joker. Your deck lasts
the whole game: every played or discarded card comes back to it after each blind. Cards
may carry an enhancement, a seal and an edition ([docs/modifiers.md](docs/modifiers.md));
hover a card to read them.

A won blind pays $3, $4 or $5, plus $1 per unplayed hand, plus $1 of interest per $5
held ($5 at most). Cash out, then the shop offers 2 cards: each slot holds a theme or
a command ($3, 1 chance in 7 each) or a joker you don't own yet, of a rarity drawn for each slot (core
70%, extra 25%, AUR 5%), sometimes with an edition (5 joker slots,
resold at half price; reroll for $5, then $1 more each time). Jokers trigger left to
right, so their order matters. The shop also sells 2 packs ([docs/packs.md](docs/packs.md)),
opened on the spot: coreutils (commands), base16 (themes), base-devel (cards) and
jokers-git (jokers) and oligarchy (oligarchy cards), in normal, `-full` and `-meta` sizes.
Each ante, the shop also offers a pull request ([docs/vouchers.md](docs/vouchers.md)),
a permanent upgrade for $10: more slots, discounts, hands, discards, interest… You start with $4 and no
jokers.

## Engine alone

```sh
cargo run -p omatro-engine   # reads commands on stdin
```

With `OMATRO_DEBUG=1`, the engine accepts `debug` commands, and the app maps them to
F5 (win the blind in progress), F6 (+$100), F7 (jump to the boss of ante 8), F8 (raise
every known hand type one level), F9 (add a random joker with a random edition), F10
(add a random theme to the clipboard) and F11 (give the cards in hand random
enhancements, seals and editions), F12 (add a random command to the clipboard) and F3 (open a random pack, in the
shop), F2 (add a random oligarchy card), F4 (switch this ante's boss to the next one). `OMATRO_JOKERS=uptime,fork` sets the jokers
every new game starts with.

## Protocol

One JSON command per line on standard input, one JSON response per line on output.

| Command | Effect |
|---|---|
| `{"cmd":"new_game","seed":42,"jokers":["dotfiles"],"deck":"thinkpad"}` | New game (`seed`, `jokers` and `deck` optional) |
| `{"cmd":"decks"}` | Lists the decks a new game may start with |
| `{"cmd":"state"}` | Returns the state |
| `{"cmd":"select_blind"}` | Plays the blind on offer: deals a hand |
| `{"cmd":"skip_blind"}` | Skips the blind on offer (not the boss) |
| `{"cmd":"play","cards":[0,2,3]}` | Plays 1 to 5 cards (indices into the hand) |
| `{"cmd":"discard","cards":[1]}` | Discards 1 to 5 cards |
| `{"cmd":"preview","cards":[0,2]}` | Hand the selection would make, without playing it |
| `{"cmd":"cash_out"}` | Cashes out a won blind's reward and opens the shop |
| `{"cmd":"buy","offer":0}` | Buys a shop offer |
| `{"cmd":"sell","joker":0}` | Sells a joker (in the shop, before or during a blind) |
| `{"cmd":"move_joker","from":0,"to":2}` | Moves a joker within the row |
| `{"cmd":"move_card","from":7,"to":0}` | Moves a card of the hand (the hand then keeps this order) |
| `{"cmd":"sort_hand","by":"suit"}` | Sorts the hand by `rank` or `suit`, and new hands from now on |
| `{"cmd":"buy_pack","pack":0}` | Buys a pack of the shop and opens it |
| `{"cmd":"pick","choice":0,"cards":[1]}` | Takes a card of the open pack (`cards`: the drawn hand's cards a command changes) |
| `{"cmd":"skip_pack"}` | Leaves the open pack |
| `{"cmd":"buy_voucher","index":0}` | Merges a pull request for sale (in the shop) |
| `{"cmd":"reroll_boss"}` | Draws another boss for the ante (Change Request, before the blind) |
| `{"cmd":"use_consumable","index":0,"cards":[1,3]}` | Uses a consumable of the clipboard (any time but after a loss); `cards`: the hand cards a command changes |
| `{"cmd":"sell_consumable","index":0}` | Sells a consumable of the clipboard |
| `{"cmd":"reroll"}` | Rerolls the shop offers |
| `{"cmd":"next_blind"}` | Leaves the shop for the choice of the next blind |
| `{"cmd":"debug","money":50,"jokers":["hotwire"],"edition":"blur","ante":8,"blind":"boss","win":true,"level_up":["pair"],"random_joker":true,"random_theme":true,"enhance_hand":true,"random_command":true,"random_pack":true,"random_oligarchy":true,"boss":"burnout","next_boss":true,"vouchers":["overtime"],"tags":["build_passing"]}` | Testing shortcuts, all fields optional; only with `OMATRO_DEBUG=1` |

Responses:

- `{"event":"state","state":{…}}`: game state: `run_deck` (the deck picked, or `null`), `ante`, `blind` (`small` / `big` /
  `boss`), the ante's three `blinds` (name, the boss's `description`, target, reward,
  `status`, and the `tag` skipping it gives), the ante's `boss` and `boss_active`, `target`, `score`,
  hands and discards left (in the `pack` phase, the hand is the one a coreutils pack
  drew; `face_down`, `debuffed` and `forced` follow the hand's cards), `remaining` (cards left to draw, or the whole deck between
  blinds), `deck_size`, `sort_by` (`rank` or `suit`), `hands` (each hand type's `level`, `chips`, `mult` and `played`
  count; secret hands only once played), `phase` = `blind_select` / `playing` / `won` / `shop` /
  `pack` / `lost`, `victory` (the winning ante's boss was just beaten), hand, `hand_size`,
  jokers (`id`, `name`, `description`, `quip`, `rarity`, `edition`, `price`,
  `sell_value`, `bonus` for what a growing joker gives now, `current` for what a Fork
  copies), `max_jokers`, `jokers_hidden`, `disabled_joker`, `vouchers` merged, `vouchers_for_sale` (in
  the shop), `tags` (badges won, waiting), `boss_reroll_cost` (when Change Request allows it now), `modifiers` (names and effects of every enhancement, seal and
  edition), `consumables` (`kind` = `theme` / `command` / `oligarchy`, `id`, `name`,
  `description`, `quip`, `price`, `sell_value`, `min_cards` / `max_cards` to select,
  `needs_hand` when it works on the cards in hand, `transparent` when it takes no slot), `consumable_slots`, `unique_themes`, `money` (below $0 with Cloud Credits, down to `-credit`),
  `reward` in the `won` phase (with `cards` for Enterprise cards held, `discards` for
  the Raspberry Pi's unused discards, and `notes` such
  as "Beta Release broke."), `shop` offers
  (jokers, consumables or `{"kind":"card","card":…,"price":n}`, told apart by `kind`), `packs` for sale, and `opened`: the
  pack being opened (`name`, `description`, `picks_left`, `with_hand`, and its
  `choices`: consumables, jokers or `{"kind":"card","card":…}`) and `reroll_cost` in the shop. Scores and
  targets are floats; an infinite target (from ante 39) is `null`.
- `{"event":"scored","score":{…},"state":{…}}` after `play`: played cards, recognized
  hand and its `hand_name`, scoring cards, base chips and mult, then `steps` (each
  trigger, with chips and mult afterwards) to animate the calculation, and the `total`.
  A step's `source` is a played `card`, a `held` card or a `joker`; its `effect` is
  `{"chips":n}`, `{"mult":n}`, `{"x_mult":n}`, `{"money":n}`, `"again"` (a retrigger) or
  `"debuffed"` (the boss keeps the card from scoring) or `"balance"` (a `deck` source:
  Hyprland Dwindle evens out chips and mult).
  `money` is what the hand earned, `notes` what happened to the jokers.
- `{"event":"preview","cards":[…],"hand":"pair","hand_name":"Pair","scoring":[…],"base_chips":10,"base_mult":2.0}`
  (`hand` is `null` when nothing is chosen, `scoring` holds hand indices), with
  `bonuses`: each joker's badge if this hand were played.
- `{"event":"decks","decks":[…]}` after `decks`: each deck's `id`, `name`,
  `description` and `quip`.
- `{"event":"error","message":"…"}`.

The hand is always sorted by descending rank.

Cards are written `{"rank":14,"suit":"spades","id":51,"enhancement":"daemon","seal":"retry","edition":null,"bonus_chips":5}`
(rank 2 to 14: 11 jack, 12 queen, 13 king, 14 ace; `id` tells apart the cards of the
deck; `enhancement`, `seal` and `edition` are ids or `null`, `bonus_chips` what the card
gained).

## Jokers

See [docs/jokers.md](docs/jokers.md) and [docs/editions.md](docs/editions.md),
generated from the engine code.
