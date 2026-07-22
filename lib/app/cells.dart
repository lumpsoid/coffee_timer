/// Static roots the UI reads to reach shared modules and long-lived
/// controllers. Populated once by `AppRoot`.
///
/// Empty during the Phase 0 scaffold — modules are registered here as each
/// feature is migrated off the event bus. See `docs/pivot-spec.md`.
///
/// Later phases add, e.g.:
/// ```dart
/// static final theme = Cell<ThemeController>();
/// static final recipeModule = Cell<RecipeModule>();
/// static final settingsModule = Cell<SettingsModule>();
/// ```
abstract final class Cells {}
