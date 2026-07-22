# Module reference

This maps every module to what it does today. See
[architecture.md](architecture.md) for the runtime concepts (`Hub`, `Leaf`,
buses, event-flow patterns) these modules build on.

- [Core — Recipe domain](#core--recipe-domain)
- [Core — Settings](#settings-core)
- [Core — Theme](#core--theme)
- [Core — Hardware/effect leaves](#core--hardwareeffect-leaves)
- [Feature: recipe_list](#feature-recipe_list)
- [Feature: recipe_details (the timer)](#feature-recipe_details-the-timer)
- [Feature: recipe_edit](#feature-recipe_edit)
- [Feature: settings](#feature-settings)

---

## Core — Recipe domain
`lib/core/recipe/`

### Entities

**`Recipe`** (`entities/recipe.dart`) — immutable value object.

| Member | Notes |
|--------|-------|
| `id, name, description` | `String` |
| `steps` | `List<RecipeStep>` |
| `static const empty` | all-blank recipe |
| `isEmpty` / `isNotEmpty` | empty when id+name+description+steps all empty |
| `totalDurationSeconds` | sum of step durations, **capped at 6000** |
| `totalDurationString` | `MM:SS` (`'99:99'` if ≥100 min) |
| `totalCoffeeGrams` | `steps.length * 10` — placeholder formula |
| `totalWaterGrams` | `steps.length * 50` — placeholder formula |
| `static totalDurationFormatted(int)` | the MM:SS formatter |

> Coffee/water grams are hardcoded per-step placeholders; the entity has no real
> coffee/water fields yet.

**`RecipeStep`** (`entities/step.dart`) — `id`, `name`, `int durationSeconds`,
plus `const RecipeStep.empty()`. No behavior.

### Events
`lib/core/recipe/events/` — all extend `RecipeEvent extends Event`.

**UI / domain events** (`events/ui.dart`):

| Event | Payload | Handled by actor? |
|-------|---------|-------------------|
| `RecipesLoad` | — | ✔ |
| `RecipesLoaded` | `List<Recipe>` | (state → UI) |
| `RecipeSelect` | `Recipe` | ✔ |
| `RecipeDeselect` | — | ✔ |
| `RecipeCreate` | `Recipe` | ✔ |
| `RecipeUpdate` | `Recipe` | ✔ |
| `RecipeDelete` | `{String id}` | ✔ |
| `RecipeTimerStart` | — | ✔ |
| `RecipeTimerStartFromStep` | `int stepIndex` | ✔ |
| `RecipeTimerReset` | — | ✔ |
| `RecipeTimerStarted` | `{int? index, RecipeStep step}` | (state → UI) |
| `RecipeTimerStepChanged` | `{int? index, RecipeStep step}` | (state → UI) |
| `RecipeTimerTick` | `{String minutes, String seconds}` (+`.empty()`) | (state → UI) |
| `RecipeTimerEnded` | — | (state → UI) |
| `RecipeModified` | `Recipe` | (state → UI) |
| `RecipesTest`, `RecipeTimerPause`, `RecipeTimerResume`, `RecipeDeleted` | — | **declared but not handled** |

**Storage events** (`events/storage.dart`) — request/response pairs:

| Request | Response |
|---------|----------|
| `RecipesLoadStorage` | `RecipesLoadedStorage(List<Recipe>)` |
| `RecipeCreateStorage(Recipe)` | `RecipeCreatedStorage(Recipe)` |
| `RecipeUpdateStorage(Recipe)` | `RecipeUpdatedStorage(Recipe)` |
| `RecipeDeleteStorage(String id)` | `RecipeDeletedStorage(String id)` |

### `RecipeActor`
`actors/recipe_actor.dart` — a `Leaf`, the brain of the recipe domain. Holds
in-memory state: `List<Recipe> _recipes` (cache), `Recipe? _currentRecipe`,
`RecipeStep? _currentStep`, and timer fields.

CRUD / selection handling:

- `RecipesLoad` → cache warm ⇒ `RecipesLoaded(_recipes)`; cache empty ⇒
  `RecipesLoadStorage`.
- `RecipesLoadedStorage` → append to cache, emit `RecipesLoaded`.
- `RecipeCreate` → `RecipeCreateStorage`; on `RecipeCreatedStorage` add to
  cache, emit `RecipesLoaded` (+`RecipeModified` if a recipe is currently open).
- `RecipeUpdate` → set `_currentRecipe`, `RecipeUpdateStorage`; on
  `RecipeUpdatedStorage` replace/add by id, emit `RecipesLoaded` (+`RecipeModified`).
- `RecipeDelete` → `RecipeDeleteStorage`; on `RecipeDeletedStorage` remove from
  cache, emit `RecipesLoaded`; if the open recipe was deleted, emit
  `RecipeModified(Recipe.empty)` (note: it emits `RecipeModified`, **not**
  `RecipeDeleted`).
- `RecipeSelect` → set `_currentRecipe`, emit `AudioLoadAsset(AudioAsset.timer)`
  (pre-load the sound).
- `RecipeDeselect` → clear recipe, reset timer, emit `WakelockDisable` +
  `AudioUnloadAssetLatest`.

**The timer engine** (this is the real one — the UI `TimerController` is dead
code):

- `_startTimer({initialStepIndex})`: reset state, guard null/empty recipe, set
  `_currentStep` and remaining seconds. Emit `WakelockEnable`,
  `RecipeTimerStarted(index, step)`, and the first `RecipeTimerTick`.
- A `Timer.periodic(1s)` decrements the step + total remaining seconds and emits
  `RecipeTimerTick(MM, SS)` every second (`_formatRemainingSeconds` → zero-padded,
  capped at `('99','60')`).
- **Step boundary** (`_stepRemainingSeconds <= 0`): advance index.
  - Past last step ⇒ `_resetTimer()` (cancel, emit `WakelockDisable` +
    `RecipeTimerEnded`) then emit `Vibrate` (final buzz).
  - Otherwise load next step; emit `Vibrate` + `AudioPlayTimerSound` +
    `RecipeTimerTick` + `RecipeTimerStepChanged(index, step)`.

So the actor never touches plugins directly — every effect is an emitted event
consumed by the hardware leaves.

### Storage — `RecipeStorageSqlite`
`storage/recipe_storage_sqlite/` — a `Leaf` wrapping a `SqliteDatabase`.

- `initialize()` → `MigrationRunner.run(db, allMigrations)`.
- Handles the four `*Storage` requests → runs SQL → emits the matching
  `*edStorage` response:
  - `_loadRecipes`: `SELECT * FROM recipe` + `SELECT * FROM step`, group steps by
    `recipe_id`, join into `Recipe`s.
  - `_createRecipe`: `writeTransaction` — `INSERT ... RETURNING` recipe, then each
    step; returns the reconstructed `Recipe`.
  - `_updateRecipe`: upsert recipe (`INSERT ... ON CONFLICT(id) DO UPDATE`), then
    `DELETE FROM step WHERE recipe_id=?` and re-insert all steps (full replace).
  - `_deleteRecipe`: delete steps then the recipe row.

**Schema & migrations** (`migration.dart`, `migration_runner.dart`,
`migrations.dart`):

- `Migration { int version; List<String> statements }`.
- `MigrationRunner` reads `PRAGMA user_version`, runs pending migrations
  (`version > current`) ascending, each in a `writeTransaction`, bumping
  `user_version`. ⚠ It iterates the **global `allMigrations`** and ignores the
  `migrations` argument it's passed.
- `allMigrations`:
  - **v1** — `recipe(id PK, name, description)` and
    `step(id PK, recipe_id, name, duration_seconds, FK→recipe ON DELETE CASCADE)`.
  - **v2** — seed data: 2 recipes ("Espresso", "Pour Over") + 5 steps.

---

## Settings core
`lib/core/settings/`

**`Settings`** entity (`entities/settings.dart`): `String theme`
(`'light'`/`'dark'`/`'system'`), `int accentColor` (ARGB, default
`0xFF2196F3`). Has `fromMap`, `toMap`, `copyWith`, `static const empty`. ⚠
`fromMap`/`toMap` are **asymmetric** — `fromMap` reads `accent_color` as an int,
`toMap` writes it as a `#AARRGGBB` string.

**Events** (all extend `SettingsEvent extends Event`):

| Domain (`settings_events.dart`) | Storage (`settings_storage_events.dart`) |
|---|---|
| `SettingsLoad` / `SettingsLoaded(Settings)` | `SettingsLoadStorage` / `SettingsLoadedStorage(Settings)` |
| `SettingsUpdate(Settings)` / `SettingsUpdated(Settings)` | `SettingsUpdateStorage(Settings)` / `SettingsUpdatedStorage(Settings)` |
| `SettingsThemeChanged(String)` | |
| `SettingsAccentColorChanged(int)` | |

**`SettingsActor`** (`actors/settings_actor.dart`) — constructed with an `emit`
callback, caches `Settings? _currentSettings`. Maps `SettingsLoad` →
load/serve-cache, `SettingsLoadedStorage`/`SettingsUpdatedStorage` → cache +
`SettingsLoaded`/`SettingsUpdated`, `SettingsUpdate` → cache +
`SettingsUpdateStorage`, and `SettingsThemeChanged`/`SettingsAccentColorChanged`
→ `copyWith` + `SettingsUpdate`.

> ⚠ **This actor is never subscribed to the settings bus in `main.dart`**, so at
> runtime it does nothing. As a consequence the `SettingsThemeChanged` /
> `SettingsAccentColorChanged` events fired by the settings page have **no
> handler** — the theme/accent change→persist→re-broadcast round trip through
> the actor does not run. (`ThemeManager` still updates the live UI on
> `SettingsLoaded`/`SettingsUpdated`, so effects only appear if some other path
> emits those.)

**`SettingsStorageSqlite`** (`storage/settings_storage_sqlite.dart`) — given a
`SqliteDatabase` + `emit`. `initialize()` creates the table and seeds defaults.
Handles `SettingsLoadStorage` → `SettingsLoadedStorage`, and
`SettingsUpdateStorage` → `SettingsUpdatedStorage` (`INSERT OR REPLACE`).

**Table** (`storage/settings_table.dart`, `SettingsTable`): a key-value store —
`settings(key TEXT PRIMARY KEY, value TEXT NOT NULL)`. Rows: `theme`,
`accent_color` (stored as `#AARRGGBB`). Defaults `light` / `#FF2196F3`.

**`SettingsEventBus`** (`buses/settings_event_bus.dart`): standalone singleton
broadcast stream (`init` / `stream` / `fire` / `dispose`) — see
[architecture.md → The buses](architecture.md#the-buses).

---

## Core — Theme
`lib/core/theme/theme_manager.dart`

**`ThemeManager`** — a singleton `ChangeNotifier` (`ThemeManager.instance`).
State: `String _currentTheme` (default `'system'`), `Color _accentColor`.

- `handleEvent` reacts to **`SettingsLoaded`** and **`SettingsUpdated`** — copies
  `settings.theme` + `Color(settings.accentColor)` and `notifyListeners()`.
- `themeMode` → `ThemeMode.dark/light/system`.
- `light` → `ThemeData.light().copyWith(colorScheme: ColorScheme.fromSeed(seedColor: accent))`.
- `dark` → same seeded from accent with `Brightness.dark`.

Consumed at the app root via `ListenableBuilder` in `MyApp`, feeding
`MaterialApp.themeMode/theme/darkTheme`.

---

## Core — Hardware/effect leaves

| Leaf | File | Kind | Events | Behavior |
|------|------|------|--------|----------|
| **AudioPlayerLeaf** | `audioplayers/audioplayer_leaf.dart` | true `Leaf` | `AudioInitialize`, `AudioDispose`, `AudioLoadAsset(String)`, `AudioUnloadAssetLatest`, `AudioPlayTimerSound` | Wraps `audioplayers`; lazily creates an `AudioPlayer` (`ReleaseMode.stop`), loads/plays `assets/sounds/step-change.mp3` (`AudioAsset.timer`). Registered on the `recipe` bus; kicked with `emitToLeaf('audioPlayer', AudioInitialize())`. |
| **VibrateFlutterLeaf** | `vibrate/vibrate_leaf.dart` | static `handle` | `Vibrate` | If `Vibration.hasVibrator()`, `Vibration.vibrate(duration: 100)`. Subscribed as `recipeToVibrate`. |
| **WakelockLeaf** | `wakelock/wakelock_leaf.dart` | static `handle` | `WakelockEnable`, `WakelockDisable` | `WakelockPlus.enable()/disable()`. Subscribed as `recipeToWakeLock`. |

Effect events live in `lib/core/events/` (`audio_player_events.dart`,
`vibrate_events.dart`, `wakelock_events.dart`). `AudioAsset`
(`core/audio_asset.dart`) just holds `timer = 'sounds/step-change.mp3'`.

---

## Feature: recipe_list
`lib/recipe_list/`

**`RecipeListPage`** (`pages/recipes_list_page.dart`) — `StatefulWidget`.

- **UI:** AppBar (title + settings icon → `SettingsPage`), FAB "New recipe" →
  `RecipeEditPage` (create), `ListView.builder` of `RecipeSummaryTile`.
- **Interactions:** tap → open `RecipeDetailPage`; long-press → bottom sheet
  (Edit / Delete); delete → confirm dialog → emit `RecipeDelete(id:)`.
- **Bus:** subscribes to `'recipe'` in `initState`, emits `RecipesLoad`; reacts
  only to `RecipesLoaded` → `ValueNotifier<List<Recipe>> _recipes`.
- **Nav nuance:** on tap it pushes the detail page, then emits `RecipeSelect`
  from an `addPostFrameCallback` so the just-mounted detail page's subscription
  is live to receive it.

**`RecipeSummaryTile`** (`widgets/recipe_summary_tile.dart`) — pure
`StatelessWidget`; a card with name, description, and coffee/water/time row. All
values are plain primitives passed in — no core coupling.

---

## Feature: recipe_details (the timer)
`lib/recipe_details/`

**`RecipeDetailPage`** (`pages/recipe_details_page.dart`) — `StatefulWidget`.

- **UI:** AppBar (reactive recipe name + edit icon → `RecipeEditPage(initial:)`);
  body = description, a big **`CircularTimer`**, and a `ListView` of
  `StepListTile`s. Tapping the timer toggles start/reset. When running, the
  center shows the current step name + MM:SS via fixed-width digit boxes (no
  jitter); when idle it shows coffee/water/total info. Long-pressing a step →
  start from that step.
- **Bus:** subscribes to `'recipe'`. Maps incoming events to `ValueNotifier`s:
  `RecipeModified`/`RecipeSelect` → recipe; `RecipeTimerTick` → minutes/seconds;
  `RecipeTimerStarted`/`RecipeTimerStepChanged` → step + index + duration;
  `RecipeTimerStart`/`…FromStep` → `started=true`; `RecipeTimerEnded` → reset;
  `RecipeTimerReset`/`RecipeTimerPause` → `started=false`.
- **Emits:** `RecipeTimerStart`, `RecipeTimerReset`, `RecipeTimerStartFromStep(i)`,
  and `RecipeDeselect` on pop (via `PopScope`) — which makes the actor disable
  wakelock and unload audio.
- **State:** several `ValueNotifier`s; `Listenable.merge([started, currentStepIndex])`
  drives the step list so tiles re-render on run-state / active-step change.

**Timer widgets:**

- **`CircularTimer`** (`widgets/circular_timer.dart`) — `Stack` of a
  `DurationalCircularProgressIndicator` (fed a `Duration?`) with an overlay child.
- **`DurationalCircularProgressIndicator`**
  (`widgets/durational_circular_progress_indicator.dart`) — drives a Material
  `CircularProgressIndicator` from an `AnimationController` whose `duration` = the
  step duration, `repeat()`ing a 0→1 sweep and resetting on step change. ⚠ It is
  a **repeating per-step sweep**, not a precise remaining-time fill — the exact
  countdown is the MM:SS text from `RecipeTimerTick`.
- **`StepListTile`** (`widgets/step_list_tile.dart`) — leading icon via
  `AnimatedSwitcher`: done steps `check_circle`, current `play_arrow`, future =
  number; title = step name, trailing = `${seconds}s`, `onLongPress` =
  start-from-step.

> ⚠ **`notifiers/timer_controller.dart` (`TimerController extends ChangeNotifier`)
> is dead code** — a self-contained countdown that nothing references. The live
> timer is `RecipeActor` (see [above](#recipeactor)).

---

## Feature: recipe_edit
`lib/recipe_edit/`

**`RecipeEditPage`** (`ui/pages/recipe_edit_page.dart`) — `StatefulWidget`,
serves both create and edit via `route({initial})`.

- **UI:** a `Form` in a `ListView` — name (required), multiline description, live
  total-duration text, and a dynamic list of step cards (name, delete button,
  seconds field with a live `MM:SS` helper). Quick-add buttons ("Add coffee",
  "Add water", "Wait", "Add Step") append steps. AppBar has delete (edit mode
  only) + save; a bottom "Save Recipe" button too.
- **Bus:** write-only (no subscription). `_save()` validates, rebuilds
  `RecipeStep`s (reuse original ids, else `Uuid().v7()`; blank name → "Step N"),
  builds the `Recipe`, then emits `RecipeCreate` (if `initial.isEmpty`) or
  `RecipeUpdate`, and `Navigator.pop(recipe)`. Delete → confirm → emit
  `RecipeDelete(id:)`, clear the stack, push a fresh `RecipeListPage`.
- **State:** local `TextEditingController`s (name/desc + parallel lists for step
  names & durations); `setState` for add/remove/helper refresh. All disposed.

---

## Feature: settings
`lib/settings/`

**`SettingsPage`** (`ui/pages/settings_page.dart`) — `StatefulWidget`.

- **UI:** theme `DropdownButtonFormField` (Light/Dark/System), a "Refresh
  Settings" button, and an accent-color `ListTile` with an animated swatch →
  `_showAccentColorPicker` (dialog with 10 predefined `AccentColorOption`s).
- **Bus:** uses **`SettingsEventBus.instance`** (not the `Hub`). Emits
  `SettingsLoad` on init/refresh; reacts to `SettingsLoaded`/`SettingsUpdated` →
  `_selectedTheme`, `_accentColor`. Theme change → `SettingsThemeChanged(theme)`;
  color change → `SettingsAccentColorChanged(color.toARGB32())`.
- **State:** `ValueNotifier<String> _selectedTheme`, `ValueNotifier<Color>
  _accentColor`; subscription cancelled in `dispose`. `AccentColorOption` and
  `predefinedAccentColors` live inline in this file.

> `ui/widgets/widgets.dart` is effectively empty — a barrel placeholder; all
> settings UI lives in `settings_page.dart`. And note the
> [`SettingsActor` wiring gap](#settings-core): the theme/accent events this page
> fires currently have no live handler.

---

## Cross-module navigation

```
RecipeListPage ──tap──────────────► RecipeDetailPage  (+ post-frame RecipeSelect)
      │                                     │
      ├─FAB────────► RecipeEditPage(create) │
      ├─long-press─► RecipeEditPage(edit)   ├─edit icon─► RecipeEditPage(edit)
      └─settings──► SettingsPage            └─on pop────► emit RecipeDeselect

RecipeEditPage ──pop(recipe)──► back to caller
              └─delete──► clear stack ─► fresh RecipeListPage
```
