# Card modifiers

<!-- Generated from the engine code by `make docs`: do not edit by hand. -->

Each card of the deck may carry an enhancement, a seal and an edition, for the
whole game. When a card scores: its chips (with Optimized, and what it gained),
then its enhancement, its edition, its seal, and the jokers; Retry and jokers
may score it again. Cards held in hand then trigger (Daemon), once more per
Cron Job.

## Enhancements

| Id | Name | Effect | Quip |
|---|---|---|---|
| `optimized` | Optimized | +30 chips when scored | *Compiled with -O3.* |
| `multithreaded` | Multithreaded | +4 mult when scored | *Uses all the cores. Finally.* |
| `symlink` | Symlink | Counts as every suit | *Points wherever you need it.* |
| `overclocked` | Overclocked | ×2 mult when scored; 1 in 4 chance to break | *Fast. Hot. Occasionally on fire.* |
| `daemon` | Daemon | ×1.5 mult while held in hand | *Runs in the background. Never sleeps.* |
| `binary_blob` | Binary Blob | +50 chips; no rank, no suit, always scores | *Nobody knows what's inside. It works.* |
| `enterprise` | Enterprise | +$3 if held in hand at the end of the blind | *Comes with a support contract.* |
| `flaky_test` | Flaky Test | 1 in 5 chance of +20 mult, 1 in 15 chance of +$20 | *Passes sometimes. Spectacularly.* |

## Seals

| Id | Name | Effect | Quip |
|---|---|---|---|
| `gold_star` | Gold Star | +$3 when scored | *A GitHub star, but it pays.* |
| `retry` | Retry | Scores one more time | *Exit code 1. Trying again.* |
| `theme_sync` | Theme Sync | If held in hand at the end of the blind, creates the theme of the last hand played | *Syncs your theme. Eventually.* |

Editions are the same as on jokers ([editions.md](editions.md)); on a card,
Transparent adds one card to the hand while the card is in it.
