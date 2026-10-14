# Commands

<!-- Generated from the engine code by `make docs`: do not edit by hand. -->

Commands are consumables (tarots, in the original): bought in the shop for $3,
they wait in the clipboard (2 slots) until applied. Those that change cards
need them selected in the hand, during a blind; the others work at any time.

| Id | Command | Effect | Cards | Quip |
|---|---|---|---|---|
| `up_arrow` | `Up Arrow` | Creates the last command or theme used (not Up Arrow) |  | *History is a flat circle.* |
| `npm_test` | `npm test` | Up to 2 selected cards become Flaky Test | 1 to 2 | *Works on the second run.* |
| `theme_install` | `omarchy-theme-install` | Creates up to 2 random themes |  | *The gallery is bigger than you think.* |
| `make_j` | `make -j` | Up to 2 selected cards become Multithreaded | 1 to 2 | *-j$(nproc), obviously.* |
| `omarchy_menu` | `omarchy-menu` | Creates up to 2 random commands |  | *Super+Alt+Space: everything's in there.* |
| `cargo_release` | `cargo build --release` | Up to 2 selected cards become Optimized | 1 to 2 | *Ten minutes to save ten milliseconds.* |
| `ln_s` | `ln -s` | 1 selected card becomes a Symlink | 1 | *Two paths, one heart.* |
| `systemctl_enable` | `systemctl enable` | 1 selected card becomes a Daemon | 1 | *Starts at boot. Forever.* |
| `cpupower` | `cpupower` | 1 selected card becomes Overclocked | 1 | *Performance governor. Fans: on.* |
| `rsync` | `rsync ~/wallet` | Doubles your money (+$20 at most) |  | *Incremental backups of your money.* |
| `hyprctl_reload` | `hyprctl reload` | 1 in 4 chance to give a random joker Blur, Shadow or Animated |  | *Nope. Nope. Nope. Oh, nice blur!* |
| `npm_version_patch` | `npm version patch` | Raises the rank of up to 2 selected cards by one | 1 to 2 | *1.0.0 → 1.0.1, ship it.* |
| `rm_rf` | `rm -rf` | Destroys up to 2 selected cards | 1 to 2 | *No undo. No regrets.* |
| `cp_f` | `cp -f` | Of 2 selected cards, the left one becomes a copy of the right one | 2 | *Overwrite without asking.* |
| `du_sh` | `du -sh ~/jokers` | Gives the total sell value of your jokers ($50 at most) |  | *Worth more than you think.* |
| `accept_eula` | `Accept EULA` | 1 selected card becomes Enterprise | 1 | *Nobody reads it. Everybody signs it.* |
| `strip` | `strip` | 1 selected card becomes a Binary Blob | 1 | *Debug symbols? Never heard of them.* |
| `mv_diamonds` | `mv * ♦/` | Up to 3 selected cards become diamonds | 1 to 3 | *Shiny new folder.* |
| `mv_clubs` | `mv * ♣/` | Up to 3 selected cards become clubs | 1 to 3 | *Members only.* |
| `mv_hearts` | `mv * ♥/` | Up to 3 selected cards become hearts | 1 to 3 | *Moving in with love.* |
| `yay_s` | `yay -S` | Creates a random joker (needs a free slot) |  | *Straight from the AUR. What could go wrong?* |
| `mv_spades` | `mv * ♠/` | Up to 3 selected cards become spades | 1 to 3 | *Dig a little deeper.* |
