import 'package:coffee_timer/domain/recipe/recipe.dart';

/// The named home for the brew figures shown on a recipe: coffee grams, water
/// grams, and total duration. Every threshold and formula lives here and
/// nowhere else, so a one-line business change touches one file.
///
/// Pure and testable with plain values — no `Recipe` method reaches into these
/// rules, and no controller re-implements them.
class BrewMetrics {
  const BrewMetrics({
    this.coffeePerStepG = 10,
    this.waterPerStepG = 50,
    this.totalCapSeconds = 6000,
  });

  final int coffeePerStepG;
  final int waterPerStepG;

  /// Total brew duration is capped here (matches the legacy `6000`s clamp).
  final int totalCapSeconds;

  int coffeeGrams(Recipe r) => r.steps.length * coffeePerStepG;

  int waterGrams(Recipe r) => r.steps.length * waterPerStepG;

  /// Sum of every step's duration, clamped to [totalCapSeconds].
  int totalSeconds(Recipe r) =>
      totalSecondsOf(r.steps.map((s) => s.duration.seconds));

  /// `M:SS` total-duration label. Minutes are unpadded; at the cap it reads
  /// `99:99` (both preserved from the legacy display).
  String durationLabel(Recipe r) =>
      durationLabelOf(r.steps.map((s) => s.duration.seconds));

  /// Sum of [stepSeconds], clamped to [totalCapSeconds]. Used both by
  /// [totalSeconds] and directly by the edit form, which needs a live total
  /// before it has assembled a `Recipe`.
  int totalSecondsOf(Iterable<int> stepSeconds) {
    var total = 0;
    for (final seconds in stepSeconds) {
      total += seconds;
      if (total >= totalCapSeconds) return totalCapSeconds;
    }
    return total;
  }

  /// [durationLabel] over a raw seconds list — same one-home formatting rule.
  String durationLabelOf(Iterable<int> stepSeconds) {
    final total = totalSecondsOf(stepSeconds);
    final minutes = total ~/ 60;
    if (minutes >= 100) return '99:99';
    final seconds = total % 60;
    return '$minutes:${seconds.toString().padLeft(2, '0')}';
  }
}
