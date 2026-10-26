# Jokers

<!-- Generated from the engine code by `make docs`: do not edit by hand. -->

Jokers are bought in the shop between blinds (5 slots at most) and sell for
half their price. Each shop slot draws a rarity, named after Arch's
repositories: core (70%), extra (25%) or AUR (5%); legendary jokers never
show up in the shop. A joker may come with an edition (see
[editions.md](editions.md)).

While a hand is scored, each scoring card scores its chips, then card jokers
trigger, and retriggers score it again; then jokers trigger for the cards
held in hand; then each joker, left to right, adds its effect. Order matters:
Fork copies the joker on its right.

Hidden jokers only show up once their achievement is done (see
[achievements.md](achievements.md)).

| Id | Name | Rarity | Price | Effect | Quip |
|---|---|---|---|---|---|
| `cloud_credits` | Cloud Credits | core | $2 | You can spend down to −$20 | *Free until the invoice arrives.* |
| `pair_programming` | Pair Programming | core | $3 | +8 mult if the hand contains a Pair | *Two keyboards, one bug.* |
| `rubber_duck` | Rubber Duck | core | $3 | +50 chips if the hand contains a Pair | *Explains your bug back to you.* |
| `big_bang_release` | Big Bang Release | core | $3 | Every played card scores | *Every card ships. No feature flags.* |
| `dotfiles` | Dotfiles | core | $4 | +30 chips | *It works on my machine. Now it works on yours.* |
| `omakase` | Omakase | core | $4 | +5 mult | *You don't pick the menu. The menu picks you.* |
| `dual_monitors` | Dual Monitors | core | $4 | +10 mult if the hand contains a Two Pair | *Twice the screens, twice the tabs.* |
| `three_way_merge` | Three-Way Merge | core | $4 | +12 mult if the hand contains a Three of a Kind | *Ours, theirs, and somehow a third.* |
| `color_scheme` | Color Scheme | core | $4 | +10 mult if the hand contains a Flush | *Every window, the same theme.* |
| `split_keyboard` | Split Keyboard | core | $4 | +80 chips if the hand contains a Two Pair | *Two halves, both ergonomic.* |
| `triple_boot` | Triple Boot | core | $4 | +100 chips if the hand contains a Three of a Kind | *Arch, Arch, and also Arch.* |
| `pipeline` | Pipeline | core | $4 | +100 chips if the hand contains a Straight | *cat \| grep \| sort \| uniq -c* |
| `dark_mode` | Dark Mode | core | $4 | +80 chips if the hand contains a Flush | *Light mode is a bug report.* |
| `two_spaces` | Two Spaces | core | $4 | +4 mult for each scoring even card (10, 8, 6, 4, 2) | *Not tabs. Never tabs.* |
| `off_by_one` | Off By One | core | $4 | +31 chips for each scoring odd card (A, 9, 7, 5, 3) | *Arrays start at one. Fight me.* |
| `arch_wiki` | Arch Wiki | core | $4 | +20 chips and +4 mult for each scoring ace | *The answer was on the Arch Wiki all along.* |
| `green_build` | Green Build | core | $4 | +1 mult per hand played, −1 mult per discard | *All tests pass. Don't touch anything.* |
| `vintage_thinkpad` | Vintage ThinkPad | core | $4 | Sell value rises by $3 at the end of each blind | *Appreciating asset. Allegedly.* |
| `data_hoarder` | Data Hoarder | core | $4 | +$2 per discard left, if no discard was used this blind | *Never delete anything. Ever.* |
| `consulting_gig` | Consulting Gig | core | $4 | 1 in 2 chance of +$2 for each scoring face card | *Faces pay. Sometimes.* |
| `segfault` | Segfault | core | $4 | +0 to +23 mult, at random | *Undefined behavior, defined fun.* |
| `stable_release` | Stable Release | core | $4 | ×3 mult; 1 in 1000 chance to break at the end of each blind | *Boring. Reliable. Beautiful.* |
| `bang_bang` | Bang Bang | core | $4 | Retriggers the first scoring card 2 more times | *!! — run it again, louder.* |
| `ultrawide` | Ultrawide | core | $4 | +1 hand size | *More screen, more cards.* |
| `undo_history` | Undo History | core | $4 | +1 discard each blind | *Ctrl+Z, but for cards.* |
| `vim_golf` | Vim Golf | core | $4 | Creates a command if the hand contains an ace and a Straight | *Done in four keystrokes.* |
| `optional_dependencies` | Optional Dependencies | core | $4 | 1 in 2 chance to create a command when a pack is opened | *Optional, but you'll install them anyway.* |
| `emoji_reaction` | Emoji Reaction | core | $4 | +5 mult for each scoring face card | *A rocket on every pull request.* |
| `error404` | Error 404 | core | $4 | +10 chips and +4 mult for each scoring 10 or 4 | *Card not found. Scored anyway.* |
| `bug_bounty` | Bug Bounty | core | $4 | +$5 per discarded card of a rank drawn at each blind | *Report the right bug, get paid.* |
| `todo_md` | TODO.md | core | $4 | +$4 if the hand is of a type drawn at each blind | *One item. Changes every blind.* |
| `headhunter` | Headhunter | core | $4 | +$5 when 3 face cards or more are discarded at once | *Three managers out. Budget's back.* |
| `grid_layout` | Grid Layout | core | $4 | Gains +4 chips per hand of exactly 4 cards | *Four windows, not a pixel wasted.* |
| `plugin_manager` | Plugin Manager | core | $4 | +3 mult per joker | *lazy.nvim, 87 plugins, 12 ms startup.* |
| `homelab` | Homelab | core | $4 | Adds the sell value of your other jokers to mult | *Spare parts, adding up.* |
| `paru` | paru | core | $4 | One free reroll per shop | *Free reroll, now in Rust.* |
| `ruby_gem` | Ruby Gem | core | $5 | +3 mult for each scoring diamond | *Optimized for programmer happiness.* |
| `fan_mail` | Fan Mail | core | $5 | +15 chips for each scoring heart | *HEY, someone loves your pull request.* |
| `rails_world` | Rails World | core | $5 | +30 chips for each scoring face card | *Everyone came for the keynote. And the opinions.* |
| `tiling_manager` | Tiling Manager | core | $5 | +12 mult if the hand is a straight | *Floating windows are a lifestyle choice.* |
| `cloud_exit` | Cloud Exit | core | $5 | +$1 per discard left at the end of the blind | *Why rent what you can own?* |
| `open_source_love` | Open Source Love | core | $5 | +3 mult for each scoring heart | *Made with ♥ and too much coffee.* |
| `rabbit_hole` | Rabbit Hole | core | $5 | +3 mult for each scoring spade | *Just one more config tweak.* |
| `linux_user_group` | Linux User Group | core | $5 | +3 mult for each scoring club | *Pizza, slides, and a broken projector.* |
| `low_battery` | Low Battery | core | $5 | Adds twice the rank of the lowest card held in hand to mult | *Still at 2%. Still going.* |
| `free_trial` | Free Trial | core | $5 | +100 chips, −5 chips per hand played | *Enjoy it while it lasts.* |
| `muscle_memory` | Muscle Memory | core | $5 | Adds the number of times the hand type was played to mult | *Your fingers type it before you think.* |
| `ci_runner` | CI Runner | core | $5 | Gains +15 chips when the hand contains a Straight | *Green checks, one after another.* |
| `beta_release` | Beta Release | core | $5 | +15 mult; 1 in 6 chance to break at the end of each blind | *Works great until it doesn't.* |
| `auto_update` | Auto Update | core | $5 | 1 in 4 chance to level up the played hand type | *pacman -Syu, every hand.* |
| `support_contract` | Support Contract | core | $5 | Enterprise cards give +$4 when scored | *Billed per incident.* |
| `free_tier` | Free Tier | core | $5 | Creates a command when a hand is played with $4 or less | *Rate-limited, but it works.* |
| `man8` | man 8 | core | $5 | 1 in 4 chance per scoring 8 to create a command | *System administration commands.* |
| `minimal_install` | Minimal Install | core | $5 | Gains +3 mult per pack skipped | *Fewer packages, more speed.* |
| `profile_picture` | Profile Picture | core | $5 | ×2 mult for the first scoring face card | *First impression: doubled.* |
| `queue_manager` | Queue Manager | core | $5 | +13 mult for each queen held in hand | *Queens waiting in line. Patiently.* |
| `empty_workspace` | Empty Workspace | core | $5 | +30 chips per discard left | *Workspaces 6 through 10, still empty.* |
| `inbox_zero` | Inbox Zero | core | $5 | +15 mult when no discard is left | *Everything discarded. Finally at peace.* |
| `hype_cycle` | Hype Cycle | core | $5 | +20 mult, −4 mult at the end of each blind | *Peak of inflated expectations.* |
| `free_ram` | Free RAM | core | $5 | +2 chips per card left to draw | *Unused RAM is wasted RAM.* |
| `uptime` | Uptime | core | $6 | +1 mult per hand in a row without a scoring face card | *847 days without a reboot.* |
| `sponsorware` | Sponsorware | core | $6 | +$4 at the end of each blind | *Thank you for your sponsorship.* |
| `shell_fu` | Shell Fu | core | $6 | +1 mult per command used this game | *Pipes, loops, one-liners.* |
| `middle_management` | Middle Management | core | $6 | 1 in 2 chance of +$1 for each face card held in hand | *Paid to be in the room.* |
| `transitive_dependencies` | Transitive Dependencies | core | $6 | When a blind starts, creates 2 core jokers (if there is room) | *You installed one. Got two more.* |
| `chaos_monkey` | Chaos Monkey | extra | $4 | Doubles every chance (1 in 4 becomes 2 in 4) | *Doubles the odds of everything.* |
| `incremental_build` | Incremental Build | extra | $5 | Every played card permanently gains +5 chips | *A little faster every time.* |
| `cron_job` | Cron Job | extra | $5 | Retriggers the effects of cards held in hand | *Runs again at midnight. And again.* |
| `rollback` | Rollback | extra | $5 | Sell it to disable the current boss blind's effect | *git revert HEAD. Everyone relax.* |
| `identicon` | Identicon | extra | $5 | Every card counts as a face card | *Every card gets a face. Even the twos.* |
| `release_train` | Release Train | extra | $5 | ×4 mult every 6 hands played | *Leaves every six hands. Packed.* |
| `last_commit` | Last Commit | extra | $5 | Retriggers every scoring card of the last hand of a blind | *Final hand. Everything runs twice.* |
| `compound_interest` | Compound Interest | extra | $5 | +$1 of interest per $5 you have (same cap) | *Einstein's favorite. Allegedly.* |
| `refresh_spam` | Refresh Spam | extra | $5 | Gains +2 mult per shop reroll | *F5 until something changes.* |
| `flatpak` | Flatpak | extra | $5 | Jokers you own can show up in the shop again | *The same app, installed three times.* |
| `btrfs_snapshot` | Btrfs Snapshot | extra | $5 | Saves a lost blind if you scored 25% of the target, then disappears | *Rolled back from the brink. Once.* |
| `super_key` | Super Key | extra | $6 | +2 mult for each scoring card | *Every good shortcut starts here.* |
| `watch_mode` | Watch Mode | extra | $6 | Retriggers every scoring face card | *Every face, hot-reloaded.* |
| `theme_collector` | Theme Collector | extra | $6 | Gains ×0.1 mult per theme applied | *37 themes installed, 1 in use.* |
| `unixporn` | r/unixporn | extra | $6 | +$1 at the end of each blind per unique theme applied | *Karma, but you can spend it.* |
| `blob_storage` | Blob Storage | extra | $6 | +25 chips per Binary Blob card in the deck | *S3 buckets all the way down.* |
| `flaky_ci` | Flaky CI | extra | $6 | Gains ×0.25 mult each time a Flaky Test card triggers | *Green on the third try.* |
| `rma_department` | RMA Department | extra | $6 | Gains ×0.75 mult per Overclocked card that breaks | *We've seen worse. Probably.* |
| `code_signing` | Code Signing | extra | $6 | When a blind starts, adds a random card with a random seal to the hand | *Signed, sealed, delivered.* |
| `alias_collection` | Alias Collection | extra | $6 | Creates a command when a blind starts (if there is room) | *alias ll='ls -la'. And 400 more.* |
| `seed_round` | Seed Round | extra | $6 | Creates an oligarchy card if the hand is a Straight Flush | *Straight to the top. Then raise again.* |
| `angel_investor` | Angel Investor | extra | $6 | If the first hand of a blind is a single 6, destroys it and creates an oligarchy card | *Sacrifices a six. Expects a unicorn.* |
| `skip_intro` | Skip Intro | extra | $6 | Gains ×0.25 mult per blind skipped | *Netflix taught us well.* |
| `deep_dive` | Deep Dive | extra | $6 | +50 chips for each scoring spade | *Three hours in, still reading source.* |
| `bit_twiddling` | Bit Twiddling | extra | $6 | Retriggers every scoring 2, 3, 4 and 5 | *Small values, flipped twice as hard.* |
| `split_brain` | Split Brain | extra | $6 | ×2 mult if a club and a card of another suit score | *Two nodes, both think they're leader.* |
| `rainbow_prompt` | Rainbow Prompt | extra | $6 | ×3 mult if all four suits score | *Starship, with every glyph enabled.* |
| `pitch_black` | Pitch Black | extra | $6 | ×3 mult if every card held in hand is a spade or a club | *Black on black. Readable to you only.* |
| `daily_driver` | Daily Driver | extra | $6 | ×2 mult for each scoring card matching a card drawn at each blind | *This one card. Today, anyway.* |
| `log_rotation` | Log Rotation | extra | $6 | Gains +3 chips per discarded card of a suit drawn at each blind | *Rotates daily. Keeps every line.* |
| `garage_sale` | Garage Sale | extra | $6 | If the first discard of a blind is a single card, destroys it and gives $3 | *Old hardware, cash only.* |
| `friday_deploy` | Friday Deploy | extra | $6 | ×3 mult on the last hand of a blind | *Last hand, no rollback, maximum mult.* |
| `warm_start` | Warm Start | extra | $6 | ×3 mult if the hand type was already played this blind | *Seen it this blind. Faster now.* |
| `dotfiles_bak` | dotfiles.bak | extra | $6 | Gains +2 mult per hand containing a Two Pair | *Two of everything. Just in case.* |
| `espresso_shot` | Espresso Shot | extra | $6 | Retriggers every scoring card, for 10 hands | *Everything twice, for ten hands.* |
| `energy_drink` | Energy Drink | extra | $6 | ×2 mult, −×0.01 mult per card discarded | *×2 now. Crash later.* |
| `swap_file` | Swap File | extra | $6 | +5 hand size, −1 at the end of each blind | *Plenty of room. Shrinks every reboot.* |
| `vertical_monitor` | Vertical Monitor | extra | $6 | +2 hand size, −1 hand each blind | *Taller view, fewer turns.* |
| `force_push` | Force Push | extra | $6 | When a blind starts, +3 hands and no discards | *No discards. No regrets. Push.* |
| `bit_rot` | Bit Rot | extra | $6 | +4 mult per card the deck has lost since the start | *Fewer cards, sharper teeth.* |
| `owned_servers` | Owned Servers | extra | $6 | +2 chips per $1 you have | *Paid once. Yours forever.* |
| `semver_inflation` | Semver Inflation | extra | $6 | +$1 sell value to every joker and consumable at the end of each blind | *1.0.0 to 47.2.0, same features.* |
| `hockey_stick` | Hockey Stick | extra | $6 | +$1 at the end of each blind, +$2 more after each boss | *Growth chart up and to the right.* |
| `sticker_pack` | Sticker Pack | extra | $6 | Sell it to win the badge badges: 2x | *Free stickers at every conference.* |
| `rm_rf_node_modules` | rm -rf node_modules | extra | $6 | When a blind starts, destroys the joker on its right and gains twice its sell value as mult | *Frees 2 GB. Feels like a win.* |
| `postmortem` | Postmortem | extra · hidden: Core Dump | $6 | Gains ×0.5 mult when a blind is won on its last hand | *We learned a lot. Mostly about failure.* |
| `single_binary` | Single Binary | extra · hidden: Bare Metal | $6 | ×3 mult if it is your only joker | *Statically linked, zero dependencies.* |
| `no_build` | No Build | extra | $7 | ×2 mult if the played hand has 3 cards or fewer | *Webpack? Never heard of her.* |
| `heartbleed` | Heartbleed | extra | $7 | 1 in 2 chance of ×1.5 mult for each scoring heart | *Your heart leaks, 64 KB at a time.* |
| `mvp` | MVP | extra | $7 | Flushes and Straights can be made with 4 cards | *Minimum viable flush.* |
| `closed_source` | Closed Source | extra | $7 | Adds a Binary Blob card to the deck when a blind starts | *Can't read it. Can't remove it.* |
| `enterprise_sales` | Enterprise Sales | extra | $7 | Scoring face cards become Enterprise cards | *Every face, upsold.* |
| `garbage_collector` | Garbage Collector | extra | $7 | Gains ×0.1 mult per enhanced card played, and removes the enhancement | *Frees memory. And your upgrades.* |
| `systemd` | systemd | extra | $7 | ×0.2 mult per Daemon card in the deck | *One daemon to rule them all.* |
| `incident_commander` | Incident Commander | extra | $7 | +$8 each time a boss blind's effect triggers | *Paged at 3 a.m. Billed at 9.* |
| `meetup_pizza` | Meetup Pizza | extra | $7 | +7 mult for each scoring club | *Free pizza, mandatory lightning talk.* |
| `gem_install` | Gem Install | extra | $7 | +$1 for each scoring diamond | *bundle install, then go make coffee.* |
| `fuzzy_finder` | Fuzzy Finder | extra | $7 | Hearts and diamonds count as each other, and so do spades and clubs | *fzf: close enough is a match.* |
| `fast_forward` | Fast-Forward | extra | $7 | Straights may skip one rank between two cards | *Skips a commit or two. No one checks.* |
| `backspace` | Backspace | extra | $7 | +3 discards each blind, −1 hand size | *Worn smooth from all the deleting.* |
| `pacman_cache` | Pacman Cache | extra | $7 | Gains ×0.25 mult per card added to the deck | *Every package you add, kept forever.* |
| `bootstrapped` | Bootstrapped | extra | $7 | +2 mult per $5 you have | *No VCs. Profitable since day one.* |
| `nine_nines` | Nine Nines | extra | $7 | +$1 per 9 in the deck at the end of each blind | *99.9999999% uptime, billed per nine.* |
| `distro_hopping` | Distro Hopping | extra | $7 | When a small or big blind starts, gains ×0.5 mult and destroys another random joker | *Wipe everything, reinstall, feel alive.* |
| `recursion` | Recursion | extra | $8 | +8 mult for each scoring A, 2, 3, 5 or 8 | *To understand recursion, see Recursion.* |
| `free_as_in_beer` | Free as in Beer | extra | $8 | Themes and base16 packs in the shop cost $0 | *Also free as in freedom.* |
| `config_reload` | Config Reload | extra | $8 | Levels up the hand type of the first discard of each blind | *Super+Shift+R: everything, but fresher.* |
| `minimal_waybar` | Minimal Waybar | extra | $8 | ×1 mult per empty joker slot (Minimal Waybar counts as empty) | *Less widgets, more wallpaper.* |
| `senior_engineer` | Senior Engineer | AUR | $7 | ×3 mult if the deck holds at least 16 enhanced cards | *Has seen every config.* |
| `bloatware` | Bloatware | AUR | $7 | +250 chips, −2 hand size | *Huge numbers. No room for anything.* |
| `hotwire` | Hotwire | AUR | $8 | ×3 mult if the hand contains a Three of a Kind | *Three frameworks walk into a monolith.* |
| `code_review` | Code Review | AUR | $8 | ×2 mult if the hand contains a Pair | *Looks good to me. Ship it.* |
| `quad_core` | Quad Core | AUR | $8 | ×4 mult if the hand contains a Four of a Kind | *All cores busy compiling Rust.* |
| `git_bisect` | Git Bisect | AUR | $8 | ×3 mult if the hand contains a Straight | *Good, bad, good, bad… found it.* |
| `monorepo` | Monorepo | AUR | $8 | ×2 mult if the hand contains a Flush | *Everything in one place. Even you.* |
| `benevolent_dictator` | Benevolent Dictator | AUR | $8 | ×1.5 mult for each king held in hand | *For life. Or until the fork.* |
| `two_factor_auth` | Two-Factor Auth | AUR | $8 | Gains +8 chips for each scoring 2 | *Every 2 adds a layer. Every layer, a prompt.* |
| `jq_filter` | jq Filter | AUR | $8 | Gains ×0.5 mult per jack discarded; resets after each blind | *Filters out the jacks. Keeps the value.* |
| `rolling_release` | Rolling Release | AUR | $8 | ×1.5 mult for each scoring card of a suit drawn at each blind | *Today's favorite suit, until the next update.* |
| `quine` | Quine | AUR | $8 | If the first hand of a blind is a single card, adds a copy of it to the deck and the hand | *Prints itself. Adds itself to the deck.* |
| `hipster_stack` | Hipster Stack | AUR | $8 | Gains ×0.2 mult per hand in a row that isn't your most played type | *Never the popular framework.* |
| `package_maintainer` | Package Maintainer | AUR | $8 | ×1.5 mult per extra joker you own | *Every extra package, lovingly maintained.* |
| `syncthing` | Syncthing | AUR | $8 | Sold after 2 blinds, copies another random joker | *Two blinds to sync, then a perfect copy.* |
| `hash_collision` | Hash Collision | AUR · hidden: Duplicate Detected | $8 | ×2 mult if the hand contains two identical cards (same rank and suit) | *Two files, one checksum. Uh oh.* |
| `integer_overflow` | Integer Overflow | AUR · hidden: Heat Death | $8 | Doubles its mult when a blind is won in one hand; back to ×1 otherwise | *MAX_INT + 1 = surprise.* |
| `orphan_cleanup` | Orphan Cleanup | AUR | $9 | Gains ×0.25 mult per card sold; resets after a boss | *pacman -Rns $(pacman -Qdtq)* |
| `fork` | Fork | AUR | $10 | Copies the ability of the joker on its right | *Same features. MIT licensed.* |
| `upstream` | Upstream | AUR | $10 | Copies the ability of the leftmost joker | *Tracks the first joker, always.* |
| `dhh` | DHH | Legendary | $20 | Played kings and queens each give ×2 mult | *Strong opinions, loosely held. Mostly strong.* |
| `the_primeagen` | ThePrimeagen | Legendary | $20 | Gains ×1 mult when a face card is destroyed | *Blazingly fast. Skill issue for everyone else.* |
| `mailing_list` | Mailing List | Legendary | $20 | Gains ×1 mult every 23 cards discarded | *23 replies later, still no consensus.* |
| `kill9` | kill -9 | Legendary | $20 | Disables every boss blind's effect | *No signal handlers. No mercy.* |
| `copy_on_write` | Copy-on-Write | Legendary | $20 | At the end of the shop, adds a Transparent copy of a random consumable | *Free copies until someone writes.* |
