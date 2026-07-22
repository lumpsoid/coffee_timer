import 'package:coffee_timer/domain/recipe/step_duration.dart';
import 'package:coffee_timer/domain/recipe/step_id.dart';

/// One step of a brew: an id, a name, and how long it runs. Immutable — a step
/// is replaced, never edited in place.
class RecipeStep {
  const RecipeStep({
    required this.id,
    required this.name,
    required this.duration,
  });

  final StepId id;
  final String name;
  final StepDuration duration;
}
