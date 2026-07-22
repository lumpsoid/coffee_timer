import 'package:coffee_timer/application/recipe/recipe_ports.dart';
import 'package:coffee_timer/domain/recipe/recipe.dart';
import 'package:coffee_timer/domain/recipe/recipe_id.dart';
import 'package:flutter/foundation.dart';

/// Page-scoped controller for the recipe list. It loads recipes through the
/// read port and deletes through the write port, exposing plain state for the
/// page to render. No business decisions live here — grams and durations come
/// from `BrewMetrics`; this only orchestrates fetch/delete and notifies.
class RecipeListController extends ChangeNotifier {
  RecipeListController({
    required RecipeReadRepository read,
    required RecipeWriteRepository write,
  }) : _read = read,
       _write = write;

  final RecipeReadRepository _read;
  final RecipeWriteRepository _write;

  List<Recipe> _recipes = const [];
  List<Recipe> get recipes => _recipes;

  bool _isLoading = false;
  bool get isLoading => _isLoading;

  String? _error;
  String? get error => _error;

  /// Loads all recipes. Also used to refresh after returning from the (still
  /// legacy) detail/edit pages, which persist through the event bus.
  Future<void> load() async {
    _isLoading = true;
    notifyListeners();
    try {
      _recipes = await _read.all();
      _error = null;
    } on Object catch (e) {
      _error = 'Failed to load recipes: $e';
    } finally {
      _isLoading = false;
      notifyListeners();
    }
  }

  /// Deletes [id], then drops it from the in-memory list on success.
  Future<void> delete(RecipeId id) async {
    try {
      await _write.delete(id);
      _recipes = _recipes.where((r) => r.id != id).toList();
      _error = null;
    } on Object catch (e) {
      _error = 'Failed to delete recipe: $e';
    }
    notifyListeners();
  }
}
