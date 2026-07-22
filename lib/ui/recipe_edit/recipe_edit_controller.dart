import 'package:coffee_timer/application/recipe/delete_recipe.dart';
import 'package:coffee_timer/application/recipe/recipe_ids.dart';
import 'package:coffee_timer/application/recipe/save_recipe.dart';
import 'package:coffee_timer/domain/recipe/brew_metrics.dart';
import 'package:coffee_timer/domain/recipe/recipe.dart';
import 'package:coffee_timer/domain/recipe/recipe_name.dart';
import 'package:coffee_timer/domain/recipe/recipe_step.dart';
import 'package:coffee_timer/domain/recipe/step_duration.dart';
import 'package:coffee_timer/domain/recipe/step_id.dart';
import 'package:coffee_timer/ui/navigation/route_args.dart';
import 'package:flutter/foundation.dart';

/// One editable step row. Free text lives here as raw strings so the form can
/// hold in-progress input (an empty duration field, a half-typed name) that a
/// domain value object would reject; the row is turned into a valid
/// [RecipeStep] only at save. [id] is null for a row the user just added — it
/// receives a fresh id then.
class StepRow {
  StepRow({this.id, this.name = '', this.duration = '0'});

  final StepId? id;
  String name;
  String duration;
}

/// Page-scoped controller for the recipe editor (create or edit). It holds the
/// raw form model, assembles a valid domain [Recipe] at save, and persists it
/// through the use cases. It holds no business rules: name/duration validity
/// come from the domain value objects, the blank-step-name fallback from
/// [RecipeStep.labelOrDefault], and the live total from [BrewMetrics].
class RecipeEditController extends ChangeNotifier {
  RecipeEditController({
    required SaveRecipe save,
    required DeleteRecipe delete,
    required RecipeIds ids,
    required RecipeEditArgs args,
  }) : _save = save,
       _delete = delete,
       _ids = ids,
       _initial = args.initial {
    final initial = _initial;
    if (initial == null) {
      name = '';
      description = '';
      rows = [StepRow()];
    } else {
      name = initial.name.value;
      description = initial.description;
      rows = [
        for (final s in initial.steps)
          StepRow(
            id: s.id,
            name: s.name,
            duration: s.duration.seconds.toString(),
          ),
      ];
    }
  }

  static const _metrics = BrewMetrics();

  final SaveRecipe _save;
  final DeleteRecipe _delete;
  final RecipeIds _ids;
  final Recipe? _initial;

  late String name;
  late String description;
  late List<StepRow> rows;

  String? _error;
  String? get error => _error;

  /// True when editing an existing recipe — the delete action is offered only
  /// then, and the id is preserved on save.
  bool get isEditing => _initial != null;

  /// Live `M:SS` total for the durations currently typed, clamped by the
  /// domain.
  String get totalDurationLabel =>
      _metrics.durationLabelOf(rows.map((r) => _rowDuration(r).seconds));

  StepDuration _rowDuration(StepRow row) =>
      StepDuration.clamped(int.tryParse(row.duration.trim()) ?? 0);

  void setStepName(int index, String value) => rows[index].name = value;

  /// A duration change rebuilds the total-duration label, so this notifies.
  void setStepDuration(int index, String value) {
    rows[index].duration = value;
    notifyListeners();
  }

  void addStep([String name = '']) {
    rows.add(StepRow(name: name));
    notifyListeners();
  }

  void removeStepAt(int index) {
    rows.removeAt(index);
    if (rows.isEmpty) rows.add(StepRow());
    notifyListeners();
  }

  /// Assembles a valid [Recipe] from the current form and persists it. Returns
  /// the saved recipe, or null if the name is blank or a save failed — [error]
  /// then holds a message to show.
  Future<Recipe?> save() async {
    final validName = RecipeName.tryParse(name);
    if (validName == null) {
      _error = 'Enter a recipe name';
      notifyListeners();
      return null;
    }

    var recipe = Recipe.blank(
      id: _initial?.id ?? _ids.newRecipeId(),
      name: validName,
    ).describedAs(description.trim());

    for (var i = 0; i < rows.length; i++) {
      final row = rows[i];
      recipe = recipe.withStepAdded(
        RecipeStep(
          id: row.id ?? _ids.newStepId(),
          name: RecipeStep.labelOrDefault(row.name, i + 1),
          duration: _rowDuration(row),
        ),
      );
    }

    try {
      final saved = await _save(recipe);
      _error = null;
      return saved;
    } on Object catch (e) {
      _error = 'Failed to save recipe: $e';
      notifyListeners();
      return null;
    }
  }

  /// Deletes the recipe being edited. Returns true on success; a no-op that
  /// returns false in create mode (there is nothing to delete).
  Future<bool> deleteExisting() async {
    final initial = _initial;
    if (initial == null) return false;
    try {
      await _delete(initial.id);
      _error = null;
      return true;
    } on Object catch (e) {
      _error = 'Failed to delete recipe: $e';
      notifyListeners();
      return false;
    }
  }
}
