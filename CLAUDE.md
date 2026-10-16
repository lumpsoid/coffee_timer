# Coffee Timer

Runs multi-step coffee brewing recipes: a per-step countdown with a sound and
vibration at each step boundary, screen kept awake while brewing. Recipes and
settings are stored locally.

@docs/code-principles.md

## Stack

- **Rust** — all business logic and state.
- **Flutter** — GUI and platform effects only. No business logic in Dart.
- **[rinf](https://github.com/cunarist/rinf)** — the Rust↔Flutter bridge
  (local clone: `/home/pi/projects/rinf`). Layering and signal rules:
  [rinf bridge](./docs/rinf-bridge.md).

## Architectural commitments

1. Rust owns state. It sends **diffs** to Flutter as `RustSignal`s; Dart
   rebuilds view state from those signals. Dart sends user intents as
   `DartSignal`s.
2. The core is usable without Flutter: exercised by `cargo test` and a CLI.
   The core is built and tested first; the Flutter GUI comes after.
3. Port behavior verbatim: step bounds, brew metrics, labels, and timer
   behavior must match the current app exactly. Structure is free.
4. No panics in Rust (`unwrap`/`expect`); return `Result`.

## Gates

- `cargo clippy --all-targets` and `cargo test`
- `rinf gen` after changing any signal struct
- `flutter analyze` / `flutter test` for the GUI

## Behavior reference

The current Dart app is the spec for *what* to build, not how.

- `lib/domain/` — value rules and `BrewMetrics`; port these exactly.
- [docs/README.md](./docs/README.md), [architecture.md](./docs/architecture.md),
  [modules.md](./docs/modules.md) — how the legacy app behaves today,
  including the timer in `RecipeActor`, and its known quirks.
