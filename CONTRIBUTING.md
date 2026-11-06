# Contributing

Thanks for wanting to help with Coffee Timer. It is a small personal
project, so there is no formal process, only a request to make things
go smoothly for both of us.

## Start with an issue

If you have an idea or want to change something, please **open an issue
first** ([Proposal](https://github.com/lumpsoid/coffee_timer/issues/new?template=proposal.yml)) and describe
the problem and what you have in mind. We can talk it through there:
whether it fits the app, and how it could be built. Once we agree on the
shape, open a pull request that links the issue.

This is a wish, not a rule: nothing stops you from opening a pull request
straight away. Talking first just saves you from building something that
then needs a different approach. Small, obvious fixes (a typo, a broken
link, a clear bug with a one-line fix) can go straight to a pull request.

Bugs are welcome as issues too ([Bug report](https://github.com/lumpsoid/coffee_timer/issues/new?template=bug_report.yml));
the version at the foot of the Settings page helps.

## Setting up

You need:

- [Flutter](https://docs.flutter.dev/get-started/install) (stable, Dart
  `^3.9.2`) with the toolchain for your target. `flutter doctor` checks it.
- [Rust](https://www.rust-lang.org/tools/install), stable.
- The rinf CLI, at the version of the `rinf` crate in `Cargo.lock`:
  `cargo install rinf_cli --version 8.11.0`.

```shell
git clone https://github.com/lumpsoid/coffee_timer.git && cd coffee_timer
flutter pub get
rinf gen          # Dart bindings; gitignored, so needed on every fresh checkout
scripts/check.sh  # every gate
flutter run
```

Run `rinf gen` again after any change under `native/hub/src/signals/`.
More on building, the CLI and the launcher icon:
[docs/README.md](docs/README.md#build-and-run).

## How the code is laid out

Rust owns every rule and all state; Flutter renders what Rust sends and
forwards user intents. If a widget needs to decide something, that
decision belongs in `native/core`. The core runs without Flutter: `cargo
test` and the CLI exercise it.

Most changes go down and back up the layers:

1. **Rule or state** → a controller or model in [`native/core`](native/core),
   tested there with `cargo test`.
2. **Persistence** → [`native/store`](native/store); a schema change is a
   new migration, never an edit to an old one.
3. **New data for the GUI** → a signal in
   [`native/hub/src/signals`](native/hub/src/signals) and its mapping in
   `native/hub/src/<feature>.rs`; then `rinf gen`.
4. **New user intent** → a `DartSignal` with one receiver in
   [`native/hub/src/run.rs`](native/hub/src/run.rs).
5. **Display** → the feature's holder in [`lib/holders`](lib/holders) and
   page in [`lib/pages`](lib/pages).

Read [docs/architecture.md](docs/architecture.md) for how it fits
together and [docs/code-principles.md](docs/code-principles.md) for the
code style. [CLAUDE.md](CLAUDE.md) lists the architectural commitments.

## Before you open a pull request

- `scripts/check.sh` passes: `cargo fmt --check`, clippy, `cargo test`, `flutter analyze`,
  `flutter test`. CI runs the same.
- No `unwrap`, `expect` or `panic` in Rust: return a `Result`. Clippy
  enforces it.
- New behaviour has tests: core rules in `cargo test`, pages as Flutter
  widget tests.
- Behaviour changes are reflected in [docs/](docs) and in
  [CHANGELOG.md](CHANGELOG.md) under Unreleased.
- Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/),
  like the history: `feat(brew): …`, `fix(editor): …`, `docs: …`.

## License

Coffee Timer is under the [0BSD license](LICENSE). By contributing you
agree that your contribution is released under it too.
