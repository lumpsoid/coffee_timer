# Port plan: Rust core + rinf + Flutter GUI

> The port is complete (phases 0–9 committed). This plan is kept as the
> record of decisions and fixes; for how the app works today see
> [architecture.md](architecture.md) and [modules.md](modules.md).

Port the current Dart app to a Rust core driven through rinf, with Flutter as
a GUI that decides nothing. Behavior is the spec (see `CLAUDE.md`); structure
is new. Legacy bugs are fixed, not ported. There are no installs to migrate.

## Core model: feature controllers over state objects

Each feature has one **owner controller** holding one **state object**. An
action takes plain values, mutates the state, and returns only what state
cannot hold. A test runs an action and asserts the state.

Dependencies enter two ways:

- **Infrastructure** (store, id generator) is constructor-injected as
  `Box<dyn Trait>`. The traits live in core (`ports/`); `coffee_timer_store`
  implements them (`SqliteStore`, `UuidIds`); tests use the fakes in core's
  `fakes/` (`Vec`-backed store, sequential ids).
- **Shared controllers** (`RecipeCatalog`) are passed as action arguments
  (`&` / `&mut`), so every cross-feature dependency is in a signature and the
  borrow checker proves who reads and who writes. Injecting them would need
  `Rc<RefCell>`/`Arc<Mutex>` and turn borrow conflicts into runtime errors.

The controller that owns data owns its persistence. There is no coordinator
object: a flow is owned by the controller of the feature that starts it.

```
native/core/src/
  model/            RecipeId, StepId, RecipeName, StepDuration, RecipeStep,
                    Recipe, BrewMetrics, AccentColor, ThemeSelection, Settings
  ports/            RecipeStore, SettingsStore, IdGen, StoreError
  fakes/            FakeRecipeStore, FakeSettingsStore, SequentialIds
                    (cfg(test) / feature `test-fakes` only)
  catalog/          RecipeCatalog       state: Vec<Recipe>         (shared)
                    CatalogChange, RecipeSummary
  settings/         SettingsController  state: Settings
  brew/             BrewController      state: BrewState
  editor/           EditorController    state: EditorState
```

| Controller | Actions (shape, not final) | Returns |
|---|---|---|
| `RecipeCatalog` (store) | `load()`; snapshot via `state()` | `Result<(), StoreError>` |
| | `upsert(recipe)`, `remove(id)` | `Result<CatalogChange, StoreError>` |
| `SettingsController` (store) | `load()`, `set_theme`, `set_accent` | `Result<(), StoreError>` |
| `BrewController` | `open(&id, &catalog)`, `tick()`, `reset`, `leave`, `on_catalog_change(&change, &catalog)` | `Vec<Effect>` |
| | `start(from)` | `Result<Vec<Effect>, BrewError>` |
| `EditorController` (ids) | `new_draft`, `close` | — |
| | `edit(id, &catalog)`, `set_name`, `set_description`, `set_step_name(row, raw)`, `set_step_duration(row, raw)`, `remove_step(row)` | `Result<(), EditError>` |
| | `add_step(preset)` | `Result<StepId, EditError>` |
| | `save(&mut catalog)` | `Result<CatalogChange, SaveError>` |
| | `delete(&mut catalog)` | `Result<CatalogChange, DeleteError>` |

```rust
let mut catalog = RecipeCatalog::new(Box::new(FakeRecipeStore::default()));
let mut editor = EditorController::new(Box::new(SequentialIds::default()));
editor.new_draft();
editor.set_name("V60")?;
let change = editor.save(&mut catalog)?;
assert_eq!(catalog.state().len(), 1);
assert_eq!(change, CatalogChange::Upserted(RecipeId::try_parse("r1")?));
```

Draft mutators return `Err(EditError)` when no draft is open or the row id is unknown.

`CatalogChange` is returned after the catalog has persisted it. Each shell
passes it on with no conditions:

1. send `RecipeListUpdate::of(&change, &catalog)` — GUI diff, the row read
   back from the catalog
2. `brew.on_catalog_change(&change, &catalog)` — brew decides whether its recipe moved

Effects are one-shot instructions: `Vibrate { ms }`, `PlayStepSound`,
`KeepScreenOn(bool)`. Controllers never read the clock; time arrives as
`tick`.

## Signals

- The recipe list is a keyed, growing collection: `Snapshot` (built from
  `RecipeCatalog::state()`), then `Upserted` / `Removed` diffs built from
  each `CatalogChange` and the catalog (`RecipeListUpdate::of`); plus the
  one-shot `Failed` notice (see GUI fixes).
- Brew, editor, settings: after an action the hub sends the feature's **whole
  view state** (a few hundred bytes). Dart replaces, it does not fold.
