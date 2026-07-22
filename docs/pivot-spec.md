# Architecture Pivot — Step-by-Step Spec

Target: replace the custom event-bus / `Leaf` / `Hub` runtime with **simple,
single-purpose classes that compose into complex behavior**. The whole thing is
optimized for one property: *any one class is small enough to read in a minute,
test with plain values, and trust.*

See [architecture.md](architecture.md) and [modules.md](modules.md) for how it
works **today**. This doc is the target and the migration path.

---

## 0. Principles (the whole point)

1. **Small classes, composed.** Complex behavior emerges from wiring small
   pieces, not from big smart objects. Prefer five 20-line classes over one
   100-line one. A class does one thing; its name says what.
2. **Invariance through types.** Make invalid states *unrepresentable*. Value
   objects with smart constructors; sealed states instead of
   boolean+nullable clusters. If you reach for `?? ''`, `!`, or a defensive
   `if` at a boundary, fix the type instead.
3. **Decisions in the domain, plumbing in shells.** Every `if`/comparison/
   arithmetic on data is either a *business decision* (put it in a named
   domain class) or *plumbing* (sequencing, fetching, dispatching, firing
   effects — put it in controllers/repositories). Never a business `if` inside
   a controller, repository, or timer callback.
4. **Testable without the framework.** A rule is right if you can test it with
   plain values — no `Timer`, no SQLite, no widgets, no mocks. If testing it
   needs a mock, it's in the wrong layer.
5. **One home per rule.** A threshold/formula (the `6000`s cap, grams formulas,
   MM:SS clamp) has exactly one named location. A one-line business change
   touches one file.

> We are **not** adopting a functional-core effect loop or a `Result` monad.
> Errors are handled plainly (a controller catches and exposes an error field).
> The value we keep is *rule placement*, not the ceremony.

---

## 1. Target layering

Dependencies point **inward only** (UI → application → domain; infrastructure
implements domain-owned ports).

```
lib/
├── domain/            Pure. No Flutter, no async, no I/O. Values, entities, policies, states.
├── application/       Use cases. Orchestrate: load via ports → ask domain → save. No decisions.
├── infrastructure/    Adapters. SQLite repos, hardware wrappers. DTO↔domain mapping lives here.
├── ui/                Controllers (ChangeNotifier), pages, widgets, RouteArgs. Cosmetics only.
└── app/               Composition root: Cell<T>, Cells, AppRoot, modules (controller factories).
```

Feature folders slice *within* these where it helps (e.g.
`domain/recipe/…`, `ui/recipe_list/…`). One barrel per folder as today.

### Layer rules
- **domain** never imports `flutter`, `sqlite`, `dart:async` timers, or DTOs.
- **application** depends on domain + **ports** (abstract interfaces it declares),
  never on concrete infrastructure.
- **infrastructure** depends on domain + application ports; it is the *only*
  place raw DB rows / plugin calls appear; it runs smart constructors when
  mapping DTO → domain.
- **ui** depends on application + domain; holds no business `if`.
- **app** is the only place that constructs concrete types and wires them.

---

## 2. Business-rule placement — the ladder (this app)

Place each rule at the **smallest level with enough data to decide**.

**Level 1 — one value → value object w/ smart constructor.** Private field, one
entry point, valid-by-construction. For this app:

```dart
// domain/recipe/step_duration.dart
class StepDuration {
  const StepDuration._(this.seconds);
  final int seconds;                          // always 0..maxStepSeconds
  static const maxStepSeconds = 5999;         // named, one home
  static StepDuration? tryParse(int raw) =>
      (raw < 0 || raw > maxStepSeconds) ? null : StepDuration._(raw);
  String get mmss => '${(seconds ~/ 60).toString().padLeft(2,'0')}'
                     ':${(seconds % 60).toString().padLeft(2,'0')}';
}
```

