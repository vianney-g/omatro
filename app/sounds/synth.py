#!/usr/bin/env python3
"""Writes the synthesized sound effects of app/sounds (the others come from
Kenney's CC0 packs). Run it from anywhere: python3 app/sounds/synth.py"""

import math
import random
import struct
import wave
from pathlib import Path

RATE = 44100
HERE = Path(__file__).parent


def write(name, samples):
    peak = max(1e-9, max(abs(s) for s in samples))
    scale = 0.7 / peak
    with wave.open(str(HERE / f"{name}.wav"), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(RATE)
        w.writeframes(b"".join(struct.pack("<h", int(s * scale * 32767)) for s in samples))


def silence(seconds):
    return [0.0] * int(seconds * RATE)


def tone(freq, seconds, decay=8.0, wave_shape="sine", attack=0.004):
    out = []
    n = int(seconds * RATE)
    for i in range(n):
        t = i / RATE
        phase = 2 * math.pi * freq * t
        if wave_shape == "square":
            v = 1.0 if math.sin(phase) >= 0 else -1.0
        elif wave_shape == "bell":
            v = math.sin(phase) + 0.35 * math.sin(2 * phase) + 0.15 * math.sin(3 * phase)
        else:
            v = math.sin(phase)
        env = min(1.0, t / attack) * math.exp(-decay * t)
        out.append(v * env)
    return out


def sweep(f0, f1, seconds, decay=10.0):
    out, phase = [], 0.0
    n = int(seconds * RATE)
    for i in range(n):
        t = i / RATE
        f = f0 + (f1 - f0) * (i / n)
        phase += 2 * math.pi * f / RATE
        env = min(1.0, t / 0.003) * math.exp(-decay * t)
        out.append((math.sin(phase) + 0.25 * math.sin(2 * phase)) * env)
    return out


def mix(*parts):
    """Each part is (start in seconds, samples)."""
    length = max(int(start * RATE) + len(s) for start, s in parts)
    out = [0.0] * length
    for start, s in parts:
        o = int(start * RATE)
        for i, v in enumerate(s):
            out[o + i] += v
    return out


def note(n):
    """Frequency of a MIDI note."""
    return 440.0 * 2 ** ((n - 69) / 12)


# Chips: a pluck per step, climbing a pentatonic scale.
PENTATONIC = [0, 2, 4, 7, 9]
for k in range(12):
    n = 72 + 12 * (k // 5) + PENTATONIC[k % 5]
    write(f"chips-{k}", tone(note(n), 0.18, decay=22, wave_shape="bell"))

# Mult: a low bloop; ×mult: a punchier rising one.
write("mult", sweep(330, 220, 0.16, decay=16))
write("x-mult", mix((0, sweep(220, 660, 0.22, decay=9)), (0.0, tone(110, 0.22, decay=12))))

# Blind won: the soft two-note chime of a "[  OK  ]".
write("blind-won", mix((0, tone(note(76), 0.5, decay=6, wave_shape="bell")),
                       (0.11, tone(note(83), 0.7, decay=5, wave_shape="bell"))))

# Victory: a boot chime, a chord rolled up.
write("victory", mix(*[(0.09 * i, tone(note(n), 1.8, decay=2.2, wave_shape="bell"))
                       for i, n in enumerate([60, 64, 67, 72, 76])]))

# Game over: the PC speaker's POST error (one long, two short), then the
# crackle of a kernel panic.
random.seed(42)
crackle = [random.uniform(-1, 1) * math.exp(-5 * i / RATE) * 0.35 for i in range(int(0.45 * RATE))]
beep = lambda s: tone(880, s, decay=0.5, wave_shape="square", attack=0.002)
write("game-over", beep(0.42) + silence(0.12) + beep(0.11) + silence(0.09) + beep(0.11)
      + silence(0.1) + crackle)

# A card destroyed: glass cracking, a noise burst over a falling sweep.
random.seed(7)
shards = [random.uniform(-1, 1) * math.exp(-28 * i / RATE) for i in range(int(0.25 * RATE))]
write("card-break", mix((0, shards), (0, sweep(1800, 500, 0.25, decay=14)),
                        (0.03, tone(note(88), 0.2, decay=25, wave_shape="bell"))))

# A card changed or added: a quick rising shimmer.
write("card-change", mix((0, tone(note(79), 0.25, decay=14, wave_shape="bell")),
                         (0.05, tone(note(86), 0.3, decay=12, wave_shape="bell")),
                         (0.1, tone(note(91), 0.35, decay=10, wave_shape="bell"))))