- Labels (`20g`, `1:35`, `MM:SS`, `Ns`), step status (`Done` / `Current` /
  `Pending`), and field errors are computed in Rust and sent.

## Timer rules (`BrewController`)

Kept: start from step 0 or a chosen step; one tick per second; at a step
boundary vibrate + sound + next step; after the last step screen-on off,
ended, vibrate, no sound (since changed deliberately: see **Done** under
the deliberate changes); reset turns screen-on off; leaving resets silently.

Fixed / changed:

- Remaining never goes negative (`saturating_sub`). A 0-second step shows
  `00:00` for one tick, then advances.
- Out-of-range start index → `Err`.
- First tick exactly 1 s after start: the hub creates the interval per brew
  (`MissedTickBehavior::Delay`) and drops it on end.
- The brew view follows catalog changes. An edit to the open recipe during a
  running brew resets it; deleting it shows "Empty recipe".
- A catalog change to a recipe other than the open one leaves the brew view
  unchanged. In legacy, any create or update replaced the open page
  (`recipe_actor.dart:32-34,49-51`).
- Deleting the open recipe stops a running brew silently (screen-on off) and
  shows "Empty recipe". In legacy the timer kept running, vibrating and
  sounding behind "Empty recipe" (`recipe_actor.dart:53-58`).
- Opening a recipe stops any running brew silently. Legacy did not stop it
  (`recipe_actor.dart:60-62`); the UI couldn't reach this.
- `reset`/`leave` emit `KeepScreenOn(false)` only when a brew was running;
  the screen is kept on exactly while running (legacy always disabled the
  wakelock — not observable).
- Dropped: unused total-remaining counter, unreachable `99:60` cap,
  per-page audio preload/unload (Dart loads the sound once at startup).

## Domain fixes (`model/`)

- `AccentColor` hex parsing rejects a sign and whitespace (legacy
  `int.tryParse` accepted `+`/`-`/spaces; `-` gave a negative ARGB).
- Default theme is `System` everywhere (legacy `Settings.fallback` was light
  while missing storage gave system).
- `Recipe::with_step_removed_at` returns `Err` for an index past the end
  (legacy threw a `RangeError`).

## Catalog and settings fixes

- Settings and catalog writes go to the store first; a failed write leaves
  state unchanged and returns `Err` (legacy `settings_controller.dart`
  `_apply` updated state before an unawaited save and ignored failures).
- Recipes load in insertion order explicitly (legacy `SELECT * FROM recipe`
  had no `ORDER BY` and relied on rowid order implicitly). Phase 5: an
  explicit `recipe.position INTEGER PRIMARY KEY` (stable across `VACUUM`,
  unlike the implicit rowid) with `ORDER BY position` on load, and an upsert
  keeps the existing position (`ON CONFLICT DO UPDATE`, not
  `INSERT OR REPLACE`).
- Steps also load in order explicitly: `step.position` with
  `ORDER BY position` (legacy `sqlite_recipe_repository.dart:30,61` had no
  `ORDER BY`).

## Editor fixes

- Field errors are hidden until the first save attempt, then follow the
  text live (legacy left them stale until the next Save: no
  `autovalidateMode`).
- Durations parse as decimal only (legacy `int.tryParse` accepted `0x`
  hex).
- Saving a duration outside 0–5999 is rejected with `0–5999 seconds`
  (the legacy controller clamped it, unreachable through the page
  validator).
- Editing a recipe with no steps shows one blank row (≥1 row). Legacy
  showed zero rows and could save a recipe with zero steps.
- One editor: the legacy details page's edit button opened a stale second
  editor (`lib/recipe_edit/`: `Enter a name`, no upper bound, step ids
  reused by index). Phase 8 routes every edit to `EditorController`.
- Phase 8: the delete dialog shows the saved recipe's name, not the
  untrimmed typed text.

## GUI fixes (phase 8)

- A failed delete from the list shows a snackbar `Failed to delete
  recipe: …` (core's `DeleteError` text, sent as
  `RecipeListUpdate::Failed`); the list stays. Legacy replaced the whole
  list with the error text.
- The brew page's edit button is hidden on "Empty recipe". Legacy opened
  the stale editor there.
- The timer ring restarts on every step change and on a restart of the
  same step, keyed on `BrewTimer`'s `(run, step_index)`. Legacy's
  `ValueNotifier<Duration?>` only reset when the duration value changed.
- The database file is `coffee_timer.db`. Legacy used `recipes.db`; there
  are no installs to migrate.
- A new `RemoveRecipe` intent deletes from the list without opening a
  draft.

## Changed after the port

Deliberate changes to behavior the port had kept.

