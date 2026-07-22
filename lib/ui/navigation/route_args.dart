import 'package:coffee_timer/domain/recipe/recipe.dart';
import 'package:coffee_timer/domain/recipe/recipe_id.dart';

/// Base type for the typed arguments a page needs.
///
/// Navigation carries these instead of routing selection through a global
/// event. See `docs/pivot-spec.md`.
sealed class RouteArgs {
  const RouteArgs();
}

/// Identifies the recipe whose details/timer to open.
class RecipeDetailsArgs extends RouteArgs {
  const RecipeDetailsArgs(this.recipeId);

  final RecipeId recipeId;
}

/// Whether the edit page opens on a blank recipe ([RecipeEditArgs.create]) or
/// an existing one ([RecipeEditArgs.edit]).
class RecipeEditArgs extends RouteArgs {
  const RecipeEditArgs.create() : initial = null;
  const RecipeEditArgs.edit(this.initial);

  final Recipe? initial;

  bool get isCreate => initial == null;
}
