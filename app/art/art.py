#!/usr/bin/env python3
"""Tools for the joker drawings of art/jokers/ (see art/README.md).

    art.py palette                     writes palette.hex and palette.png
    art.py colors IMAGE                lists its colours, and the nearest letters
    art.py import ID IMAGE [--map rrggbb=x,...] [--crop 64] [--key ...] [--outline k]
    art.py preview ID... [-o FILE] [--themes a,b,c]

A drawing is a square of palette letters (build.rs, ART_PALETTE), 64 × 64
(an upscaled image is brought back to its own size, then cut around its
subject). Jokers are not animated. The
reference palette is Tokyo Night's: draw (or generate) with exactly these
colours, and `import` turns each into its letter, i.e. its theme role, so that
the drawing follows every theme. Needs Pillow.
"""
import argparse
import math
import os
import re
import sys

from PIL import Image

HERE = os.path.dirname(os.path.abspath(__file__))
LETTERS = ".kambwrgcpodlAWRGCPO"
ROLES = ["transparent", "foreground", "accent", "muted", "background", "warm",
         "urgent", "green", "cyan", "magenta", "orange", "ink (always dark)",
         "light (always light)", "accent shade", "warm shade", "urgent shade",
         "green shade", "cyan shade", "magenta shade", "orange shade"]
# How far a shade goes towards black (shaders/palette.frag).
SHADE = 0.35

# The reference theme, Tokyo Night, as theme.rs reads it.
REFERENCE = {
    "foreground": "#a9b1d6", "accent": "#7aa2f7", "muted": "#414868",
    "background": "#1a1b26", "warm": "#e0af68", "urgent": "#f7768e",
    "green": "#9ece6a", "cyan": "#449dab", "magenta": "#ad8ee6",
    "orange": "#eb927b",
}


def rgb(h):
    h = h.lstrip("#")
    return tuple(int(h[i:i + 2], 16) for i in (0, 2, 4))


def mix(a, b, t):
    return tuple(round(x + (y - x) * t) for x, y in zip(a, b))


def colours(theme):
    """The colours of the palette (index 1 on) for a theme, as
    shaders/palette.frag computes them."""
    c = {k: rgb(v) for k, v in theme.items()}
    base = [c[k] for k in ("foreground", "accent", "muted", "background", "warm",
                           "urgent", "green", "cyan", "magenta", "orange")]
    luma = lambda x: 0.299 * x[0] + 0.587 * x[1] + 0.114 * x[2]
    dark = luma(c["background"]) < luma(c["foreground"])
    darker, lighter = (c["background"], c["foreground"]) if dark else (c["foreground"], c["background"])
    fixed = [mix(darker, (0, 0, 0), 0.4), mix(lighter, (255, 255, 255), 0.4)]
    shades = [mix(c[k], (0, 0, 0), SHADE)
              for k in ("accent", "warm", "urgent", "green", "cyan", "magenta", "orange")]
    return base + fixed + shades


def omarchy_theme(name):
    """An installed Omarchy theme, read with theme.rs's fallbacks."""
    path = os.path.expanduser(f"~/.local/share/omarchy/themes/{name}/colors.toml")
    keys = {}
    for line in open(path):
        m = re.match(r'\s*([a-z0-9_]+)\s*=\s*"(#[0-9a-fA-F]{6})', line)
        if m:
            keys[m.group(1)] = m.group(2)
    get = lambda *names, default=None: next((keys[n] for n in names if n in keys), default)
    fg = get("foreground", "color7", default="#cccccc")
    yellow = get("yellow", "color3", default="#c8a050")
    return {
        "foreground": fg,
        "background": get("background", "color0", default="#101010"),
        "accent": get("accent", "color4", default="#cacccc"),
        "muted": get("muted", "color8", default=fg),
        "warm": yellow,
        "urgent": get("red", "color1", default="#a55555"),
        "green": get("green", "color2", default="#7a9a60"),
        "cyan": get("cyan", "color6", default="#5a9aa0"),
        "magenta": get("magenta", "color5", default="#9a6a9a"),
        "orange": get("orange", default=yellow),
    }


# ---------------------------------------------------------------- palette

