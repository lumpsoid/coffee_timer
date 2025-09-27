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

  bool get isEmpty =>
      id.isEmpty && name.isEmpty && description.isEmpty && steps.isEmpty;

  /// Assuming coffee grams per step is 10g (example)
  /// You can extend Step to include coffee and water grams if needed
  int get totalCoffeeGrams => steps.length * 10;

  /// Total duration formatted as MM:SS
  String get totalDurationFormatted {
    final minutes = totalDurationSeconds ~/ 60;
    final seconds = totalDurationSeconds % 60;
    return '$minutes:${seconds.toString().padLeft(2, '0')}';
  }

  /// Total duration in seconds (sum of all steps)
  int get totalDurationSeconds =>
      steps.fold(0, (sum, step) => sum + step.durationSeconds);

  /// Assuming water grams per step is 50g (example)
  int get totalWaterGrams => steps.length * 50;
}
