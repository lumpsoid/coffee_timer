import 'package:coffee_timer/domain/recipe/recipe_id.dart';
import 'package:coffee_timer/domain/recipe/step_id.dart';

/// Mints fresh identifiers for newly created recipes and steps.
///
/// A port so the domain/ui stay free of `uuid`: minting an id is a boundary
/// concern (see `docs/pivot-spec.md`). The SQLite-side implementation is
/// `UuidRecipeIds`; a test can inject a deterministic fake.
abstract interface class RecipeIds {
  RecipeId newRecipeId();
  StepId newStepId();
}
