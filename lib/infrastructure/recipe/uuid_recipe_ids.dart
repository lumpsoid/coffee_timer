import 'package:coffee_timer/application/recipe/recipe_ids.dart';
import 'package:coffee_timer/domain/recipe/recipe_id.dart';
import 'package:coffee_timer/domain/recipe/step_id.dart';
import 'package:uuid/uuid.dart';

/// [RecipeIds] backed by UUID v7. This is the boundary where a raw string turns
/// into a value object: `v7()` is never empty, so the smart constructors always
/// succeed — the `!` is safe by construction.
class UuidRecipeIds implements RecipeIds {
  const UuidRecipeIds([this._uuid = const Uuid()]);

  final Uuid _uuid;

  @override
  RecipeId newRecipeId() => RecipeId.tryParse(_uuid.v7())!;

  @override
  StepId newStepId() => StepId.tryParse(_uuid.v7())!;
}
