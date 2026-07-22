import 'dart:io';

import 'package:coffee_timer/domain/recipe/recipe.dart';
import 'package:coffee_timer/domain/recipe/recipe_id.dart';
import 'package:coffee_timer/domain/recipe/recipe_name.dart';
import 'package:coffee_timer/domain/recipe/recipe_step.dart';
import 'package:coffee_timer/domain/recipe/step_duration.dart';
import 'package:coffee_timer/domain/recipe/step_id.dart';
import 'package:coffee_timer/infrastructure/database/app_migrations.dart';
import 'package:coffee_timer/infrastructure/recipe/sqlite_recipe_repository.dart';
import 'package:coffee_timer/sql_migrator/sql_migrator.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:sqlite_async/sqlite_async.dart';

class _SilentLogger implements MigrationLogger {
  const _SilentLogger();
  @override
  void info(String message) {}
  @override
  void warning(String message) {}
  @override
  void error(String message, [Object? error, StackTrace? stackTrace]) {}
}

void main() {
  late Directory dir;
  late SqliteDatabase db;
  late SqliteRecipeRepository repo;

  setUp(() async {
    dir = await Directory.systemTemp.createTemp('recipe_repo_test');
    db = SqliteDatabase(path: '${dir.path}/test.db');
    // Real migrations, but stop at v1 (create tables) so the seed rows (v2)
    // don't land — the tests own their data.
    await Migrator(
      executor: SqliteAsyncExecutor(db),
      migrations: appMigrations,
      config: const MigratorConfig(useLocking: false),
      logger: const _SilentLogger(),
    ).migrateTo(1);
    repo = SqliteRecipeRepository(db);
  });

  tearDown(() async {
    await db.close();
    await dir.delete(recursive: true);
  });

  Recipe recipe(String id, {List<int> steps = const [30, 45]}) => Recipe(
    id: RecipeId.tryParse(id)!,
    name: RecipeName.tryParse('Recipe $id')!,
    description: 'desc $id',
    steps: [
      for (var i = 0; i < steps.length; i++)
        RecipeStep(
          id: StepId.tryParse('$id-s$i')!,
          name: 'step $i',
          duration: StepDuration.clamped(steps[i]),
        ),
    ],
  );

  test('upsert then all round-trips a recipe with its steps', () async {
    await repo.upsert(recipe('a', steps: [30, 45, 60]));

    final all = await repo.all();
    expect(all, hasLength(1));
    final loaded = all.single;
    expect(loaded.id, RecipeId.tryParse('a'));
    expect(loaded.name.value, 'Recipe a');
    expect(loaded.description, 'desc a');
    expect(loaded.steps.map((s) => s.duration.seconds), [30, 45, 60]);
  });

  test('upsert on an existing id replaces its steps', () async {
    await repo.upsert(recipe('a', steps: [30, 45]));
    await repo.upsert(recipe('a', steps: [10]));

    final loaded = await repo.byId(RecipeId.tryParse('a')!);
    expect(loaded, isNotNull);
    expect(loaded!.steps.map((s) => s.duration.seconds), [10]);
  });

  test('byId returns null for an unknown id', () async {
    expect(await repo.byId(RecipeId.tryParse('missing')!), isNull);
  });

  test('delete removes the recipe and its steps', () async {
    await repo.upsert(recipe('a'));
    await repo.upsert(recipe('b'));

    await repo.delete(RecipeId.tryParse('a')!);

    final all = await repo.all();
    expect(all.map((r) => r.id.value), ['b']);
    // steps for the deleted recipe are gone too
    final orphanSteps = await db.getAll(
      'SELECT * FROM step WHERE recipe_id = ?',
      ['a'],
    );
    expect(orphanSteps, isEmpty);
  });
}
