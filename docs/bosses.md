# Boss blinds

<!-- Generated from the engine code by `make docs`: do not edit by hand. -->

Each ante's boss blind has an effect, announced before the blind. Bosses are
drawn among those allowed at the ante, without repeat until all were met; the
final bosses come at ante 8 and its multiples. A boss pays $5 ($8 for a final
one). The kill -9 joker disables every boss; Rollback, sold during the boss
blind, disables it.

| Id | Boss | Effect | From ante | Target | Quip |
|---|---|---|---|---|---|
| `oom_killer` | The OOM Killer | Discards 2 random cards from the hand after each hand played | 1 | ×2 | *Something had to go.* |
| `cloud_bill` | The Cloud Bill | Playing your most played hand type sets money to $0 | 6 | ×2 | *Your usage spiked. So did the invoice.* |
| `black_screen` | The Black Screen | The first hand is dealt face down | 2 | ×2 | *Did it boot? Nobody knows.* |
| `kubernetes_cluster` | The Kubernetes Cluster | A huge blind: ×4 the base score | 2 | ×4 | *Seventeen YAML files for one hello world.* |
| `flaky_wifi` | The Flaky Wi-Fi | 1 in 7 cards is drawn face down | 2 | ×2 | *Connected, no internet.* |
| `regression` | The Regression | Each hand played loses one level | 2 | ×2 | *It worked in the last release.* |
| `electron_app` | The Electron App | Clubs don't score | 1 | ×2 | *500 MB of RAM to show a club.* |
| `screen_lock` | The Screen Lock | Cards drawn after each hand are face down | 2 | ×2 | *Hyprlock kicked in. Type your password.* |
| `mandatory_meeting` | The Mandatory Meeting | Hands must have 5 cards | 1 | ×2 | *Everyone has to attend.* |
| `nvidia_driver` | The Nvidia Driver | Spades don't score | 1 | ×2 | *Spades? Not supported on Wayland.* |
| `read_only_filesystem` | The Read-Only Filesystem | No discards | 2 | ×2 | *mount: / is read-only.* |
| `paywall` | The Paywall | Diamonds don't score | 1 | ×2 | *Diamonds are for premium users.* |
| `monitor43` | The 4:3 Monitor | −1 hand size | 1 | ×2 | *Remember when screens were square?* |
| `code_reviewer` | The Code Reviewer | No hand type can be played twice | 3 | ×2 | *Seen this pattern before. Rejected.* |
| `style_guide` | The Style Guide | Only one hand type can be played | 2 | ×2 | *Pick a convention. Forever.* |
| `anonymous_account` | The Anonymous Account | Face cards don't score | 4 | ×2 | *No faces, only opinions.* |
| `rate_limiter` | The Rate Limiter | After each hand or discard, draw only 3 cards | 5 | ×2 | *429 Too Many Requests.* |
| `stale_cache` | The Stale Cache | Cards already played this ante don't score | 1 | ×2 | *Invalidate and try again.* |
| `deadline` | The Deadline | Only 1 hand, but a smaller blind (×1) | 2 | ×1 | *Ship it today. One shot.* |
| `burnout` | The Burnout | Hearts don't score | 1 | ×2 | *No love left for this codebase.* |
| `per_seat_license` | The Per-Seat License | −$1 per card played | 3 | ×2 | *Every seat is billed. Every one.* |
| `thermal_throttle` | The Thermal Throttle | Base chips and mult are halved | 2 | ×2 | *Fans at 100%. Clocks at 50%.* |
| `dark_pattern` | The Dark Pattern | Face cards are drawn face down | 2 | ×2 | *Hidden on purpose.* |
| `kernel_panic` | The Kernel Panic | Jokers are flipped face down and shuffled | 8, 16… | ×2 | *Attempted to kill init!* |
| `vendor_lock_in` | The Vendor Lock-in | No card scores until a joker is sold | 8, 16… | ×2 | *Pay to leave.* |
| `microservices` | The Microservices | A colossal blind: ×6 the base score | 8, 16… | ×6 | *Two hundred services for a todo app.* |
| `windows_update` | The Windows Update | A random joker is disabled each hand | 8, 16… | ×2 | *Restarting in 10 minutes. Or now.* |
| `clippy` | The Clippy | One card is always forced into the selection | 8, 16… | ×2 | *It looks like you're playing a hand!* |