- **Real coffee and water amounts (schema v2).** Legacy `BrewMetrics`
  derived the figures from the step count: 10 g coffee and 50 g water per
  step, so the ratio was always `1:5`. A recipe now stores its coffee dose
  (`CoffeeDose`, 1–200 g) and total water (`WaterAmount`, 1–2000 g),
  edited in the editor with the other fields' live validation. The list
  card, the brew page label, the ratio, and the brew page's ratio adjuster
  all use the stored values. The v2 migration gives each existing recipe
  what the old formula showed (a recipe the formula would put outside the
  bounds, such as one with no steps, reads clamped into them). A new
  database seeds Espresso 18 g / 36 g and Pour Over 15 g / 250 g. Because a
  dose is never 0 g, the brew page's "no coffee, no ratio" case
  (`RatioSetting::NoCoffee`, `BrewError::NoCoffee`) is gone.
- **Pause and resume.** Legacy (and the port) reset a running brew when the
  dial was tapped. Tapping it now pauses (`BrewState::Paused`, keeping the
  step and its time left) and tapping again resumes. Reset moved to a Reset
  button under the dial, shown while a brew runs or is paused, rather than
  to a long-press on the dial: long-press already means "start from here"
  on the steps, and a visible button is discoverable where a second
  gesture on the same dial is not. While paused the screen may sleep and
  the notification stays, reading `Paused`. Every rule about stopping
  (reset, leave, open, an edit or delete of the open recipe) treats a
  paused brew like a running one.
- **Undo a delete.** Legacy deletes were final, and both delete dialogs
  said "This action cannot be undone." A delete, from the list or the
  editor, now offers a `Deleted V60` snackbar with Undo, which restores
  the whole recipe at its old place; the dialogs now ask only
  `Delete "V60"?`. Only the last removal can be undone, and it is held in
  memory, not stored.
- **Done.** Legacy (and the port) went back to idle after the last step,
  so an ended brew looked like one never started, and it ended with the
  same single 100 ms buzz as a step boundary, with no sound. The brew now
  ends in `BrewState::Finished`: the dial reads `DONE`, the time the run
  brewed (from the step it started at; the recipe's total for a full brew)
  and, when any step pours, the scale's final reading (`Poured to 250g`,
  at the ratio shown); every step is marked done. Finishing returns
  screen-on off, a double long buzz (`VibratePattern [0, 400, 200, 400]`),
  the step sound, then the notification hidden (last, so the platform keeps
  the app awake for the buzz and sound). Tapping the dial while done brews
  again from step 1, in one tap, as it does while idle; the Reset button
  (shown while done) goes back to idle; long-pressing a step starts there.
  A ratio change keeps a finished brew finished, as it keeps idle idle;
  an edit to the open recipe shows the new version idle, and a delete
  shows "Empty recipe", with no effects, exactly as while idle.

## Phases

| # | Phase | Gate |
|---|---|---|
| 0 | Cargo workspace; `rinf template`; crates `core`, `store`, `cli`, `hub`; deny `unwrap_used`/`expect_used` | clippy + test |
| 1 | `model/`: port `lib/domain/` rules and `BrewMetrics` exactly; port `test/domain/*` | `cargo test` |
| 2 | Traits `RecipeStore`, `SettingsStore`, `IdGen` + test fakes; `RecipeCatalog`, `SettingsController` (10 accent options as data) | `cargo test` |
| 3 | `BrewController`, tick-by-tick tests | `cargo test` |
| 4 | `EditorController`: rows keyed by row id, live labels, field errors, strict save, ≥1 row, presets "Add coffee" / "Add water" / "Wait" | `cargo test` |
| 5 | `store`: implements `RecipeStore` / `SettingsStore` with rusqlite (bundled), schema v1 (`recipe`, `step` with `position`, `settings`) seeded with Espresso and Pour Over, `user_version` for later migrations, in-memory round-trip tests | `cargo test` |
| 6 | `cli` on controllers + store: list / show / new / delete / brew / settings | `cargo run -p coffee_timer_cli` |
| 7 | `hub`: `Boot { db_dir }`, `<Feature>Ready` → state, uuid v7 `IdGen` (`coffee_timer_store::UuidIds`, shared with the CLI), one task owning all controllers, per-brew interval | `rinf gen`, clippy |
| 8 | Flutter: one holder per feature; keep display widgets (`CircularTimer`, `StepListTile`, `RecipeSummaryTile`, `FixedDigit`); one `PlatformEffect` performer; delete legacy Dart and unused packages | `flutter analyze` / `flutter test` + device run |
| 9 | Rewrite `docs/README.md`, `architecture.md`, `modules.md` | — |

`lib/domain/` remains the behavior reference until phase 8.
