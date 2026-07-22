import 'package:coffee_timer/application/recipe/recipe_ports.dart';
import 'package:coffee_timer/domain/recipe/recipe.dart';
import 'package:coffee_timer/domain/recipe/recipe_id.dart';
import 'package:coffee_timer/domain/recipe/recipe_name.dart';
import 'package:coffee_timer/domain/recipe/recipe_step.dart';
import 'package:coffee_timer/domain/recipe/step_duration.dart';
import 'package:coffee_timer/domain/recipe/step_id.dart';
import 'package:sqlite_async/sqlite_async.dart';

/// SQLite storage for recipes, over the `recipe` / `step` tables.
///
/// This is the only place raw rows appear: the smart constructors
/// ([RecipeId.tryParse], [RecipeName.tryParse], [StepDuration.clamped], …) run
/// here at the boundary, so everything above holds valid domain values. A row
/// whose primary key is somehow blank (impossible under the schema) is dropped
/// rather than smuggled up as an invalid entity.
///
/// The schema is owned elsewhere: `AppDatabase` runs all migrations at boot
/// before this repository is handed the open database, so a query can never
/// race ahead of table creation.
class SqliteRecipeRepository
    implements RecipeReadRepository, RecipeWriteRepository {
  SqliteRecipeRepository(this._db);

  final SqliteDatabase _db;

  @override
  Future<List<Recipe>> all() async {
    final recipeRows = await _db.getAll('SELECT * FROM recipe');
    final stepRows = await _db.getAll('SELECT * FROM step');

    final stepsByRecipe = <String, List<RecipeStep>>{};
    for (final row in stepRows) {
      final step = _stepFromRow(row);
      if (step == null) continue;
      stepsByRecipe
          .putIfAbsent(row['recipe_id'] as String, () => [])
          .add(step);
    }

    return recipeRows
        .map(
          (row) => _recipeFromRow(
            row,
            stepsByRecipe[row['id'] as String] ?? const [],
          ),
        )
        .whereType<Recipe>()
        .toList();
  }

  @override
  Future<Recipe?> byId(RecipeId id) async {
    final recipeRows = await _db.getAll(
      'SELECT * FROM recipe WHERE id = ?',
      [id.value],
    );
    if (recipeRows.isEmpty) return null;

    final stepRows = await _db.getAll(
      'SELECT * FROM step WHERE recipe_id = ?',
      [id.value],
    );
    final steps = stepRows.map(_stepFromRow).whereType<RecipeStep>().toList();

    return _recipeFromRow(recipeRows.first, steps);
  }

  @override
  Future<Recipe> upsert(Recipe recipe) async {
    await _db.writeTransaction((tx) async {
      await tx.execute(
        'INSERT INTO recipe (id, name, description) VALUES (?, ?, ?) '
        'ON CONFLICT(id) DO UPDATE SET '
        'name=excluded.name, description=excluded.description',
        [recipe.id.value, recipe.name.value, recipe.description],
      );

      await tx.execute('DELETE FROM step WHERE recipe_id = ?', [
        recipe.id.value,
      ]);
      for (final step in recipe.steps) {
        await tx.execute(
          'INSERT INTO step (id, recipe_id, name, duration_seconds) '
          'VALUES (?, ?, ?, ?)',
          [step.id.value, recipe.id.value, step.name, step.duration.seconds],
        );
      }
    });
    return recipe;
  }

  @override
  Future<void> delete(RecipeId id) async {
    await _db.writeTransaction((tx) async {
      await tx.execute('DELETE FROM step WHERE recipe_id = ?', [id.value]);
      await tx.execute('DELETE FROM recipe WHERE id = ?', [id.value]);
    });
  }

  // --- boundary mapping: smart constructors run here ---

  Recipe? _recipeFromRow(
    Map<String, dynamic> row,
    List<RecipeStep> steps,
  ) {
    final id = RecipeId.tryParse(row['id'] as String);
    if (id == null) return null;
    return Recipe(
      id: id,
      name: RecipeName.tryParse(row['name'] as String) ?? RecipeName.untitled,
      description: row['description'] as String,
      steps: steps,
    );
  }

  RecipeStep? _stepFromRow(Map<String, dynamic> row) {
    final id = StepId.tryParse(row['id'] as String);
    if (id == null) return null;
    return RecipeStep(
      id: id,
      name: row['name'] as String,
      duration: StepDuration.clamped(row['duration_seconds'] as int),
    );
  }
}
