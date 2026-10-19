# Difficulty levels

<!-- Generated from the engine code by `make docs`: do not edit by hand. -->

Chosen on the new game screen with the deck (stakes, in the original): each
level adds its rule to those of the levels below it. Each deck starts at the
first level only; winning a game with a deck unlocks the next level for
that deck.

| Level | Id | Name | Adds | Quip |
|---|---|---|---|---|
| 1 | `daily_commute` | Daily Commute | The game as it is | *Speed limits respected. Mostly.* |
| 2 | `track_day` | Track Day | The small blind pays no reward | *Fun costs money. Lots of it.* |
| 3 | `club_racing` | Club Racing | Targets grow faster from ante to ante | *The amateurs got faster.* |
| 4 | `quattro` | Quattro | Shop jokers may be Welded | *Permanent all-wheel drive. Permanent jokers.* |
| 5 | `group_b` | Group B | −1 discard each blind | *Too fast, no brakes. Banned in '86.* |
| 6 | `le_mans` | Le Mans | Targets grow faster still | *24 hours. Ask DHH how it went.* |
| 7 | `endurance` | Endurance | Shop jokers may be Worn | *Tyres wear out. So do jokers.* |
| 8 | `hypercar` | Hypercar | Shop jokers may be Leased | *You don't own it. The sponsor does.* |

Targets grow with the ante along three tables: levels 1 and 2 use the
first, from Club Racing the second, from Le Mans the third.

## Stickers

From the levels that allow them, each shop joker has a 30% chance to be
Welded, then (from Endurance) a 30% chance to be Worn instead; from Hypercar, a
30% chance to be Leased as well. Jokers that run out, break or are sold to
work never get one. A Worn joker lasts 5 blinds; a Leased one costs
$1, then $3 at the end of each blind, and sells for $1.

| Id | Sticker | Effect |
|---|---|---|
| `welded` | Welded | Can't be sold or destroyed |
| `worn` | Worn | Stops working after 5 blinds |
| `leased` | Leased | Costs $1 to buy, then $3 at the end of each blind |
