import 'package:coffee_timer/application/recipe/recipe_ports.dart';
import 'package:coffee_timer/domain/recipe/recipe_id.dart';

/// Deletes a recipe by id. Orchestration only.
class DeleteRecipe {
  const DeleteRecipe(this._writes);

  final RecipeWriteRepository _writes;

  Future<void> call(RecipeId id) => _writes.delete(id);
}
