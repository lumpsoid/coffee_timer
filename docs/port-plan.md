# Port plan: Rust core + rinf + Flutter GUI

Port the current Dart app to a Rust core driven through rinf, with Flutter as
a GUI that decides nothing. Behavior is the spec (see `CLAUDE.md`); structure
is new. Legacy bugs are fixed, not ported. There are no installs to migrate.

## Core model: feature controllers over state objects

Each feature has one **owner controller** holding one **state object**. An
action takes plain values, mutates the state, and returns only what state
cannot hold. A test runs an action and asserts the state.

Dependencies enter two ways:

- **Infrastructure** (store, id generator) is constructor-injected as
  `Box<dyn Trait>`. The traits live in core; `coffee_timer_store` and the hub
  implement them; tests use fakes (`Vec`-backed store, sequential ids).
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
  catalog/          RecipeCatalog       state: Vec<Recipe>         (shared)
  settings/         SettingsController  state: Settings
  brew/             BrewController      state: BrewState
  editor/           EditorController    state: Draft
```

| Controller | Actions (shape, not final) | Returns |
|---|---|---|
| `RecipeCatalog` (store) | `load()`, `upsert(recipe)`, `remove(id)` | `Result<CatalogChange, StoreError>` |
| `SettingsController` (store) | `load()`, `set_theme`, `set_accent` | `Result<(), StoreError>` |
| `BrewController` | `open(id, &catalog)`, `start(from, &catalog)`, `tick(&catalog)`, `reset`, `leave`, `on_catalog_change(&change)` | `Vec<Effect>` |
| `EditorController` (ids) | `new_draft`, `edit(id, &catalog)`, `set_name`, `set_step_duration(row, raw)`, `add_step(preset)`, `remove_step(row)`, `save(&mut catalog)`, `delete(&mut catalog)` | `Result<CatalogChange, SaveError>` |

```rust
let mut catalog = RecipeCatalog::new(Box::new(FakeRecipeStore::default()));
let mut editor = EditorController::new(Box::new(SequentialIds::default()));
editor.new_draft();
editor.set_name("V60");
let change = editor.save(&mut catalog)?;
assert_eq!(catalog.state().len(), 1);
assert_eq!(change, CatalogChange::Upserted(RecipeId::try_parse("r1")?));
```

`CatalogChange` is returned after the catalog has persisted it. Each shell
passes it on with no conditions:

1. send `RecipeListUpdate::from(&change)` — GUI diff
2. `brew.on_catalog_change(&change)` — brew decides whether its recipe moved

Effects are one-shot instructions: `Vibrate { ms }`, `PlayStepSound`,
`KeepScreenOn(bool)`. Controllers never read the clock; time arrives as
`tick`.

## Signals

- The recipe list is a keyed, growing collection: `Snapshot` / `Upserted` /
  `Removed`, mapped one-to-one from `CatalogChange`.
- Brew, editor, settings: after an action the hub sends the feature's **whole
  view state** (a few hundred bytes). Dart replaces, it does not fold.
- Labels (`20g`, `1:35`, `MM:SS`, `Ns`), step status (`Done` / `Current` /
  `Pending`), and field errors are computed in Rust and sent.

## Timer rules (`BrewController`)

Kept: start from step 0 or a chosen step; one tick per second; at a step
boundary vibrate + sound + next step; after the last step screen-on off,
ended, vibrate, no sound; reset turns screen-on off; leaving resets silently.

Fixed / changed:

- Remaining never goes negative (`saturating_sub`). A 0-second step shows
  `00:00` for one tick, then advances.
- Out-of-range start index → `Err`.
- First tick exactly 1 s after start: the hub creates the interval per brew
  (`MissedTickBehavior::Delay`) and drops it on end.
- The brew view follows catalog changes. An edit to the open recipe during a
  running brew resets it; deleting it shows "Empty recipe".
- Dropped: unused total-remaining counter, unreachable `99:60` cap,
  per-page audio preload/unload (Dart loads the sound once at startup).

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
| 7 | `hub`: `Boot { db_dir }`, `<Feature>Ready` → state, uuid v7 `IdGen`, one task owning all controllers, per-brew interval | `rinf gen`, clippy |
| 8 | Flutter: one holder per feature; keep display widgets (`CircularTimer`, `StepListTile`, `RecipeSummaryTile`, `FixedDigit`); one `PlatformEffect` performer; delete legacy Dart and unused packages | `flutter analyze` / `flutter test` + device run |
| 9 | Rewrite `docs/README.md`, `architecture.md`, `modules.md` | — |

`lib/domain/` remains the behavior reference until phase 8.
