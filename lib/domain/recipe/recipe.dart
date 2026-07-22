import 'package:coffee_timer/domain/recipe/recipe_id.dart';
import 'package:coffee_timer/domain/recipe/recipe_name.dart';
import 'package:coffee_timer/domain/recipe/recipe_step.dart';

/// An immutable brew recipe. Every field is a valid-by-construction value; the
/// derived brew figures (grams, total duration) live in `BrewMetrics`, not
/// here.
///
/// Named mutation operations (`renamed`, `withStepAdded`, …) arrive with the
/// edit feature; this phase only reads recipes.
class Recipe {
  const Recipe({
    required this.id,
    required this.name,
    required this.description,
    required this.steps,
  });

  final RecipeId id;
  final RecipeName name;
  final String description;
  final List<RecipeStep> steps;
}
