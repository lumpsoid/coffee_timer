import 'dart:async';

import 'package:coffee_timer/core/di/hub.dart';
import 'package:coffee_timer/core/events/events.dart';
import 'package:coffee_timer/core/recipe/entities/entities.dart';
import 'package:coffee_timer/core/recipe/events/events.dart';
import 'package:sqlite_async/sqlite_async.dart';

class RecipeStorageSqlite extends Leaf {
  RecipeStorageSqlite({
    required SqliteDatabase db,
  }) : _db = db;

  final SqliteDatabase _db;

  @override
  Future<void> initialize() async {
    // Migrations are now owned by SqliteRecipeRepository (the new stack), which
    // runs them once at boot. Running them here too would race and double-seed.
  }

  @override
  Future<void> handleEvent(Event event) async {
    switch (event) {
      case RecipesLoadStorage():
        final recipes = await _loadRecipes();
        emit(RecipesLoadedStorage(recipes));

      case RecipeCreateStorage(:final recipe):
        final createdRecipe = await _createRecipe(recipe);
        emit(RecipeCreatedStorage(createdRecipe));

      case RecipeUpdateStorage(:final recipe):
        final updatedRecipe = await _updateRecipe(recipe);
        emit(RecipeUpdatedStorage(updatedRecipe));

      case RecipeDeleteStorage(:final id):
        await _deleteRecipe(id);
        emit(RecipeDeletedStorage(id));

      default:
        break;
    }
  }

  Future<List<Recipe>> _loadRecipes() async {
    final recipeRows = await _db.getAll('SELECT * FROM recipe');
    final stepsRows = await _db.getAll('SELECT * FROM step');

    final stepsMap = <String, List<RecipeStep>>{};

    for (final row in stepsRows) {
      final recipeId = row['recipe_id'] as String;
      final step = RecipeStep(
        id: row['id'] as String,
        name: row['name'] as String,
        durationSeconds: row['duration_seconds'] as int,
      );
      stepsMap.putIfAbsent(recipeId, () => []).add(step);
    }

    return recipeRows.map((row) {
      final id = row['id'] as String;
      return Recipe(
        id: id,
        name: row['name'] as String,
        description: row['description'] as String,
        steps: stepsMap[id] ?? [],
      );
    }).toList();
  }

  Future<Recipe> _createRecipe(Recipe recipe) async {
    return _db.writeTransaction((tx) async {
      // Insert recipe and return the inserted row
      final recipeRows = await tx.getAll(
        'INSERT INTO recipe (id, name, description) VALUES (?, ?, ?)'
        ' RETURNING id, name, description',
        [recipe.id, recipe.name, recipe.description],
      );
      final insertedRecipeRow = recipeRows.first;

      // Insert steps and collect returned rows
      final insertedSteps = <RecipeStep>[];
      for (final step in recipe.steps) {
        final stepRows = await tx.getAll(
          'INSERT INTO step (id, recipe_id, name, duration_seconds) VALUES (?, ?, ?, ?)'
          ' RETURNING id, recipe_id, name, duration_seconds',
          [step.id, recipe.id, step.name, step.durationSeconds],
        );
        final r = stepRows.first;
        insertedSteps.add(
          RecipeStep(
            id: r['id'] as String,
            name: r['name'] as String,
            durationSeconds: r['duration_seconds'] as int,
          ),
        );
      }

      return Recipe(
        id: insertedRecipeRow['id'] as String,
        name: insertedRecipeRow['name'] as String,
        description: insertedRecipeRow['description'] as String,
        steps: insertedSteps,
      );
    });
  }

  Future<Recipe> _updateRecipe(Recipe recipe) async {
    return _db.writeTransaction((tx) async {
      // upsert recipe and return row
      final recipeRows = await tx.getAll(
        'INSERT INTO recipe (id, name, description) VALUES (?, ?, ?) '
        'ON CONFLICT(id) DO UPDATE SET name=excluded.name, description=excluded.description '
        'RETURNING id, name, description',
        [recipe.id, recipe.name, recipe.description],
      );
      final savedRecipeRow = recipeRows.first;

      // remove old steps and insert new ones, returning inserted rows
      await tx.execute('DELETE FROM step WHERE recipe_id = ?', [recipe.id]);

      final insertedSteps = <RecipeStep>[];
      for (final step in recipe.steps) {
        final stepRows = await tx.getAll(
          'INSERT INTO step (id, recipe_id, name, duration_seconds) '
          'VALUES (?, ?, ?, ?) RETURNING id, recipe_id, name, duration_seconds',
          [step.id, recipe.id, step.name, step.durationSeconds],
        );
        final r = stepRows.first;
        insertedSteps.add(
          RecipeStep(
            id: r['id'] as String,
            name: r['name'] as String,
            durationSeconds: r['duration_seconds'] as int,
          ),
        );
      }

      return Recipe(
        id: savedRecipeRow['id'] as String,
        name: savedRecipeRow['name'] as String,
        description: savedRecipeRow['description'] as String,
        steps: insertedSteps,
      );
    });
  }

  Future<void> _deleteRecipe(String id) async {
    await _db.execute('DELETE FROM step WHERE recipe_id = ?', [id]);
    await _db.execute('DELETE FROM recipe WHERE id = ?', [id]);
  }
}
