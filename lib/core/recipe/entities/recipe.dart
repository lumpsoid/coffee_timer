import 'package:coffee_timer/core/recipe/entities/entities.dart';

class Recipe {
  const Recipe({
    required this.id,
    required this.name,
    required this.description,
    required this.steps,
  });
  const Recipe.empty() : id = '', name = '', description = '', steps = const [];

  final String id;
  final String name;
  final String description;
  final List<Step> steps;

  static String totalDurationFormatted(int totalSeconds) {
    final minutes = totalSeconds ~/ 60;
    if (minutes >= 100) {
      return '99:99';
    }
    final seconds = totalSeconds % 60;
    return '$minutes:${seconds.toString().padLeft(2, '0')}';
  }

  bool get isEmpty =>
      id.isEmpty && name.isEmpty && description.isEmpty && steps.isEmpty;
  bool get isNotEmpty => !isEmpty;

  /// Assuming coffee grams per step is 10g (example)
  /// You can extend Step to include coffee and water grams if needed
  int get totalCoffeeGrams => steps.length * 10;

  /// Total duration formatted as MM:SS
  String get totalDurationString {
    return totalDurationFormatted(totalDurationSeconds);
  }

  /// Total duration in seconds (sum of all steps)
  int get totalDurationSeconds {
    var total = 0;
    for (final step in steps) {
      total += step.durationSeconds;
      if (total >= 6000) {
        return 6000;
      }
    }
    return total;
  }

  /// Assuming water grams per step is 50g (example)
  int get totalWaterGrams => steps.length * 50;
}
