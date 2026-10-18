# Coffee Timer — Documentation

Coffee Timer runs multi-step coffee brewing recipes. Each step counts down on
its own. At every step boundary the phone vibrates and plays a sound, and the
screen stays awake while a brew runs. Recipes and settings are stored
locally in SQLite.

The app is a Rust core driven through [rinf](https://github.com/cunarist/rinf)
by a Flutter GUI. Rust owns every rule and all state. Flutter renders what
Rust sends, forwards user intents, and performs platform effects when told
to.

## What the app does

- **Recipe list.** Lists every recipe with its coffee, water and total time
  (the coffee and water are the recipe's own stored grams),
  each drawn as a coffee-bag label. Short brews get a dark roast label, long
  ones a light crema label, and each recipe has its own paper texture.
  Tap a recipe to brew it. Long-press it to edit, duplicate or delete it.
  Duplicate opens the editor on a new recipe copied from it (description,
  coffee, water, and every step with its time and pour), named `V60 copy`;
  nothing is saved until **Save**, which adds it at the end of the list,
  and backing out leaves the list as it was. The `+` button
  creates a recipe. After a delete, from the list or the editor, a
  `Deleted V60` snackbar offers **Undo** for a few seconds; it puts the
  whole recipe back where it was. Only the last delete can be undone, and
  a restart forgets it.
- **Brew page.** Shows the recipe printed on its own coffee-bag label (the
  same look as its list card), a timer dial and the steps. Tap the dial to
  start the brew from step 1. Tap it while it runs to pause, and again to
  resume where it left off. When the last step ends the dial reads `DONE`
  with the time brewed (and `Poured to 250g`, the scale's final reading,
  if any step pours), every step is marked done, the phone buzzes twice
  and plays the sound, the screen may sleep again and the brew
  notification goes away. Tap the dial then to brew again. The Reset
  button under the dial, shown while a brew runs, is paused or is done,
  goes back to idle. Long-press a step to start from that step, paused,
  done or not. The dial sweeps through the current step, and its
  outer track and the step list mark each step done, current, paused or
  pending. While paused the sweep stops and fades, the countdown fades, the
  screen may sleep again, and the brew notification reads `Paused`.
  The coffee, water and ratio are the recipe's stored grams.
  Tap the coffee/water/ratio figures to change the ratio for this brew in
  steps of 0.5. The coffee stays the same and the water follows. Leaving the
  page goes back to the recipe's own ratio. While the ratio is changed,
  **Save to recipe** in the ratio sheet makes it the recipe's own: the
  recipe keeps its coffee and takes the water and step pours shown, the
  list shows the new water, and a running, paused or done brew carries on
  where it is. A recipe holds at most 2000 g of water, and each pour 1–2000
  g; a ratio that would go past that is not saved, and a snackbar says why
  (`Can't save this ratio: 2050 g of water is outside 1–2000 g`).
  Each step that pours water shows its grams in the step list, and while it
  runs or is paused the dial reads the scale total to pour up to, such as
  `Pour to 150g` (this step's pour plus every earlier one). An adjusted
  ratio scales every pour with the water.
  Under the dial, a running or paused brew previews the step after this
  one, `Next: Pour 2 · 0:45 · 100g` (its pour only if it pours), or reads
  `Last step`. Three seconds before a step ends the phone gives one short
  buzz, lighter than a step boundary's, so you can get ready to pour. A
  step of 3 s or less gets none, and pausing never repeats it. The last
  step gets one too; the end of the brew then buzzes twice as usual.
- **Editor.** Edits a recipe's name, description, coffee (1–200 g) and
  water (1–2000 g), and its timed steps. A new recipe starts at 15 g to
  250 g. The total
  and each step's `MM:SS` update live. Each step has an optional pour in
  grams (1–2000 g; blank pours nothing). When the step pours do not add up
  to the recipe's water, a hint says so; it is not an error, and the two
  are saved as typed. Preset buttons add "Add coffee", "Add water", "Wait"
  or blank steps, each with a blank pour. Drag a step by its handle to
  reorder it; it keeps its text, pour and errors, and the `Step N name`
  labels follow the new order. Field errors appear from the first save
  attempt onward.
- **Settings.** Sets the theme (light, dark or system) and one of ten accent
  colours.

## Repository layout

| Path | What it is |
|---|---|
| [`native/core`](../native/core) | `coffee_timer_core`: business logic and state. It has no rinf, no async runtime and no I/O. |
| [`native/store`](../native/store) | `coffee_timer_store`: SQLite persistence, migrations, and the UUID v7 id generator. |
| [`native/cli`](../native/cli) | `coffee_timer_cli`: a command-line head on core + store. |
| [`native/hub`](../native/hub) | `hub`: the rinf bridge and the entry point of the Rust side of the app. rinf requires this crate name. |
| [`lib/`](../lib) | The Flutter GUI: holders, pages, widgets, and the platform-effect performer. |
| `lib/src/bindings/` | Dart signal classes generated by `rinf gen`. They are gitignored. |
| [`test/`](../test) | Flutter tests: holders, pages, widgets and the effect performer. |
| [`assets/sounds/`](../assets/sounds) | The step-change sound. |
| [`assets/fonts/`](../assets/fonts) | Oswald and Fraunces (SIL OFL 1.1) for the recipe labels, with their licences. |
| [`docs/mockups/`](mockups) | The HTML mockup the recipe labels were designed from. |

[modules.md](modules.md) describes each module.

## Build and run

You need the Flutter SDK, a Rust toolchain and the rinf CLI
(`cargo install rinf_cli`).

The Dart signal bindings are generated and gitignored. On a fresh checkout,
and after any change under `native/hub/src/signals/`, run this at the repo
root:

```shell
rinf gen
```

Run it before `flutter analyze`, `flutter test` or `flutter run`. Without the
bindings, `package:coffee_timer/src/bindings/bindings.dart` does not exist.
`rinf gen --watch` is broken; always run it by hand.

```shell
flutter run
```

The app keeps its database at `<app documents>/coffee_timer.db`.

### Gates

```shell
cargo clippy --all-targets -- -D warnings
cargo test
flutter analyze
flutter test
```

The workspace denies `unwrap_used`, `expect_used`, `panic` and
`wildcard_imports` (see [`Cargo.toml`](../Cargo.toml)).

### CLI

The CLI runs the same controllers on the same database format as the app,
with no Flutter:

```shell
cargo run -p coffee_timer_cli -- --db /tmp/ct.db list
cargo run -p coffee_timer_cli -- --db /tmp/ct.db show espresso-uuid
cargo run -p coffee_timer_cli -- --db /tmp/ct.db show pourover-uuid --ratio-steps -2   # 1:16
cargo run -p coffee_timer_cli -- --db /tmp/ct.db show pourover-uuid --ratio-steps 3 --save  # 1:18, saved
cargo run -p coffee_timer_cli -- --db /tmp/ct.db new V60 --description "Clean" \
    --coffee 15 --water 250 --step Bloom=30@50 --step Pour=120@200
cargo run -p coffee_timer_cli -- --db /tmp/ct.db brew pourover-uuid --from 1 --fast
cargo run -p coffee_timer_cli -- --db /tmp/ct.db brew espresso-uuid --pause-at 10 --pause-for 5
cargo run -p coffee_timer_cli -- --db /tmp/ct.db duplicate pourover-uuid --name "Iced"
cargo run -p coffee_timer_cli -- --db /tmp/ct.db delete <id>
cargo run -p coffee_timer_cli -- --db /tmp/ct.db settings
cargo run -p coffee_timer_cli -- --db /tmp/ct.db settings theme dark
cargo run -p coffee_timer_cli -- --db /tmp/ct.db settings accent Teal   # or #AARRGGBB / #RRGGBB
```

- `--db <PATH>` is optional. If the file is missing, it is created and seeded
  with Espresso and Pour Over. The default path is
  `$XDG_DATA_HOME/coffee_timer/coffee_timer.db`, then
  `~/.local/share/coffee_timer/coffee_timer.db`, then `./coffee_timer.db`.
- `brew` ticks once per second until the last step ends, and prints the
  countdown, the next-step preview, each step's status and the effects
  (`[vibrate 100ms]` at a step boundary, `[vibrate 40ms]` for the
  pre-alert 3 s before a step ends, `[vibrate pattern 0/400/200/400ms]`, `[sound]`,
  `[screen on]`/`[screen off]`, `[notify …]`/`[notify off]`). Its last
  tick line is the done line: the time brewed, and the scale's final
  reading if any step pours, as in `done 2:30 (Poured to 250g)`.
  `--fast` skips the waits.
  `--from N` starts at a 0-based step. `--pause-at N` pauses after tick
  `N` (ignored if the brew has ended by then) and `--pause-for S` stays
  paused `S` seconds without ticking, then resumes; it prints `pause` and
  `resume` lines with their effects.
- `list` and `show` print each recipe's coffee and water grams; `show` also
  prints the ratio and each step's pour (scaled with `--ratio-steps`).
  `show --ratio-steps N --save` saves that ratio to the recipe, as "Save to
  recipe" does: it prints `upserted <id>`, then the page at the recipe's
  new own ratio. A ratio the recipe cannot hold is an error and saves
  nothing.
  `brew` prints a pouring step's target next to its countdown, then the
  next step, as in `00:30 Bloom (Pour to 50g)  Next: Pour · 2:00 · 200g`.
- `new --coffee GRAMS --water GRAMS` sets the dose (1–200 g and
  1–2000 g). Without them the recipe gets the editor's 15 g and 250 g.
- `--step NAME=SECONDS[@GRAMS]` splits at the last `=`, then the seconds at
  an `@`; `@GRAMS` is the water the step pours. A blank name is saved as
  `Step N`.
- `duplicate <id> [--name NAME]` saves a copy at once, at the end of the
  list, and prints `upserted <new id>`. Without `--name` it is named
  `<name> copy`.
- Use `--help` on any command for its options.
- There is no undo command. The removed recipe is held in memory only, and
  each CLI run is one command, so nothing is left to undo by the next run;
  core's `catalog_test.rs` covers delete → undo instead.

## Contents

| Doc | What's in it |
|---|---|
| [architecture.md](architecture.md) | How it fits together: the layers, controllers and state, the catalog-change flow, effects, the timer, the snapshot handshake, startup and shutdown, how errors reach the user, and the Flutter side |
| [modules.md](modules.md) | A module-by-module guide to each crate and Dart directory, with its key types and tests |
| [rinf-bridge.md](rinf-bridge.md) | The rules for the Rust↔Flutter bridge: signal conventions, the one-listener rule, testing per layer, and rinf gotchas |
| [port-plan.md](port-plan.md) | The plan for porting the legacy Dart app, kept as the record of decisions and of every legacy bug fixed |
| [code-principles.md](code-principles.md) | Rules for simple, readable code that the project follows |

[`CLAUDE.md`](../CLAUDE.md) holds the project's architectural commitments
and gates.
