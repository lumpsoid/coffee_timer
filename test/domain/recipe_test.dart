import 'package:coffee_timer/domain/recipe/brew_metrics.dart';
import 'package:coffee_timer/domain/recipe/recipe.dart';
import 'package:coffee_timer/domain/recipe/recipe_id.dart';
import 'package:coffee_timer/domain/recipe/recipe_name.dart';
import 'package:coffee_timer/domain/recipe/recipe_step.dart';
import 'package:coffee_timer/domain/recipe/step_duration.dart';
import 'package:coffee_timer/domain/recipe/step_id.dart';
import 'package:flutter_test/flutter_test.dart';

RecipeStep _step(int seconds) => RecipeStep(
  id: StepId.tryParse('s$seconds')!,
  name: 'step',
  duration: StepDuration.clamped(seconds),
);

Recipe _recipe(List<int> stepSeconds) => Recipe(
  id: RecipeId.tryParse('r')!,
  name: RecipeName.tryParse('Recipe')!,
  description: '',
  steps: stepSeconds.map(_step).toList(),
);

void main() {
  group('RecipeId / StepId', () {
    test('reject empty, keep non-empty, and compare by value', () {
      expect(RecipeId.tryParse(''), isNull);
      expect(StepId.tryParse(''), isNull);
      expect(RecipeId.tryParse('a'), RecipeId.tryParse('a'));
      expect(RecipeId.tryParse('a') == RecipeId.tryParse('b'), isFalse);
    });
  });

  group('RecipeName', () {
    test('trims and rejects blank', () {
      expect(RecipeName.tryParse('  V60  ')?.value, 'V60');
      expect(RecipeName.tryParse('   '), isNull);
      expect(RecipeName.tryParse(''), isNull);
    });

    test('untitled fallback is a valid name', () {
      expect(RecipeName.untitled.value, 'Untitled');
    });
  });

  group('StepDuration', () {
    test('tryParse enforces bounds strictly', () {
      expect(StepDuration.tryParse(-1), isNull);
      expect(StepDuration.tryParse(0)?.seconds, 0);
      expect(StepDuration.tryParse(StepDuration.maxStepSeconds)?.seconds, 5999);
      expect(StepDuration.tryParse(StepDuration.maxStepSeconds + 1), isNull);
    });

    test('clamped never leaves the range', () {
      expect(StepDuration.clamped(-5).seconds, 0);
      expect(StepDuration.clamped(90).seconds, 90);
      expect(StepDuration.clamped(10000).seconds, StepDuration.maxStepSeconds);
    });

    test('mmss zero-pads both parts', () {
      expect(StepDuration.clamped(0).mmss, '00:00');
      expect(StepDuration.clamped(65).mmss, '01:05');
      expect(StepDuration.clamped(600).mmss, '10:00');
    });
  });

  group('RecipeStep.labelOrDefault', () {
    test('keeps trimmed input, falls back to Step N when blank', () {
      expect(RecipeStep.labelOrDefault('  Bloom ', 1), 'Bloom');
      expect(RecipeStep.labelOrDefault('', 3), 'Step 3');
      expect(RecipeStep.labelOrDefault('   ', 2), 'Step 2');
    });
  });

  group('Recipe mutations', () {
    final base = _recipe([30, 45]);

    test('renamed / describedAs return new instances, id preserved', () {
      final renamed = base.renamed(RecipeName.tryParse('V60')!);
      expect(renamed.name.value, 'V60');
      expect(renamed.id, base.id);
      expect(base.name.value, 'Recipe', reason: 'original untouched');

      expect(base.describedAs('pour slowly').description, 'pour slowly');
    });

    test('withStepAdded appends without mutating the original', () {
      final added = base.withStepAdded(_step(10));
      expect(added.steps.length, 3);
      expect(base.steps.length, 2);
    });

    test('withStepRemovedAt drops the step at the index', () {
      final removed = base.withStepRemovedAt(0);
      expect(removed.steps.single.duration.seconds, 45);
      expect(base.steps.length, 2);
    });

    test('blank starts empty and builds up', () {
      final built = Recipe.blank(
        id: RecipeId.tryParse('x')!,
        name: RecipeName.tryParse('Aero')!,
      ).describedAs('quick').withStepAdded(_step(20));
      expect(built.description, 'quick');
      expect(built.steps.single.duration.seconds, 20);
    });
  });

  group('BrewMetrics', () {
    const metrics = BrewMetrics();

    test('durationLabelOf formats a raw seconds list', () {
      expect(metrics.durationLabelOf([65]), '1:05');
      expect(metrics.durationLabelOf([]), '0:00');
      expect(metrics.durationLabelOf([5999, 5999]), '99:99');
    });

    test('grams scale with step count', () {
      final r = _recipe([30, 30, 30]);
      expect(metrics.coffeeGrams(r), 30);
      expect(metrics.waterGrams(r), 150);
    });

    test('totalSeconds sums steps', () {
      expect(metrics.totalSeconds(_recipe([30, 45, 15])), 90);
      expect(metrics.totalSeconds(_recipe([])), 0);
    });

    test('totalSeconds clamps at the cap', () {
      // three near-max steps blow past 6000
      final r = _recipe([5999, 5999, 5999]);
      expect(metrics.totalSeconds(r), 6000);
    });

    test('durationLabel formats M:SS with unpadded minutes', () {
      expect(metrics.durationLabel(_recipe([0])), '0:00');
      expect(metrics.durationLabel(_recipe([65])), '1:05');
      expect(metrics.durationLabel(_recipe([600])), '10:00');
    });

    test('durationLabel reads 99:99 at the cap', () {
      expect(metrics.durationLabel(_recipe([5999, 5999])), '99:99');
    });
  });
}