Similar value objects: `RecipeName` (non-empty, trimmed), `RecipeId` /
`StepId` (non-empty; created via `uuid` at the boundary), `AccentColor`
(valid ARGB int), `ThemeSelection` (a closed enum: `light | dark | system`).
These replace today's raw `String`/`int` fields and the `?? ''`/`!` at the DB
boundary.

**Level 2 — one entity's own fields → method, no public setters.** `Recipe`
and `RecipeStep` stay immutable; mutation is named operations returning new
instances:

```dart
// domain/recipe/recipe.dart
class Recipe {
  const Recipe({required this.id, required this.name,
                required this.description, required this.steps});
  final RecipeId id; final RecipeName name;
  final String description; final List<RecipeStep> steps;

  Recipe renamed(RecipeName n)        => copyWith(name: n);
  Recipe withStepAdded(RecipeStep s)  => copyWith(steps: [...steps, s]);
  Recipe withStepRemovedAt(int i)     => copyWith(steps: [...steps]..removeAt(i));
  // no public setters, ever
}
```

**Level 3 — spans entities, still pure → named policy.** Extract every
threshold/formula that today sits inline in `Recipe`:

```dart
// domain/recipe/brew_metrics.dart  — the grams/duration rules, named, one home
class BrewMetrics {
  const BrewMetrics({this.coffeePerStepG = 10, this.waterPerStepG = 50,
                     this.totalCapSeconds = 6000});
  int coffeeGrams(Recipe r) => r.steps.length * coffeePerStepG;
  int waterGrams(Recipe r)  => r.steps.length * waterPerStepG;
  int totalSeconds(Recipe r) { var t = 0;
    for (final s in r.steps) { t += s.duration.seconds; if (t >= totalCapSeconds) return totalCapSeconds; }
    return t; }
}
```

**Level 4 — needs fetched data → use case.** Orchestrates only: load via a
port → ask domain → save. Zero decisions in its body (see §4).

### The timer, done as a pure state machine (the big win)
Today the countdown, step-advance, and effect-firing are tangled in
`RecipeActor` with a `Timer.periodic`. Split it:

```dart
// domain/timer/timer_state.dart — sealed; invalid states impossible
sealed class TimerState { const TimerState(); }
class TimerIdle    extends TimerState { const TimerIdle(); }
class TimerRunning extends TimerState {
  const TimerRunning({required this.stepIndex, required this.stepRemaining,
                      required this.totalRemaining});
  final int stepIndex; final int stepRemaining; final int totalRemaining;
}
class TimerFinished extends TimerState { const TimerFinished(); }
```

```dart
// domain/timer/brew_timeline.dart — PURE decision: what happens on each tick
class BrewTimeline {
  const BrewTimeline(this.recipe);
  final Recipe recipe;

  TimerState start(int fromStep) { /* returns TimerRunning or TimerFinished */ }

  /// One second elapsed. Returns (next state, transition) with NO side effects.
  (TimerState, TimerTransition) tick(TimerRunning s) { /* pure */ }
}

// domain/timer/timer_transition.dart — what *kind* of boundary we crossed
enum TimerTransition { none, stepAdvanced, finished }
```

`BrewTimeline` is fully testable with plain values: feed a `Recipe` + state,
assert the next state + transition. The rule "advance at 0, finish after last
step, cap at `totalCapSeconds`" lives here and nowhere else.

The **controller** (§3) owns the `Timer.periodic`, calls `tick`, stores the new
state, and *maps transitions to effects* (`stepAdvanced → vibrate + sound`,
`finished → vibrate + wakelock off`). Firing effects is plumbing; deciding a
boundary was crossed is the domain.

---

## 3. Controllers = `ChangeNotifier` (the imperative shell)

One controller per feature page. It holds state, exposes read-only getters,
receives UI intents as methods, and calls `notifyListeners()`. It contains
**no business `if`** — it composes named domain decisions and runs effects.

