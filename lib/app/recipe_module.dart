import 'package:coffee_timer/infrastructure/recipe/sqlite_recipe_repository.dart';
import 'package:coffee_timer/ui/recipe_list/recipe_list_controller.dart';

/// Shared recipe infrastructure plus factories for page-scoped controllers.
/// Registered once by `AppRoot` into `Cells`.
///
/// Grows a `newDetailsController` / `newEditController` as those features
/// migrate off the event bus (see `docs/pivot-spec.md`).
class RecipeModule {
  RecipeModule({required this.repository});

  final SqliteRecipeRepository repository;

  RecipeListController newListController() =>
      RecipeListController(read: repository, write: repository);
}
