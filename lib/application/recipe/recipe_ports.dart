import 'package:coffee_timer/domain/recipe/recipe.dart';
import 'package:coffee_timer/domain/recipe/recipe_id.dart';

/// Reads persisted recipes. Narrow — read only.
abstract interface class RecipeReadRepository {
  Future<List<Recipe>> all();
  Future<Recipe?> byId(RecipeId id);
}

/// Persists recipes. Narrow — write only.
abstract interface class RecipeWriteRepository {
  Future<Recipe> upsert(Recipe recipe);
  Future<void> delete(RecipeId id);
}