```dart
// ui/recipe_details/recipe_details_controller.dart
class RecipeDetailsController extends ChangeNotifier {
  RecipeDetailsController({
    required RecipeReadRepository recipes,     // narrow port
    required AudioModule audio,                // effect modules, injected
    required VibrationModule vibration,
    required WakelockModule wakelock,
    required RecipeId recipeId,
  });

  // state
  Recipe? get recipe => _recipe;
  TimerState get timer => _timer;             // sealed → UI switches on it
  String? get error => _error;

  // intents
  Future<void> load() async { … }             // plumbing: fetch → set → notify
  void start()              { … }             // start timeline, WakelockModule.enable()
  void startFrom(int i)     { … }
  void reset()              { … }
  @override void dispose()  { _ticker?.cancel(); wakelock.disable(); audio.unload(); super.dispose(); }

  void _onTick() {
    final r = _timer; if (r is! TimerRunning) return;
    final (next, transition) = _timeline.tick(r);       // DOMAIN decides
    _timer = next;
    switch (transition) {                                // controller only dispatches
      case TimerTransition.stepAdvanced: vibration.buzz(); audio.playStepChange();
      case TimerTransition.finished:     vibration.buzz(); wakelock.disable();
      case TimerTransition.none: break;
    }
    notifyListeners();
  }
}
```

Rules for controllers:
- Created when the feature page **mounts**, disposed when it **unmounts** (§5).
- Depend on **narrow, named ports** (§6), not a fat repository.
- No `Timer`/effect logic decides business outcomes — it dispatches on domain
  verdicts.
- `error` is a plain nullable field set in a `try/catch`; no `Result` type.

Long-lived exception: **`ThemeController`** drives `MaterialApp` for the whole
app lifetime, so it lives in a cell created by the construction root, not per
page.

---

## 4. Use cases (thin, optional for trivial CRUD)

Use cases orchestrate fetch → domain → persist and hold zero decisions.

```dart
// application/recipe/save_recipe.dart
class SaveRecipe {
  const SaveRecipe(this._writes);
  final RecipeWriteRepository _writes;
  Future<Recipe> call(Recipe recipe) => _writes.upsert(recipe);   // plumbing only
}
```

Pragmatic stance: for pure passthrough CRUD (load all, delete by id) the
controller may call the repository port directly — a use case that only forwards
adds no value. Introduce a use case the moment orchestration or a domain call
appears between fetch and save.

---

## 5. Static cells + construction root

### `Cell<T>` — a tiny typed holder
```dart
// app/cell.dart
class Cell<T> {
  T? _v;
  bool get isSet => _v != null;
  T get value => _v ?? (throw StateError('Cell<$T> read before it was set'));
  set value(T v) => _v = v;
}
```

### `Cells` — the static roots (what the UI reads)
```dart
// app/cells.dart
abstract final class Cells {
  static final theme          = Cell<ThemeController>();   // long-lived
  static final recipeModule   = Cell<RecipeModule>();      // controller factory
  static final settingsModule = Cell<SettingsModule>();
}
```

### Modules — controller factories + shared singletons
A module owns the shared infra for a feature area and builds fresh controllers.
The construction root registers **modules** (and long-lived controllers);
feature pages ask a module to build a **fresh feature controller** on mount.

```dart
// app/recipe_module.dart
class RecipeModule {
  RecipeModule({required this.recipes, required this.audio,
                required this.vibration, required this.wakelock});
  final SqliteRecipeRepository recipes;     // implements the narrow read/write ports
  final AudioModule audio; final VibrationModule vibration; final WakelockModule wakelock;

  RecipeListController    newListController()             => RecipeListController(recipes: recipes);
  RecipeDetailsController newDetailsController(RecipeId id)=> RecipeDetailsController(
        recipes: recipes, audio: audio, vibration: vibration, wakelock: wakelock, recipeId: id);
  RecipeEditController    newEditController(RecipeEditArgs a) => RecipeEditController(recipes: recipes, args: a);
}
```

