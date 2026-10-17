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

1. Rust owns state, in feature controllers that each own one state object;
   shared data lives in a shared controller passed into actions. Rust sends
   view state to Flutter as `RustSignal`s (whole state per feature; diffs
   for the recipe list); Dart replaces its view state from them. Dart sends
   user intents as `DartSignal`s.
2. The core is usable without Flutter: exercised by `cargo test` and a CLI.
   If the CLI cannot do something without Flutter, logic leaked out of core.
3. Behavior carried over from the legacy app (step bounds, brew metrics,
   labels, timer behavior) is fixed; change it only deliberately. The legacy
   bugs fixed during the port are listed in the port plan; don't reintroduce
   them.
4. No panics in Rust (`unwrap`/`expect`); return `Result`.

## Gates

- `cargo clippy --all-targets -- -D warnings` and `cargo test`
- `rinf gen` after changing any signal struct, and on a fresh checkout
  before `flutter analyze` (the generated bindings are gitignored)
- `flutter analyze` / `flutter test` for the GUI

## Behavior reference

The port is complete; the code is the specification.

- [docs/README.md](./docs/README.md) — what the app does, build/run, gates, CLI.
- [docs/architecture.md](./docs/architecture.md) — controllers, catalog-change
  flow, timer, handshake, startup, error surfacing.
- [docs/modules.md](./docs/modules.md) — module-by-module guide.
- [docs/port-plan.md](./docs/port-plan.md) — record of port decisions and of
  every legacy bug fixed.
