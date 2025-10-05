class RecipeStep {
  const RecipeStep({
    required this.id,
    required this.name,
    required this.durationSeconds,
  });

  const RecipeStep.empty() : id = '', name = '', durationSeconds = 0;

  final String id;
  final String name;
  final int durationSeconds;
}