### `AppRoot` — the single construction root
A `StatefulWidget` that hosts `MaterialApp`. `initState` opens the DB, builds
modules, and populates the cells **once**. This replaces `main.dart`'s bus
wiring.

```dart
// app/app_root.dart
class AppRoot extends StatefulWidget { … }
class _AppRootState extends State<AppRoot> {
  @override void initState() {
    super.initState();
    final db = SqliteDatabase(path: widget.dbPath);
    final audio = AudioModule()..initialize();
    // wire modules
    Cells.recipeModule.value = RecipeModule(
      recipes: SqliteRecipeRepository(db)..ensureMigrated(),
      audio: audio, vibration: VibrationModule(), wakelock: WakelockModule());
    Cells.settingsModule.value = SettingsModule(SqliteSettingsRepository(db));
    // long-lived controller
    Cells.theme.value = ThemeController(Cells.settingsModule.value.repository)..load();
  }
  @override Widget build(BuildContext c) => ListenableBuilder(
    listenable: Cells.theme.value,
    builder: (c, _) => MaterialApp(
      themeMode: Cells.theme.value.mode,
      theme: Cells.theme.value.light, darkTheme: Cells.theme.value.dark,
      home: const RecipeListPage()));
}

// main.dart shrinks to:
void main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final dir = await getApplicationDocumentsDirectory();
  runApp(AppRoot(dbPath: p.join(dir.path, 'recipes.db')));
}
```

### Feature page lifecycle (create on mount, dispose on unmount)
```dart
// ui/recipe_details/recipe_details_page.dart
class _RecipeDetailsPageState extends State<RecipeDetailsPage> {
  late final RecipeDetailsController _c;
  @override void initState() {
    super.initState();
    _c = Cells.recipeModule.value.newDetailsController(widget.args.recipeId);
    _c.load();
  }
  @override void dispose() { _c.dispose(); super.dispose(); }
  @override Widget build(BuildContext ctx) =>
      ListenableBuilder(listenable: _c, builder: (ctx, _) => …);
}
```

This is the discipline that replaces today's `RecipeSelect`/`RecipeDeselect`
post-frame dance: the controller *is* the selection, scoped to the page's
lifetime; `dispose()` does the wakelock-off / audio-unload cleanup.

---

## 6. Navigation — `RouteArgs`, not events merged into the system

One typed args class per navigable page; carried explicitly, not smuggled
through a global event.

```dart
// ui/navigation/route_args.dart
sealed class RouteArgs { const RouteArgs(); }

class RecipeDetailsArgs extends RouteArgs {
  const RecipeDetailsArgs(this.recipeId);
  final RecipeId recipeId;
}
class RecipeEditArgs extends RouteArgs {
  const RecipeEditArgs.create() : initial = null;
  const RecipeEditArgs.edit(this.initial);
  final Recipe? initial;
  bool get isCreate => initial == null;
}
```

Pages accept their args as a constructor field:
```dart
Navigator.of(context).push(MaterialPageRoute(
  builder: (_) => RecipeDetailsPage(args: RecipeDetailsArgs(recipe.id))));
```

This deletes the `.route({initial})` helpers and the post-frame `RecipeSelect`
emit. Selection = navigating with args; deselection = the page (and its
controller) disposing.

### Narrow, named ports
Repositories are split by capability and named for what they do — never a
catch-all `RecipeRepository`.

```dart
// application/recipe/ports.dart
abstract interface class RecipeReadRepository  { Future<List<Recipe>> all(); Future<Recipe?> byId(RecipeId id); }
abstract interface class RecipeWriteRepository { Future<Recipe> upsert(Recipe r); Future<void> delete(RecipeId id); }
```
`SqliteRecipeRepository` implements both; a future `InMemoryRecipeRepository`
(for tests) implements only what a test needs.

---

## 7. Target inventory for this app

