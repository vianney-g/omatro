# Poker hands

<!-- Generated from the engine code by `make docs`: do not edit by hand. -->

A hand scores its chips and mult at its current level, plus the chips of its
scoring cards, before the jokers. Each level past the first adds the per-level
chips and mult. The secret hands need duplicate cards; they stay hidden in the
game until played.

| Id | Hand | Level 1 | Per level | Secret |
|---|---|---|---|---|
| `fork_bomb` | Fork Bomb | 160 × 16 | +50 × +3 | yes: *:(){ :\|:& };:* |
| `majestic_monolith` | Majestic Monolith | 140 × 14 | +40 × +4 | yes: *One big beautiful app. No microservices.* |
| `five_of_a_kind` | Five of a Kind | 120 × 12 | +35 × +3 | yes: *Statistically suspicious.* |
| `straight_flush` | Straight Flush | 100 × 8 | +40 × +4 |  |
| `four_of_a_kind` | Four of a Kind | 60 × 7 | +30 × +3 |  |
| `full_house` | Full House | 40 × 4 | +25 × +2 |  |
| `flush` | Flush | 35 × 4 | +15 × +2 |  |
| `straight` | Straight | 30 × 4 | +30 × +3 |  |
| `three_of_a_kind` | Three of a Kind | 30 × 3 | +20 × +2 |  |
| `two_pair` | Two Pair | 20 × 2 | +20 × +1 |  |
| `pair` | Pair | 10 × 2 | +15 × +1 |  |
| `high_card` | High Card | 5 × 1 | +10 × +1 |  |
