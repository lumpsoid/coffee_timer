# Changelog

All notable changes to Coffee Timer. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions
follow [Semantic Versioning](https://semver.org/). Before 1.0.0, a minor
version may change behaviour.

## [Unreleased]

## [0.3.1] - 2026-11-13

### Fixed

- 0.3.0 was tagged but its APKs failed to build, so it was never
  published. This release carries everything listed under 0.3.0.

### Changed

- Built with Flutter 3.50.0. No change in behaviour is intended; the
  app still runs on Android 7.0 (API 24) and later.

## [0.3.0] - 2026-11-13

### Added

- Eight new label papers: graph, ledger, dot grid, crosshatch,
  herringbone, ripple, woodgrain and laid.
- The recipe editor has a label paper gallery. Pick one of the fourteen
  papers, plain paper, or Auto, which keeps the paper the app picks for
  the recipe. A copied recipe keeps its chosen paper.
- Export and import carry a recipe's chosen paper. Recipes without one
  export exactly as before.
- CLI: `new --texture NAME` sets a recipe's paper, and `show` prints the
  paper it is printed on.

### Changed

- A recipe that has not chosen a paper may look different: with more
  papers to pick from, its paper, shade and coffee ring can all move. A
  recipe that chose a paper keeps it, but its shade and ring can still
  move.
- The coffee ring is only on the brew page, now a larger stain sized to
  the label. Recipe cards in the list no longer show one.

## [0.2.0] - 2026-11-09

### Added

- Settings > Sounds: choose the sound between steps from seven short
  sounds, and the one at the end of a brew from twelve longer ones that
  ring out and resolve, each shown with its length. Tapping a sound
  previews it, even with sound off.
- A brew ends with a knock you can feel: "shave and a haircut, two bits"
  in vibration, played in time, softer than before and with each note at
  its own strength (on phones that can vary it). Settings > Finish knock
  turns it off for the old two long buzzes.
- CLI: `settings step-sound`, `settings finish-sound` and
  `settings finish-knock on|off`.

### Changed

- The end-of-brew singing bowl is 3 s and rings down to silence instead of
  being cut off.
- Every sound plays at the same loudness, so changing the sound does not
  make the cue louder or quieter at one volume. The wooden knocks were
  about 11 dB quieter than the counter bell.

### Fixed

- The settings page scrolls instead of overflowing on a short screen.

## [0.1.0] - 2026-11-06

The first public release.

### Added

- Multi-step brewing recipes with a per-step countdown, a sound and a
  vibration at each step boundary, an optional pre-alert before a step
  ends, and the screen kept awake while brewing.
- A brew page with a timer dial: pause and resume, start from any step,
  reset, and a Done state with the time brewed.
- Change the coffee dose or the ratio for one brew, and save it to the
  recipe.
- Per-step water pours, with a "pour to" target while brewing.
- An Android notification that keeps a running brew alive in the
  background.
- A recipe editor: steps with durations and pours, drag to reorder.
- A recipe list drawn as coffee-bag labels: pin, reorder, duplicate,
  delete with undo.
- Import and export recipes as JSON through the clipboard.
- Settings: theme, accent colour, sound, vibration, pre-alert, and the
  app version.
- A command-line interface over the same core and database.
