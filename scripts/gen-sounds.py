#!/usr/bin/env python3
"""Synthesize the brew sounds into assets/sounds/ as mono 128 kb/s MP3s.

Every sound is generated here from damped sine modes and band-limited noise,
so the files carry no third-party license. The noise is seeded: a rerun
writes the same audio.

Needs numpy and the `lame` encoder. Run from the repo root:

    python3 scripts/gen-sounds.py
"""

import subprocess
import tempfile
import wave
from pathlib import Path

import numpy as np

SR = 44100
OUT = Path(__file__).resolve().parent.parent / "assets" / "sounds"
rng = np.random.default_rng(7)


def t_of(dur):
    return np.arange(int(SR * dur)) / SR


def modes(dur, freq, partials, attack=0.002, sweep=None):
    """Damped sine modes [(ratio, amp, decay)]. `sweep(t)` scales frequency over time."""
    t = t_of(dur)
    f = freq * (sweep(t) if sweep else 1.0)
    sig = np.zeros_like(t)
    for ratio, amp, decay in partials:
        phase = 2 * np.pi * np.cumsum(np.broadcast_to(f * ratio, t.shape)) / SR
        sig += amp * np.sin(phase) * np.exp(-t * decay)
    return sig * np.minimum(1.0, t / attack)


def noise_band(dur, lo, hi):
    """White noise band-limited in the frequency domain."""
    n = int(SR * dur)
    spec = np.fft.rfft(rng.standard_normal(n))
    f = np.fft.rfftfreq(n, 1 / SR)
    spec[(f < lo) | (f > hi)] = 0
    return np.fft.irfft(spec, n)


def place(total, parts):
    """Mix (start_seconds, signal) pairs into one buffer of `total` seconds."""
    buf = np.zeros(int(SR * total))
    for start, sig in parts:
        i = int(SR * start)
        k = min(len(sig), len(buf) - i)
        buf[i : i + k] += sig[:k]
    return buf


def write(name, sig, peak_db=-3.0, tail=0.15):
    """Normalize to `peak_db`, cosine-fade the last `tail` seconds, encode to MP3."""
    sig = sig / np.max(np.abs(sig)) * 10 ** (peak_db / 20)
    n = int(SR * tail)
    sig[-n:] *= 0.5 * (1 + np.cos(np.linspace(0, np.pi, n)))
    with tempfile.TemporaryDirectory() as tmp:
        wav = Path(tmp) / f"{name}.wav"
        with wave.open(str(wav), "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(SR)
            w.writeframes((sig * 32767).astype(np.int16).tobytes())
        subprocess.run(
            ["lame", "--quiet", "-m", "m", "-b", "128", str(wav), str(OUT / f"{name}.mp3")],
            check=True,
        )


# Temple block (mokugyo): one hollow wooden knock.
WOOD = [(1, 1.0, 22), (2.57, 0.35, 40), (4.2, 0.12, 60)]
knock = modes(0.6, 520, WOOD, sweep=lambda t: 1 + 0.06 * np.exp(-t * 60))
knock[: int(SR * 0.004)] += 0.4 * noise_band(0.004, 1500, 6000)
write("temple-block", place(1.0, [(0, knock)]))

# Wooden clappers (hyoshigi): two dry claps, the second a little softer.
CLAP = [(1, 1.0, 35), (1.48, 0.6, 45), (2.9, 0.3, 70)]
clap = modes(0.4, 1250, CLAP)
clap[: int(SR * 0.003)] += 0.5 * noise_band(0.003, 2000, 9000)
write("wooden-clappers", place(1.0, [(0, clap), (0.2, 0.7 * clap)]))

# Tingsha: two small cymbals struck together; detuned so they shimmer.
TING = [(1, 1.0, 3.5), (2.32, 0.3, 6), (4.1, 0.1, 9)]
ting = modes(1.0, 2350, TING) + modes(1.0, 2361, TING)
write("tingsha", place(1.0, [(0, ting)]), peak_db=-5, tail=0.35)

# Water drop: a drop, then a smaller echo drop.
DROP = [(1, 1.0, 28), (2.1, 0.15, 45)]
def rise(t):
    return 1 + 1.2 * (1 - np.exp(-t * 55))


drop = modes(0.4, 620, DROP, attack=0.001, sweep=rise)
small = modes(0.4, 900, DROP, attack=0.001, sweep=rise)
write("water-drop", place(1.0, [(0, drop), (0.23, 0.45 * small)]))

# Kalimba: two plucked tines a fifth apart.
KAL = [(1, 1.0, 5), (5.4, 0.25, 30), (11.1, 0.06, 55)]
write("kalimba", place(1.0, [(0, modes(0.9, 587.33, KAL)), (0.16, 0.85 * modes(0.84, 880.0, KAL))]), tail=0.3)

# Breath and bowl: a soft breath-like swell resolving into a low bowl.
t = t_of(0.45)
breath = noise_band(0.45, 300, 1800) * np.sin(np.pi * t / 0.45) ** 2
BREATH_BOWL = [(1, 1.0, 2.6), (1.004, 0.8, 2.6), (2.71, 0.25, 4.5)]
bowl = modes(0.7, 440, BREATH_BOWL, attack=0.02)
write("breath-and-bowl", place(1.0, [(0, 0.12 * breath / np.max(np.abs(breath))), (0.3, bowl)]), tail=0.3)

# Bamboo (shishi-odoshi): a hollow tock with a short low resonance after.
BAMBOO = [(1, 1.0, 18), (1.9, 0.4, 28), (3.3, 0.15, 45)]
tock = modes(0.6, 330, BAMBOO, sweep=lambda t: 1 + 0.12 * np.exp(-t * 40))
tock[: int(SR * 0.003)] += 0.5 * noise_band(0.003, 1000, 5000)
write("bamboo-tock", place(1.0, [(0, tock)]))

# Singing bowl: A5 with a slow beat between two close partials, 2 s.
SINGING_BOWL = [(1, 1.0, 2.4), (1.004, 0.8, 2.4), (2.71, 0.3, 3.84), (5.1, 0.08, 6.0)]
write("singing-bowl", modes(2.0, 880.0, SINGING_BOWL, attack=0.012), tail=0.7)
