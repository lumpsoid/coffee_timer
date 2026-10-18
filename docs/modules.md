# Module guide

This page has one section per crate and per Dart directory. Each section
says what the module owns, names its key types, and lists its tests. It
links files rather than restating them; the source is the specification.
For how the modules work together, see [architecture.md](architecture.md).

- [native/core](#nativecore--coffee_timer_core)
- [native/store](#nativestore--coffee_timer_store)
- [native/cli](#nativecli--coffee_timer_cli)
- [native/hub](#nativehub--hub)
- [lib/](#lib--flutter-gui)
- [test/](#test--flutter-tests)

---

## native/core — `coffee_timer_core`

All business rules and state. It has no dependencies. Its crate docs are in
[`lib.rs`](../native/core/src/lib.rs). It has one directory per concern and
one file per type.

### `model/`: values and entities

[`native/core/src/model`](../native/core/src/model)

Every value here is valid by construction, so nothing downstream checks it
again.

- **Ids and names:** `RecipeId` and `StepId` are non-empty and can be built
  from a `u128` as UUID text. `RecipeName` is trimmed and non-blank, with a
  lenient `from_storage` that maps blank to `Untitled`.
- **`StepDuration`** is a step's length, `0..=5999` seconds. It has two entry
  points: strict parsing for user input and lenient clamping for storage and
  live labels. It also produces the `MM:SS` and `Ns` labels.
- **`CoffeeDose`** (`1..=200` g) and **`WaterAmount`** (`1..=2000` g) are a
  recipe's stored coffee and total water. Like `StepDuration`, each has
  strict text parsing (`Enter grams`, `1–200 g` / `1–2000 g`) and lenient
  clamping for storage. `WaterAmount` is a general amount of water: a
  step's pour is one too, with optional text entry (blank is no pour).
- **`RecipeStep` and `Recipe`** are immutable and change only through named
  operations. A recipe holds its `CoffeeDose` and `WaterAmount`; a step
  holds an optional `WaterAmount`, the water it pours. A blank step name is
  saved as `Step N`. The step pours need not add up to the recipe's water.
- **`PourScale`** is the one rounding rule for step pours in a brew whose
  water differs from the recipe's: it scales the running totals by brew
  water over recipe water, half up to a gram, and each `Pour` is the
  difference between two totals, so pours always add up to their "pour to"
  targets. `Pour` is one step's grams and its running total, with the
  `50g` and `Pour to 150g` labels.
- **`BrewMetrics`** is the one home for the figures derived from a recipe:
  the ratio of its stored coffee and water (`1:5`), and the capped total
  duration with its `M:SS` label.
- **`BrewRatio`** is water per gram of coffee, in tenths, with its label.
  The brew page moves it in steps of 0.5 within `1:1..=1:30` and works out
  the water for a dose of coffee.
- **`LabelStyle`** is how a recipe looks in the list, as a coffee-bag label.
  `LabelTone` (roast, caramel or crema) follows total brew time, with the
  thresholds as named constants. `LabelTexture`, `LabelShade` and
  `StainCorner` come from an FNV-1a hash of the recipe id, so a recipe keeps
  its look across renames, reorders and builds.
- **Settings:** `ThemeSelection` (light, dark or system), `AccentColor` (ARGB,
  with `#AARRGGBB` and `#RRGGBB` parsing), `AccentOption` (the ten named
  accents), and `Settings`. `Settings` has a per-field `from_storage`
  fallback.
- **`ModelError`** carries user-facing texts that match the legacy app's.

Tests: [`recipe_test.rs`](../native/core/tests/recipe_test.rs),
[`pour_scale_test.rs`](../native/core/tests/pour_scale_test.rs),
[`label_style_test.rs`](../native/core/tests/label_style_test.rs) and
[`settings_test.rs`](../native/core/tests/settings_test.rs).

### `ports/`: infrastructure traits

[`native/core/src/ports`](../native/core/src/ports)

This is the border with what cannot run in a test.

- `RecipeStore`: `load_all`, `upsert` and `remove`. Recipe order is part of
  the contract.
- `SettingsStore`: `load` and `save`.
- `IdGen`: `recipe_id` and `step_id`.
- `StoreError`: a backend's reason, carried as text.

Every trait requires `Send`, so the controllers can move into the hub task.
The store crate implements all three.

### `fakes/`: test doubles

[`native/core/src/fakes`](../native/core/src/fakes)

- `FakeRecipeStore` is `Vec`-backed and keeps the order contract.
- `FakeSettingsStore` holds settings in memory.
- Both can be told to fail loads or writes.
- `SequentialIds` produces `r1, r2, …` and `s1, s2, …`.

The module compiles only under `cfg(test)` or the `test-fakes` feature. Core
enables that feature for its own integration tests through a self
dev-dependency, and the CLI and hub enable it for their tests. Test:
[`ids_test.rs`](../native/core/tests/ids_test.rs).

### `catalog/`: the shared recipe list

[`native/core/src/catalog`](../native/core/src/catalog)

- `RecipeCatalog` owns `Vec<Recipe>` and writes through its `RecipeStore`
  before it changes its state. It provides `load`, `upsert`, `remove`, `get`
  and `state`.
- `CatalogChange` (`Upserted` / `Removed`) is returned only after the write
  has persisted.
- `RecipeSummary` is a recipe-list row with its labels (`20 g`, `100 g`,
  `1:35`) and its `LabelStyle`.

Test: [`catalog_test.rs`](../native/core/tests/catalog_test.rs).

### `settings/`: user settings

[`native/core/src/settings`](../native/core/src/settings)

`SettingsController` owns `Settings`. It provides `load`, `set_theme` and
`set_accent`, and writes through its `SettingsStore` first.

Test:
[`settings_controller_test.rs`](../native/core/tests/settings_controller_test.rs).

### `brew/`: the brew page and its timer

[`native/core/src/brew`](../native/core/src/brew)

- `BrewController` owns `BrewState` and runs the timer state machine. It
  provides `open`, `start`, `tick`, `pause`, `resume`, `reset`, `leave`,
  `adjust_ratio` and `on_catalog_change`.
- `BrewState` is `Empty`, `Idle(BrewView)`, `Running(BrewView,
  RunningStep)` or `Paused(BrewView, RunningStep)`.
- `BrewView` is the open recipe with its labels, its `LabelStyle` (the
  brew page wears the list card's label), `Step N of M` positions, and each
  step's `Pour` at the water shown (`pour_to_label(index)`). Its
  `RatioSetting` is `Recipe` or `Adjusted`; `RatioChange` is
  one press of the page's ratio controls.
- `RunningStep` is the current (or paused) step, the time left, the run
  number, and the dial's sweep.
- `StepRow` and `StepStatus` (`Done` / `Current` / `Paused` / `Pending`) describe the
  step list. Each row also carries its arc on the dial's step track, as
  fractions of the whole brew, and its pour label if it pours.
- `Effect` is a platform command. `BrewNotice` is the content of the brew
  notification.
- `BrewError` is a rejected start, pause, resume or ratio change.

Test:
[`brew_controller_test.rs`](../native/core/tests/brew_controller_test.rs),
which runs the timer tick by tick.

### `editor/`: the recipe editor

[`native/core/src/editor`](../native/core/src/editor)

- `EditorController` owns `EditorState` (`Closed` / `Open(Draft)`) and an
  `IdGen`. It provides `new_draft`, `edit`, the field setters (including
  `set_step_water`), `add_step`
  (which returns the new row's `StepId`), `remove_step`, `move_step(row, to)`
  (`to` is the 0-based index the row ends at; past the last row is
  `EditError::StepPositionOutOfRange`), `save`, `delete` and `close`.
- `Draft` is the whole editor view: the raw text of each field (name,
  description, coffee and water grams, a new draft starting at 15 g to
  250 g), at least one
  step row, the live total, the pour hint (`Step pours add up to 200 g of
  250 g`), field errors (hidden until the first save attempt), and the
  store-failure notice. The private `Field<T>` keeps the
  typed text and its parse result together.
- `EditorStepRow` is one step card: name, duration and pour text, with
  their labels and errors.
- `StepPreset` covers the add-step buttons: "Add coffee", "Add water",
  "Wait" and blank.
- `DraftMode` says whether the draft is new or existing.
- The error types are `EditError`, `SaveError` and `DeleteError`.

Test:
[`editor_controller_test.rs`](../native/core/tests/editor_controller_test.rs).

---

## native/store — `coffee_timer_store`

[`native/store/src`](../native/store/src)

The store implements the core's ports. It makes no decisions.

- **`SqliteStore`**
  ([`sqlite_store.rs`](../native/store/src/sqlite_store.rs)) is one
  migrated rusqlite connection (SQLite bundled). It implements both
  `RecipeStore` and `SettingsStore`. Clones share the connection, so the
  catalog and the settings controller each get a clone. Foreign keys are
  turned on after migrating.
- **Migrations**
  ([`migrations.rs`](../native/store/src/migrations.rs)) are keyed by
  `PRAGMA user_version` and run in one transaction. A database newer than
  the build is refused. v1 creates `recipe` (with an explicit `position`),
  `step` (with `position`) and `settings`, and seeds Espresso and Pour Over
  once. v2 adds `recipe.coffee_grams` and `recipe.water_grams`: an upgraded
  recipe gets what the old step-count formula showed (10 g coffee and 50 g
  water per step), while a database created at v2 gives the seeds real
  doses (Espresso 18 g / 36 g, Pour Over 15 g / 250 g). A migration entry
  can carry such `fresh` SQL, run only on a database created in that
  migration. v3 adds the nullable `step.water_grams` (NULL is no pour, so
  every upgraded step pours nothing); a database created at v3 gives the
  Pour Over seed's Bloom 50 g and Pour 200 g.
- **Row mapping:** [`recipe_rows.rs`](../native/store/src/recipe_rows.rs)
  loads `ORDER BY position`. An upsert keeps the recipe's position and
  replaces its steps. Rows are read through core's lenient constructors
  (grams are clamped into bounds).
  [`settings_rows.rs`](../native/store/src/settings_rows.rs) stores key/value
  rows.
- **`UuidIds`** ([`uuid_ids.rs`](../native/store/src/uuid_ids.rs)) is the
  production `IdGen`, which produces UUID v7 text. The hub and the CLI share
  it.

Tests: [`sqlite_store_test.rs`](../native/store/tests/sqlite_store_test.rs)
covers schema and seed, order, round trips, fallback, and controllers over
the real store. [`uuid_ids_test.rs`](../native/store/tests/uuid_ids_test.rs)
covers the id generator. `migrations.rs` also has unit tests.

---

## native/cli — `coffee_timer_cli`

[`native/cli/src`](../native/cli/src)

A second head on core and store. It proves that core needs no rinf. It
decides nothing: core produces every label, status and error text.

- [`args.rs`](../native/cli/src/args.rs) defines the clap arguments: `list`,
  `show`, `new`, `delete`, `brew` and `settings`, plus the global `--db`.
- [`lib.rs`](../native/cli/src/lib.rs) provides `Controllers::boot` (build
  and load), `run` (dispatch one command) and `default_db_path`.
- [`commands.rs`](../native/cli/src/commands.rs) has one function per
  command, plus `pass_on` for catalog changes. `new` fills a draft as the
  editor page would. `delete` goes through the editor.
- [`render.rs`](../native/cli/src/render.rs) lays out state, effects and
  field errors as text.
- [`cli_error.rs`](../native/cli/src/cli_error.rs) wraps core's errors, and
  [`main.rs`](../native/cli/src/main.rs) maps them to the exit code.

Test: [`commands_test.rs`](../native/cli/tests/commands_test.rs) runs each
command against an in-memory seeded store with sequential ids and checks
the printed output. See [README.md](README.md#cli) for usage.

---

## native/hub — `hub`

[`native/hub/src`](../native/hub/src)

The rinf bridge. The crate name `hub` is fixed by rinf's build presets. It
makes no business decisions.

- [`lib.rs`](../native/hub/src/lib.rs) is the runtime entry point. It
  defines `boot_then_run`, which waits for `Boot`, opens
  `<db_dir>/coffee_timer.db`, sends `BootFailed` on error, and otherwise
  runs the hub task.
- [`run.rs`](../native/hub/src/run.rs) defines `Controllers::boot` (called
  from `lib.rs`) and runs the hub task, which is one `select!` loop over
  every Dart signal and the brew tick. It also holds `pass_on`, `report`
  (log and continue), and the per-brew `Interval`.
- [`signals/`](../native/hub/src/signals) holds the signal types, one file
  per feature: `boot`, `recipe_list`, `brew` (including `PlatformEffect`),
  `editor` and `settings`. These types are the Dart contract. Run `rinf gen`
  after any change.
- [`brew.rs`](../native/hub/src/brew.rs),
  [`editor.rs`](../native/hub/src/editor.rs),
  [`recipe_list.rs`](../native/hub/src/recipe_list.rs) and
  [`settings.rs`](../native/hub/src/settings.rs) hold the pure mappings
  between core state and signals.

| Feature | Dart → Rust | Rust → Dart |
|---|---|---|
| boot | `Boot` | `BootFailed` |
| recipe list | `RecipeListReady`, `RemoveRecipe` | `RecipeListUpdate` (`Snapshot` / `Upserted` / `Removed` / `Failed`) |
| brew | `BrewReady`, `OpenBrew`, `StartBrew`, `PauseBrew`, `ResumeBrew`, `ResetBrew`, `AdjustBrewRatio`, `LeaveBrew` | `BrewUpdate` (`Empty` / `Idle` / `Running` / `Paused`), `PlatformEffect` |
| editor | `EditorReady`, `NewDraft`, `EditRecipe`, `SetRecipeName`, `SetRecipeDescription`, `SetRecipeCoffee`, `SetRecipeWater`, `SetStepName`, `SetStepDuration`, `SetStepWater`, `AddStep`, `RemoveStep`, `MoveStep`, `SaveRecipe`, `DeleteRecipe`, `CloseEditor` | `EditorUpdate` (`Closed` / `Open`) |
| settings | `SettingsReady`, `SetTheme`, `SetAccent` | `SettingsUpdate` |

Tests: unit tests in `brew.rs` and `recipe_list.rs` cover the non-trivial
mappings. The loops are not unit tested; they are kept thin enough to
read.

---

## lib/ — Flutter GUI

[`lib/`](../lib)

The GUI renders, sends intents, and performs effects. It holds no business
`if`.

- [`main.dart`](../lib/main.dart) starts Rust, connects the effect
  performer and every holder, then boots Rust.
- [`app.dart`](../lib/app.dart) holds `CoffeeTimerApp`, which builds the
  theme from settings, shows the spinner, the boot failure with Retry, or
  the recipe list, and calls `finalizeRust()` on exit.
- **[`holders/`](../lib/holders)** has one `ChangeNotifier` per feature:
  `RecipeListHolder` (folds the snapshot and diffs, and exposes `failures`
  and `byId`), `BrewHolder`, `EditorHolder`, `SettingsHolder` (each replaces
  its state) and `BootHolder` (sends `Boot` and holds the boot failure).
- **[`effects/`](../lib/effects)** has `PlatformEffectPerformer`, which runs
  one plugin call per `PlatformEffect`, in order. `EffectPlugins` is the
  test border, and `PluginEffects` is the production implementation.
- **[`pages/`](../lib/pages)** has `RecipeListPage`, `BrewPage`,
  `EditorPage` and `SettingsPage`. Each page reads holders and sends
  signals; none of them computes a label. The editor's steps are a
  `SliverReorderableList`: a drop sends `MoveStep` (row id and final index,
  from `onReorderItem`) and the order changes only when Rust's draft comes
  back. The list re-creates a moved card, whose fields start from the text
  Rust holds for that row.
- **[`widgets/`](../lib/widgets)** has the display widgets. Recipes are
  drawn as printed labels: `LabelColors` maps the `RecipeLabel` to colours
  (dimmed in dark theme), `LabelPaperPainter` paints the paper, texture,
  stain and frame, `label_parts.dart` has the name badge, tagline and spec
  row, and `RoastMeter` draws the beans. `RecipeSummaryTile` is a list row
  on its own label; the brew page lays the whole page on one. `BrewDial` is
  the brew timer: bezel ticks, an inner arc that sweeps per step and
  restarts on `restartKey` (it stops and fades while a step is `paused`,
  and goes on from there), and an outer track with each step's arc.
  `StepListTile` is one step of the brew page's method.
- `src/bindings/` holds the generated, gitignored signal classes. Run
  `rinf gen` to create them.

---

## test/ — Flutter tests

[`test/`](../test)

Every test uses hand-built signal values, so no test needs Rust. Shared values live in [`fixtures.dart`](../test/fixtures.dart).

- [`holders/`](../test/holders) tests list folding, replace semantics, and
  notification on identical updates.
- [`effects/`](../test/effects) tests one plugin call per effect, order, and
  that a failure does not block later effects.
- [`pages/`](../test/pages) tests the list page's snackbars and the editor
  page: open, field errors, pop on `Closed`, notices, and the delete dialog.
- [`widgets/`](../test/widgets) tests the display widgets, including the
  ring's restart key.
