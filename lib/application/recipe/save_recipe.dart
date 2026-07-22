import 'package:coffee_timer/application/recipe/recipe_ports.dart';
import 'package:coffee_timer/domain/recipe/recipe.dart';

/// Persists a recipe (insert or update). Orchestration only — the controller
/// has already assembled a valid domain [Recipe]; this just hands it to the
/// write port.
class SaveRecipe {
  const SaveRecipe(this._writes);

  final RecipeWriteRepository _writes;

  Future<Recipe> call(Recipe recipe) => _writes.upsert(recipe);
}