def cmd_palette(_args):
    cols = colours(REFERENCE)
    with open(os.path.join(HERE, "palette.hex"), "w") as f:
        for c in cols:
            f.write("%02x%02x%02x\n" % c)
    img = Image.new("RGB", (len(cols), 1))
    for i, c in enumerate(cols):
        img.putpixel((i, 0), c)
    img.save(os.path.join(HERE, "palette.png"))
    for letter, role, c in zip(LETTERS[1:], ROLES[1:], cols):
        print(f"  {letter}  #%02x%02x%02x  {role}" % c)


# ---------------------------------------------------------------- import

def native(frame):
    """An upscaled pixel art at its own size: the largest block size that its
    pixels all fill (a 256 px image drawn at 64 px gives 64)."""
    w, h = frame.size
    px = frame.load()
    for s in sorted((d for d in range(2, h + 1) if h % d == 0), reverse=True):
        if all(px[x, y] == px[x - x % s, y - y % s] for y in range(h) for x in range(w)):
            small = Image.new("RGBA", (w // s, h // s))
            for y in range(h // s):
                for x in range(w // s):
                    small.putpixel((x, y), px[x * s, y * s])
            return small
    return frame


def load(path):
    """An image at the size it was drawn. A GIF gives its first frame: jokers
    are not animated."""
    img = Image.open(path)
    if getattr(img, "n_frames", 1) > 1:
        print(f"{path}: {img.n_frames} frames: the first one is taken (jokers are not animated)",
              file=sys.stderr)
    img = img.convert("RGBA")
    w, h = img.size
    if w != h:
        sys.exit(f"{path}: {w}×{h}: a square image expected")
    return native(img)


def outline(rows, letter):
    """The background-coloured pixels on the drawing's edge (next to a
    transparent one) in `letter`: a dark outline vanishes on the card, which is
    tinted with the background; in the foreground, it shows on every theme."""
    n = len(rows)
    def clear(x, y):
        return not (0 <= x < n and 0 <= y < n) or rows[y][x] == "."
    return ["".join(letter if c == "b" and any(clear(x + dx, y + dy) for dx, dy in ((1, 0), (-1, 0), (0, 1), (0, -1)))
                    else c for x, c in enumerate(row)) for y, row in enumerate(rows)]


def parse_map(text):
    """--map 3b2a5c=m,fe7a6b=a: colours of the image and their letters."""
    pairs = {}
    for item in filter(None, (text or "").split(",")):
        colour, _, letter = item.partition("=")
        if letter not in LETTERS:
            sys.exit(f"--map: {item}: one of {LETTERS} expected after =")
        pairs[rgb(colour)] = letter
    return pairs


def crop(rows, size):
    """The drawing cut to size × size around what it draws, so that the
    subject fills the card and every drawing has the same pixel size."""
    n = len(rows)
    cells = [(x, y) for y, row in enumerate(rows) for x, c in enumerate(row) if c != "."]
    x0, x1 = min(x for x, _ in cells), max(x for x, _ in cells)
    y0, y1 = min(y for _, y in cells), max(y for _, y in cells)
    if x1 - x0 + 1 > size or y1 - y0 + 1 > size:
        sys.exit(f"the drawing is {x1 - x0 + 1} × {y1 - y0 + 1}: larger than --crop {size}")
    left = (x0 + x1 + 1 - size) // 2
    top = (y0 + y1 + 1 - size) // 2
    def at(x, y):
        return rows[y][x] if 0 <= x < n and 0 <= y < n else "."
    return ["".join(at(left + i, top + j) for i in range(size)) for j in range(size)]


def cmd_colors(args):
    img = load(args.image)
    n = img.size[0]
    print(f"{args.image}: {n} × {n}")
    cols = colours(REFERENCE)
    for count, c in sorted(img.getcolors(1 << 20), reverse=True):
        if c[3] < 128:
            print(f"  transparent  {count}")
            continue
        d, i = min((math.dist(c[:3], r), i) for i, r in enumerate(cols))
        print(f"  %02x%02x%02x  {count:6}  nearest: {LETTERS[i + 1]} ({ROLES[i + 1]}, {d:.0f} away)" % c[:3])


def cmd_import(args):
    cols = colours(REFERENCE)
    mapping = parse_map(args.map)
    img = load(args.image)
    n = img.size[0]
    key = None
    if args.key == "auto":
        corners = {img.getpixel(p) for p in [(0, 0), (n - 1, 0), (0, n - 1), (n - 1, n - 1)]}
        if len(corners) == 1 and next(iter(corners))[3] >= 128:
            key = next(iter(corners))[:3]
            print(f"background #%02x%02x%02x taken as transparent" % key)
    elif args.key != "none":
        key = rgb(args.key)
    off, worst, rows = 0, 0, []
    for y in range(n):
        row = ""
        for x in range(n):
            r, g, b, a = img.getpixel((x, y))
            if a < 128 or (key and math.dist((r, g, b), key) < 8):
                row += "."
            elif (r, g, b) in mapping:
                row += mapping[(r, g, b)]
            else:
                d, i = min((math.dist((r, g, b), c), i) for i, c in enumerate(cols))
                worst = max(worst, d)
                off += d > 24
                row += LETTERS[i + 1]
        rows.append(row)
    if args.crop:
        rows = crop(rows, args.crop)
    if args.outline:
        rows = outline(rows, args.outline)
    path = os.path.join(HERE, "jokers", f"{args.id}.txt")
    with open(path, "w") as f:
        f.write("\n".join(rows) + "\n")
    print(f"{path}: {len(rows)} × {len(rows)}")
    if off:
        print(f"warning: {off} pixel(s) far from the palette (up to {worst:.0f}): "
              "use the reference palette, or --map (see `art.py colors`)", file=sys.stderr)


# ---------------------------------------------------------------- preview

def read_art(art_id):
    return [line.rstrip("\n") for line in open(os.path.join(HERE, "jokers", f"{art_id}.txt"))
            if line.strip() and not line.startswith("#")]


def cmd_preview(args):
    names = args.themes.split(",")
    themes = [colours(omarchy_theme(n)) for n in names]
    arts = [read_art(i) for i in args.ids]
    scale, pad = args.scale, 4
    side = max(len(rows) for rows in arts) + 2 * pad
    cell = side * scale
    img = Image.new("RGB", (cell * len(themes), cell * len(arts)))
    for ti, cols in enumerate(themes):
        face = mix(cols[3], cols[1], 0.10)  # the card: background tinted with the accent
        for ai, rows in enumerate(arts):
            tile = Image.new("RGB", (side, side), face)
            off = (side - len(rows)) // 2
            for y, row in enumerate(rows):
                for x, ch in enumerate(row):
                    if ch != ".":
                        tile.putpixel((x + off, y + off), cols[LETTERS.index(ch) - 1])
            img.paste(tile.resize((cell, cell), Image.NEAREST), (ti * cell, ai * cell))
    out = args.output or "/tmp/omatro-art.png"
    img.save(out)
    print(f"{out}: {', '.join(args.ids)} on {', '.join(names)}")


def main():
    p = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = p.add_subparsers(dest="cmd", required=True)
    sub.add_parser("palette", help="write palette.hex and palette.png")
    i = sub.add_parser("import", help="turn images into art/jokers/ID.txt")
    i.add_argument("id")
    i.add_argument("image")
    i.add_argument("--map", help="colours and their letters: 3b2a5c=m,fe7a6b=a (others: nearest)")
    i.add_argument("--key", default="auto", help="background to make transparent: auto, none or #rrggbb")
    i.add_argument("--crop", type=int, default=64,
                   help="cut to N × N around the subject (default 64; 0: keep the size)")
    i.add_argument("--outline", choices=list(LETTERS[1:]), help="letter for the dark outline's edge pixels (k: foreground)")
    c = sub.add_parser("colors", help="list an image's colours and their nearest letters")
    c.add_argument("image")
    v = sub.add_parser("preview", help="render drawings on Omarchy themes")
    v.add_argument("ids", nargs="+")
    v.add_argument("-o", "--output")
    v.add_argument("--themes", default="tokyo-night,catppuccin-latte,gruvbox,rose-pine")
    v.add_argument("--scale", type=int, default=5)
    args = p.parse_args()
    {"palette": cmd_palette, "colors": cmd_colors, "import": cmd_import, "preview": cmd_preview}[args.cmd](args)


if __name__ == "__main__":
    main()
