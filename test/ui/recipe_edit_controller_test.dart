import 'package:coffee_timer/application/recipe/delete_recipe.dart';
import 'package:coffee_timer/application/recipe/recipe_ids.dart';
import 'package:coffee_timer/application/recipe/recipe_ports.dart';
import 'package:coffee_timer/application/recipe/save_recipe.dart';
import 'package:coffee_timer/domain/recipe/recipe.dart';
import 'package:coffee_timer/domain/recipe/recipe_id.dart';
import 'package:coffee_timer/domain/recipe/recipe_name.dart';
import 'package:coffee_timer/domain/recipe/recipe_step.dart';
import 'package:coffee_timer/domain/recipe/step_duration.dart';
import 'package:coffee_timer/domain/recipe/step_id.dart';
import 'package:coffee_timer/ui/navigation/route_args.dart';
import 'package:coffee_timer/ui/recipe_edit/recipe_edit_controller.dart';
import 'package:flutter_test/flutter_test.dart';

/// Records writes; reads are never used by the editor.
class _FakeWriteRepository implements RecipeWriteRepository {
  Recipe? saved;
  RecipeId? deleted;
  Object? failWith;

  @override
  Future<Recipe> upsert(Recipe recipe) async {
    if (failWith != null) throw failWith!;
    return saved = recipe;
  }

  @override
  Future<void> delete(RecipeId id) async {
    if (failWith != null) throw failWith!;
    deleted = id;
  }
}

/// Deterministic id minting: `r1`, `r2`… for recipes, `s1`, `s2`… for steps.
class _FakeIds implements RecipeIds {
  int _recipe = 0;
  int _step = 0;

  @override
  RecipeId newRecipeId() => RecipeId.tryParse('r${++_recipe}')!;

  @override
  StepId newStepId() => StepId.tryParse('s${++_step}')!;
}

RecipeEditController _controller(
  RecipeEditArgs args, {
  _FakeWriteRepository? repo,
  _FakeIds? ids,
}) {
  final writes = repo ?? _FakeWriteRepository();
  return RecipeEditController(
    save: SaveRecipe(writes),
    delete: DeleteRecipe(writes),
    ids: ids ?? _FakeIds(),
    args: args,
  );
}

Recipe _existing() => Recipe(
  id: RecipeId.tryParse('recipe-1')!,
  name: RecipeName.tryParse('V60')!,
  description: 'clean',
  steps: [
    RecipeStep(
      id: StepId.tryParse('step-1')!,
      name: 'Bloom',
      duration: StepDuration.clamped(30),
    ),
  ],
);

void main() {
  group('create', () {
    test('starts with one blank step and no name', () {
      final c = _controller(const RecipeEditArgs.create());
      expect(c.isEditing, isFalse);
      expect(c.name, '');
      expect(c.rows.length, 1);
    });

    test('assembles a recipe: minted ids, default step name, clamped duration',
        () async {
      final repo = _FakeWriteRepository();
      final c = _controller(const RecipeEditArgs.create(), repo: repo);

      c.name = 'Aeropress';
      c.description = '  strong  ';
      c.setStepDuration(0, '10000'); // over the cap → clamped

      final saved = await c.save();

      expect(saved, isNotNull);
      expect(repo.saved, same(saved));
      expect(saved!.id.value, 'r1');
      expect(saved.name.value, 'Aeropress');
      expect(saved.description, 'strong');
      final step = saved.steps.single;
      expect(step.id.value, 's1');
      expect(step.name, 'Step 1'); // blank → default
      expect(step.duration.seconds, StepDuration.maxStepSeconds);
    });

    test('blank name is rejected and nothing is written', () async {
      final repo = _FakeWriteRepository();
      final c = _controller(const RecipeEditArgs.create(), repo: repo);
      c.name = '   ';

      final saved = await c.save();

      expect(saved, isNull);
      expect(c.error, 'Enter a recipe name');
      expect(repo.saved, isNull);
    });

    test('a write failure surfaces as an error, not an exception', () async {
      final repo = _FakeWriteRepository()..failWith = StateError('boom');
      final c = _controller(const RecipeEditArgs.create(), repo: repo);
      c.name = 'X';

      final saved = await c.save();

      expect(saved, isNull);
      expect(c.error, contains('Failed to save recipe'));
    });
  });

  group('edit', () {
    test('prefills from the initial recipe', () {
      final c = _controller(RecipeEditArgs.edit(_existing()));
      expect(c.isEditing, isTrue);
      expect(c.name, 'V60');
      expect(c.description, 'clean');
      expect(c.rows.single.duration, '30');
    });

    test('save preserves the recipe id and existing step ids', () async {
      final repo = _FakeWriteRepository();
      final c = _controller(RecipeEditArgs.edit(_existing()), repo: repo);
      c.setStepDuration(0, '45');

      final saved = await c.save();

      expect(saved!.id.value, 'recipe-1');
      expect(saved.steps.single.id.value, 'step-1'); // not re-minted
      expect(saved.steps.single.duration.seconds, 45);
    });

    test('a newly added step gets a minted id, existing one is kept', () async {
      final repo = _FakeWriteRepository();
      final ids = _FakeIds();
      final c = _controller(
        RecipeEditArgs.edit(_existing()),
        repo: repo,
        ids: ids,
      );
      c.addStep('Pour');

      final saved = await c.save();

      expect(saved!.steps.map((s) => s.id.value), ['step-1', 's1']);
      expect(saved.steps.last.name, 'Pour');
    });
  });

  group('steps', () {
    test('removeStepAt keeps at least one row', () {
      final c = _controller(const RecipeEditArgs.create());
      c.removeStepAt(0);
      expect(c.rows, hasLength(1));
    });

    test('totalDurationLabel reflects typed, clamped durations', () {
      final c = _controller(const RecipeEditArgs.create())
        ..setStepDuration(0, '65')
        ..addStep();
      c.setStepDuration(1, '600');
      expect(c.totalDurationLabel, '11:05');
    });
  });

  group('delete', () {
    test('edit mode deletes the initial recipe', () async {
      final repo = _FakeWriteRepository();
      final c = _controller(RecipeEditArgs.edit(_existing()), repo: repo);

      expect(await c.deleteExisting(), isTrue);
      expect(repo.deleted?.value, 'recipe-1');
    });

    test('create mode has nothing to delete', () async {
      final repo = _FakeWriteRepository();
      final c = _controller(const RecipeEditArgs.create(), repo: repo);

      expect(await c.deleteExisting(), isFalse);
      expect(repo.deleted, isNull);
    });
  });
}
