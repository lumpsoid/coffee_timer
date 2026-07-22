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

  /// The name to store for a step at [position] (1-based), given the raw text a
  /// user typed. Blank input falls back to `Step N` — the one home for that
  /// rule, so no form re-implements it.
  static String labelOrDefault(String raw, int position) {
    final trimmed = raw.trim();
    return trimmed.isEmpty ? 'Step $position' : trimmed;
  }
}