| Area | Domain | Application (ports/use cases) | Infrastructure | UI |
|------|--------|-------------------------------|----------------|----|
| Recipe | `Recipe`, `RecipeStep`, `RecipeId`, `StepId`, `RecipeName`, `StepDuration`, `BrewMetrics` | `RecipeReadRepository`, `RecipeWriteRepository`; `SaveRecipe`, `DeleteRecipe`, `LoadRecipes` | `SqliteRecipeRepository` (+ existing migrations, DTO mappers) | `RecipeListController`, `RecipeEditController` + pages/tiles |
| Timer | `TimerState` (sealed), `BrewTimeline`, `TimerTransition` | — (drives via recipe read) | — | `RecipeDetailsController` + circular-timer widgets |
| Settings | `ThemeSelection`, `AccentColor`, `Settings` | `SettingsReadRepository`, `SettingsWriteRepository` | `SqliteSettingsRepository` (+ existing kv table) | `SettingsController`, `ThemeController` + page |
| Effects | — | — | `AudioModule`, `VibrationModule`, `WakelockModule` (plugin wrappers) | injected into controllers |

Everything under `lib/core/di`, `lib/core/modules/base_module.dart`,
`lib/core/events`, `*/buses`, `*/actors`, and the dead `TimerController`
disappears (§9).

---

## 8. Migration plan — incremental strangler

Ground rule: **`flutter analyze` is clean and the app runs after every phase.**
New code lands alongside the old bus; each phase strangles one slice. Commit per
phase.

### Phase 0 — Scaffold (no behavior change)
- Add `app/cell.dart`, `app/cells.dart`, `ui/navigation/route_args.dart`.
- Add `AppRoot` that, for now, just hosts the existing `MaterialApp`/pages and
  keeps the current bus wiring inside `initState` (move `main.dart`'s body here
  verbatim). `main.dart` becomes the thin bootstrap.
- **DoD:** app behaves identically; cells exist but unused. Commit.

### Phase 1 — Settings (smallest feature, proves the pattern)
- **domain:** `ThemeSelection`, `AccentColor`, `Settings` (value objects; smart
  constructors run at the DB boundary).
- **ports + infra:** `SettingsReadRepository` / `SettingsWriteRepository`;
  `SqliteSettingsRepository` reusing the existing `settings` kv table + defaults.
- **ui:** `SettingsController` (ChangeNotifier) and `ThemeController` (long-lived,
  in `Cells.theme`); new `SettingsPage` reading the controller; theme/accent
  changes call controller methods → repo → `notifyListeners()`.
- **strangle:** delete `SettingsEventBus`, the never-subscribed `SettingsActor`,
  `settings/events/*`, old `SettingsStorageSqlite` wiring in the root, old
  `ThemeManager` (replaced by `ThemeController`). `AppRoot` now builds the
  settings module + theme controller from cells.
- **verify:** change theme + accent, kill/relaunch → persisted; app themes live.
  *(This phase also fixes the current dead-`SettingsActor` bug for free.)*
- Commit.

### Phase 2 — Recipe list
- **domain/infra:** `Recipe`/`RecipeStep`/value objects + `BrewMetrics`;
  `RecipeReadRepository` + `SqliteRecipeRepository.all()/byId()`; DTO↔domain
  mappers (reuse existing SQL + migrations).
- **ui:** `RecipeListController` (load, delete); new `RecipeListPage` +
  `RecipeSummaryTile` reading `BrewMetrics` for grams/duration; navigation via
  `RouteArgs` (still pushing the *old* detail/edit pages through a thin adapter
  that unpacks args → old `RecipeSelect`/`route` for now).
- **strangle:** list page no longer emits `RecipesLoad`/`RecipeDelete` on the bus.
- **verify:** list loads, delete works, still opens old detail/edit. Commit.

### Phase 3 — Recipe edit
- **domain:** entity methods (`renamed`, `withStepAdded/Removed`), `RecipeName`,
  `StepDuration` validation replacing the form's ad-hoc checks.
