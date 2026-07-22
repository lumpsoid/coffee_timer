import 'package:coffee_timer/domain/recipe/recipe_id.dart';
import 'package:coffee_timer/domain/recipe/recipe_name.dart';
import 'package:coffee_timer/domain/recipe/recipe_step.dart';

/// An immutable brew recipe. Every field is a valid-by-construction value; the
/// derived brew figures (grams, total duration) live in `BrewMetrics`, not
/// here.
///
/// Mutation is expressed as named operations (`renamed`, `withStepAdded`, …)
/// that each return a new instance — there are no public setters, so a `Recipe`
/// you hold never changes underneath you.
class Recipe {
  const Recipe({
    required this.id,
    required this.name,
    required this.description,
    required this.steps,
  });

  /// A fresh recipe with no description and no steps — the starting point for
  /// the create flow. Build it up with `describedAs` / `withStepAdded`.
  factory Recipe.blank({required RecipeId id, required RecipeName name}) =>
      Recipe(id: id, name: name, description: '', steps: const []);

  final RecipeId id;
  final RecipeName name;
  final String description;
  final List<RecipeStep> steps;

  Recipe renamed(RecipeName name) => copyWith(name: name);

  Recipe describedAs(String description) => copyWith(description: description);

  Recipe withStepAdded(RecipeStep step) =>
      copyWith(steps: [...steps, step]);

  Recipe withStepRemovedAt(int index) =>
      copyWith(steps: [...steps]..removeAt(index));

  Recipe copyWith({
    RecipeId? id,
    RecipeName? name,
    String? description,
    List<RecipeStep>? steps,
  }) => Recipe(
    id: id ?? this.id,
    name: name ?? this.name,
    description: description ?? this.description,
    steps: steps ?? this.steps,
  );
}
