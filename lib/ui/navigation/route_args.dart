/// Base type for the typed arguments a page needs.
///
/// Concrete args (e.g. `RecipeDetailsArgs`, `RecipeEditArgs`) are added with
/// their feature during migration; navigation carries these instead of routing
/// selection through a global event. See `docs/pivot-spec.md`.
sealed class RouteArgs {
  const RouteArgs();
}