- **ports/use cases:** `RecipeWriteRepository`; `SaveRecipe`, `DeleteRecipe`.
- **ui:** `RecipeEditController` + new `RecipeEditPage`, `RecipeEditArgs`
  (create/edit). Remove `.route({initial})`.
- **strangle:** edit page stops emitting `RecipeCreate/Update/Delete` on the bus;
  list navigates to the *new* edit page.
- **verify:** create, edit, delete round-trip persists. Commit.

### Phase 4 — Recipe details + timer (the core)
- **domain:** `TimerState` sealed, `BrewTimeline`, `TimerTransition` — with unit
  tests (plain values, no `Timer`).
- **infra:** `AudioModule`, `VibrationModule`, `WakelockModule` wrapping the
  plugins (move logic out of the leaves).
- **ui:** `RecipeDetailsController` owning the `Timer.periodic`, calling
  `BrewTimeline.tick`, dispatching effects on transitions; new
  `RecipeDetailsPage` + `RecipeDetailsArgs`; circular-timer widgets rebound to
  the controller's `TimerState`.
- **strangle:** delete the timer + selection code in `RecipeActor`; list navigates
  to the new details page with args (drop the post-frame `RecipeSelect`).
- **verify:** run a recipe end-to-end — per-second countdown, step boundary
  buzz + sound, finish buzz, wakelock on while running / off on leave, screen
  correct. Use the `/verify` skill to drive it. Commit.

### Phase 5 — Teardown
- Delete: `core/di/`, `core/di.dart`, `core/modules/base_module.dart` +
  `base_event.dart`, all `core/**/buses`, `core/**/actors`, `core/**/events`
  that are now unused, `RecipeEventBus`, `recipe_bus_extension`, the dead
  `recipe_details/notifiers/timer_controller.dart`, old `ThemeManager`.
- Collapse `core/` barrels; update `docs/` (rewrite architecture.md/modules.md to
  the new design; keep this file as the migration record).
- **verify:** `flutter analyze` clean, full manual pass of all four features.
  Commit.

---

## 9. What gets deleted (net)

| Removed | Replaced by |
|---------|-------------|
| `Hub` / `Leaf` / `kernel` (`core/di`) | `Cell<T>` + `Cells` + modules |
| `base_module.dart` (unused FBP runtime) | — (deleted) |
| `RecipeEventBus`, `SettingsEventBus`, all `*/buses` | direct controller↔repo calls |
| `RecipeActor`, `SettingsActor` (`*/actors`) | controllers + `BrewTimeline` + use cases |
| `core/**/events/*`, effect events | plain method calls into modules |
| `ThemeManager` | `ThemeController` (in `Cells.theme`) |
| dead `TimerController` | `RecipeDetailsController` + `TimerState` |
| `.route({initial})` helpers, post-frame `RecipeSelect` | `RouteArgs` + page-scoped controllers |

---

## 10. Testing strategy

- **Domain (the bulk):** pure unit tests with plain values — `StepDuration`
  bounds, `RecipeName` emptiness, `BrewMetrics` caps/formulas, and especially
  `BrewTimeline` (start / tick / step-advance / finish) with no `Timer` or DB.
  These are the tests that "make sure it won't fail."
- **Infrastructure:** a handful of repository tests against an in-memory SQLite
  DB (round-trip a `Recipe`, settings kv).
- **Controllers:** thin — inject fake/in-memory ports + no-op effect modules;
  assert state transitions and that the right effect module method was called on
  a transition. Because decisions live in the domain, controller tests need no
  business assertions.
- **Widgets:** minimal; cosmetics only.

Guiding check per new conditional (from the source contract): if it's a business
decision it is **not** in a controller/repo/timer callback; it sits at the
smallest sufficient level; it has a name and one home; no public path yields an
invalid state; and it's testable with plain values.
