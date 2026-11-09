# Changelog

All notable changes to Coffee Timer. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions
follow [Semantic Versioning](https://semver.org/). Before 1.0.0, a minor
version may change behaviour.

## [Unreleased]

### Added

- Settings > Sounds: choose the sound between steps and the one at the
  end of a brew from eight sounds, each shown with its length. Tapping a
  sound previews it, even with sound off.

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
