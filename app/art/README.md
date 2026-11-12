# Joker drawings

Each drawn joker has `jokers/<id>.txt`: a square pixel art, one letter per
pixel, at the size it was drawn: 64 × 64 for every joker, so that their pixels
match. Letters are theme colours, not
fixed colours, so a drawing follows every Omarchy theme (`build.rs` turns it into an image, `shaders/palette.frag`
paints it). Jokers without a drawing keep their text card.

## Palette

The reference colours are Tokyo Night's (`palette.png`, `palette.hex`):
`art.py import` turns each into its letter. A drawing made with other colours
gets a letter per colour with `--map` (`art.py colors image` lists them).

| Letter | Role | Reference |
|---|---|---|
| `.` | transparent | |
| `k` | foreground (text) | `#a9b1d6` |
| `a` | accent | `#7aa2f7` |
| `m` | muted | `#414868` |
| `b` | background | `#1a1b26` |
| `w` | warm (yellow) | `#e0af68` |
| `r` | urgent (red) | `#f7768e` |
| `g` | green | `#9ece6a` |
| `c` | cyan | `#449dab` |
| `p` | magenta | `#ad8ee6` |
| `o` | orange | `#eb927b` |
| `d` | ink: always dark, whatever the theme | `#101017` |
| `l` | light: always light, whatever the theme | `#cbd0e6` |
| `A` `W` `R` `G` `C` `P` `O` | dark shades of accent, warm, urgent, green, cyan, magenta, orange | 35% towards black |

`k`, `m` and `b` swap between dark and light themes (`b` turns light, `k`
dark): fine for flat parts, wrong for lighting. Give outlines and dark lines
`d`, highlights `l`, shading the capitals: they keep their sense on every
theme. A drawing's identity colour follows the theme (a duck in `w` is the
theme's yellow); check it on light and dark themes with `preview`.

## Style

- One subject, centred, filling most of the frame, on a transparent
  background, in a front or three-quarter view; the same view for the whole
  series. It is shown at about 1.75 screen pixels per pixel: details under
  2 pixels are lost.
- Few colours (4 to 7), one of them a dark outline.
- Clean outlines, flat colours, a shade or two; no dithering, no gradients.
- No text or letters (unreadable at this size), no real logos or brands, no
  real people.
- The humour of the name and quip, affectionate. Original: never Balatro's
  look, never "in the style of" anything.

Jokers are not animated: a still drawing, one image.

## Workflow (Retro Diffusion)

1. Generate at 64 × 64 (an upscaled image is brought back to its own size),
   the background removed, with `palette.png` as the input palette if
   possible. Start the prompt with the style line of `briefs.md`. Generate
   several, keep one.
2. Import: `python3 art.py import <id> image.png --map 0f0c1d=k,...` (a
   GIF gives its first frame). The drawing is cut
   to 64 × 64 around its subject (`--crop`), so that it fills the card. A
   warning says when pixels are left far from the palette. Keep the source and
   the command in a `#` line at the top of the `.txt`, and what was touched up
   by hand, to import it again.
3. Check: `python3 art.py preview <id>...` renders it on four Omarchy themes
   (in `/tmp`, or `-o file`); then in the game,
   `OMATRO_DEBUG=1 OMATRO_JOKERS=<id> make run`.
4. Touch up by hand in the `.txt` if needed: it is the source.

`python3 art.py palette` writes the palette files again, should the palette
change (keep `build.rs`, `palette.frag` and `art.py` in step).

# Theme pictures

Each theme card has `themes/<id>.png` (the theme's id in the engine:
`matte_black`, `rose_pine`…): a 64 × 40 picture, the shape of a wallpaper,
in its own colours, never the current theme's (a Nord card stays Nord
blue). `build.rs` embeds them as they are; a theme card shows its picture
framed at its top, the collection's theme tiles beside the name.

They were made with Retro Diffusion from each theme's own wallpaper
(`~/.local/share/omarchy/themes/<theme>/backgrounds`):

| Theme | Wallpaper |
|---|---|
| Matte Black | `0-ship-at-sea` |
| Rosé Pine | `1-funky-shapes` |
| Ristretto | `1-color-curves` |
| Osaka Jade | the theme's background |
| Nord | `0-black-moon` |
| Everforest | `1-tree-tops` |
| Kanagawa | `1-kanagawa` |
| Catppuccin | `1-totoro` (a black cat, not the character) |
| Tokyo Night | `2-quattro` |
| Hackerman | `1-synth-scape` |
| Ethereal | `1-cosmic` |
| Flexoki Light | `1-orb` |
