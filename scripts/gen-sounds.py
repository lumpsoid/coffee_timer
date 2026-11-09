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

# Singing bowl: A5 with a slow beat between two close partials, 3 s: long
# enough to ring down to near silence before the fade.
SINGING_BOWL = [(1, 1.0, 2.4), (1.004, 0.8, 2.4), (2.71, 0.3, 3.84), (5.1, 0.08, 6.0)]
write("singing-bowl", modes(3.0, 880.0, SINGING_BOWL, attack=0.012), tail=1.2)

# --- End of brew ---
# A step sound marks one boundary of many; the finish marks the only end. So
# each finish sound is longer, rings out, and resolves (a falling cadence, a
# slowing rhythm, a strike let ring), and is long enough to ring down to
# near silence before the fade.

BOWL = [(1, 1.0, 1.6), (1.004, 0.8, 1.6), (2.71, 0.3, 2.8), (5.1, 0.08, 5.0)]
KAL_RING = [(1, 1.0, 3.2), (5.4, 0.25, 30), (11.1, 0.06, 55)]
TING_RING = [(1, 1.0, 2.2), (2.32, 0.3, 3.5), (4.1, 0.1, 5)]
BELL = [(1, 1.0, 2.2), (2.76, 0.45, 3.5), (5.40, 0.2, 6), (8.93, 0.08, 9)]
TUBE = [(1, 1.0, 2.0), (2.76, 0.35, 3.0), (5.40, 0.12, 4.5)]
D4, D5, FS5, A5 = 293.66, 587.33, 739.99, 880.0

# Bowl, struck twice: a second softer strike while the first still rings.
write("bowl-twice", place(4.6, [(0, modes(4.6, 660, BOWL, attack=0.012)),
                                 (0.7, 0.75 * modes(3.9, 660, BOWL, attack=0.012))]), tail=1.5)

# Bowl cadence: a high bowl, then a low one a fifth below — V to I.
write("bowl-cadence", place(4.6, [(0, 0.8 * modes(4.6, 659.25, BOWL, attack=0.012)),
                                   (0.55, modes(4.05, 440.0, BOWL, attack=0.015))]), tail=1.5)

# Kalimba cadence: three tines falling 5-3-1 in D, closing on a low D under
# the tonic.
write("kalimba-cadence", place(2.6, [
    (0.00, 0.85 * modes(1.0, A5, KAL_RING)),
    (0.18, 0.85 * modes(1.0, FS5, KAL_RING)),
    (0.36, modes(2.2, D5, KAL_RING) + 0.7 * modes(2.2, D4, KAL_RING)),
]), tail=0.8)

# Kalimba rise: a D-major arpeggio up to the octave, the top note let ring.
write("kalimba-rise", place(2.6, [
    (0.00, 0.8 * modes(1.0, D5, KAL_RING)),
    (0.13, 0.8 * modes(1.0, FS5, KAL_RING)),
    (0.26, 0.85 * modes(1.0, A5, KAL_RING)),
    (0.42, modes(2.1, 2 * D5, KAL_RING) + 0.5 * modes(2.1, D5, KAL_RING)),
]), tail=0.8)


# Tingsha, three times: the cymbals struck three times, the last let ring.
def tingsha_ring(dur):
    return modes(dur, 2350, TING_RING) + modes(dur, 2361, TING_RING)


write("tingsha-three", place(3.8, [(0, 0.6 * tingsha_ring(0.5)), (0.35, 0.6 * tingsha_ring(0.5)),
                                    (0.7, tingsha_ring(3.1))]), peak_db=-5, tail=1.3)

# Counter bell: the service bell on a café counter, rung twice.
write("counter-bell", place(3.4, [(0, 0.8 * modes(3.4, 2093, BELL, attack=0.001)),
                                   (0.28, modes(3.1, 2093, BELL, attack=0.001))]), peak_db=-4, tail=1.2)

# Wind chimes: five pentatonic tubes brushed in a falling run, each left to
# ring.
CHIMES = [(0.00, 1567.98), (0.11, 1318.51), (0.24, 1174.66), (0.40, 987.77), (0.60, 783.99)]
write("wind-chimes", place(4.0, [(s, (0.7 + 0.3 * i / 4) * modes(4.0 - s, f, TUBE, attack=0.003))
                                  for i, (s, f) in enumerate(CHIMES)]), peak_db=-4, tail=1.3)

# Last drops: the brew's last drips slowing to a stop, the final one a
# deeper drop that rings into the cup.
drips = [(0.0, 700, 0.8), (0.22, 820, 0.6), (0.52, 640, 0.7), (0.95, 760, 0.5)]
parts = [(s, a * modes(0.4, f, DROP, attack=0.001, sweep=rise)) for s, f, a in drips]
parts.append((1.5, modes(0.5, 420, DROP, attack=0.001, sweep=rise)
              + 0.25 * modes(0.5, 880, [(1, 1.0, 6)], attack=0.01)))
write("last-drops", place(2.4, parts), tail=0.4)

# Gong: a low gong whose upper partials bloom a moment after the strike,
# sliding a little flat as it rings.
t = t_of(4.4)
low = modes(4.4, 196, [(1, 1.0, 1.6), (1.52, 0.5, 2.0), (2.31, 0.3, 2.4)], attack=0.02,
            sweep=lambda t: 1 - 0.015 * (1 - np.exp(-t * 2)))
bloom = modes(4.4, 196, [(3.17, 0.35, 1.8), (4.45, 0.25, 2.5), (5.9, 0.12, 3.2)], attack=0.02)
bloom *= 1 - np.exp(-t * 4)
strike = np.zeros_like(t)
strike[: int(SR * 0.01)] = 0.2 * noise_band(0.01, 200, 2000)
write("gong", low + bloom + strike, tail=1.4)

# Spoon on cup: a teaspoon tapping a ceramic cup, two light taps then a
# firmer one that rings — the stir is done.
CUP = [(1, 1.0, 3.0), (1.007, 0.5, 3.0), (2.24, 0.4, 5), (3.9, 0.15, 8), (5.8, 0.06, 12)]


def tap(dur, amp):
    sig = modes(dur, 1480, CUP, attack=0.0005)
    sig[: int(SR * 0.002)] += 0.6 * noise_band(0.002, 3000, 12000)
    return amp * sig


write("spoon-on-cup", place(3.2, [(0, tap(0.6, 0.45)), (0.16, tap(0.6, 0.4)),
                                  (0.5, tap(2.7, 1.0))]), peak_db=-4, tail=1.3)

# Music box: a comb's tines plucked in a falling phrase, A-F#-E-D, the last
# held over its octave below.
COMB = [(1, 1.0, 2.6), (3.0, 0.12, 9), (5.9, 0.05, 16)]
A6, FS6, E6, D6 = 1760.0, 1479.98, 1318.51, 1174.66
write("music-box", place(3.9, [
    (0.00, 0.7 * modes(1.2, A6, COMB, attack=0.001)),
    (0.20, 0.7 * modes(1.2, FS6, COMB, attack=0.001)),
    (0.40, 0.7 * modes(1.2, E6, COMB, attack=0.001)),
    (0.70, modes(3.2, D6, COMB, attack=0.001) + 0.5 * modes(3.2, D6 / 2, COMB, attack=0.001)),
]), peak_db=-4, tail=1.3)
