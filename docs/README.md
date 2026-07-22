# Coffee Timer — Documentation

A Flutter app for running multi-step coffee brewing recipes with a per-step
countdown timer, audio cue, vibration, and screen-wakelock. Recipes are stored
locally in SQLite.

This folder documents **how the app works today** (reverse-engineered from the
code). It is descriptive, not prescriptive — where the code has quirks or dead
paths, they are called out in [Known issues & quirks](#known-issues--quirks).

## Contents

| Doc | What's in it |
|-----|--------------|
| [architecture.md](architecture.md) | The custom event-bus / actor runtime, the two independent buses, event-flow patterns, and the unused component runtime |
| [modules.md](modules.md) | Per-module reference — the `core/` domain (recipe, settings, theme, hardware leaves) and the feature UI (`recipe_list`, `recipe_details`, `recipe_edit`, `settings`) |

## What the app does

- **Recipe list** — browse recipes, create/edit/delete them.
- **Recipe details** — a large circular timer that counts down each brewing
  step; advancing a step vibrates + plays a sound, the last step ends the run.
  Keeps the screen awake while brewing.
- **Recipe edit** — a form to build a recipe out of named, timed steps.
- **Settings** — theme (light / dark / system) and accent color, persisted.

## Tech stack

- **Flutter**, Dart SDK `^3.9.2`.
- **Persistence:** `sqlite_async` + `sqlite3_flutter_libs`, DB file at
  `<app documents>/recipes.db`.
- **Effects:** `audioplayers` (step-change sound), `vibration`, `wakelock_plus`.
- **Utilities:** `uuid` (v7 ids), `path` / `path_provider`, `provider`
  (declared, but the app mostly uses `ValueNotifier` + plain streams).
- **Assets:** `assets/sounds/step-change.mp3`.

## Top-level layout

```
lib/
├── main.dart                 App bootstrap: wires buses, leaves, storage, theme
├── core/                     Domain + runtime (no Flutter widgets, except theme)
│   ├── di/hub.dart           The Hub singleton + Leaf base class (the runtime)
│   ├── di.dart               `final kernel = Hub()`
│   ├── modules/base_module.dart   A separate, UNUSED component/port runtime
│   ├── recipe/               Recipe domain: entities, actor, events, storage
│   ├── settings/             Settings domain: entity, actor, events, storage, bus
│   ├── theme/                ThemeManager (ChangeNotifier)
│   ├── audioplayers/, vibrate/, wakelock/   Hardware "leaves"
│   └── events/               Cross-cutting effect events + Event base class
├── recipe_list/              Feature UI: list page + summary tile
├── recipe_details/           Feature UI: timer page + circular-timer widgets
├── recipe_edit/              Feature UI: create/edit form
└── settings/                 Feature UI: settings page
```

## Known issues & quirks

These are documented so the docs match reality; each is described in more
detail in the linked docs.

1. **`SettingsActor` is never subscribed.** In `main.dart` a `SettingsActor` is
   constructed but its `handleEvent` is not wired to `SettingsEventBus.stream`.
   The theme/accent-color update flow that goes *through* the actor
   (`SettingsThemeChanged`/`SettingsAccentColorChanged` → `SettingsUpdate`) has
   no live handler. See [modules.md → Settings](modules.md#settings-core).
2. **`TimerController` is dead code.** `recipe_details/notifiers/timer_controller.dart`
   is a self-contained countdown `ChangeNotifier` that nothing references. The
   real timer engine is `RecipeActor`. See
   [modules.md → recipe_details](modules.md#feature-recipe_details-the-timer).
3. **Two parallel bus implementations.** The recipe domain uses the `Hub`
   singleton; settings uses a standalone `SettingsEventBus`. There is also an
   abstract `RecipeEventBus` that is defined but not the one `main.dart` wires
   up. See [architecture.md](architecture.md#the-buses).
4. **`base_module.dart` is an unused runtime.** A full flow-based
   component/port/connector engine that no code imports.
5. **The circular progress arc is a per-step sweep**, driven by an
   `AnimationController` whose duration equals the step length — it is decoupled
   from the actual second-by-second countdown that drives the MM:SS readout.
6. **`MigrationRunner.run` ignores its `migrations` argument** and always uses
   the global `allMigrations`.
7. **`Settings.fromMap`/`toMap` are asymmetric** — `fromMap` reads
   `accent_color` as an int, `toMap` writes it as a `#AARRGGBB` string (the
   SQLite layer stores the string form).
