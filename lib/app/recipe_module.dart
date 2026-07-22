import 'package:coffee_timer/application/recipe/delete_recipe.dart';
import 'package:coffee_timer/application/recipe/recipe_ids.dart';
import 'package:coffee_timer/application/recipe/recipe_ports.dart';
import 'package:coffee_timer/application/recipe/save_recipe.dart';
import 'package:coffee_timer/ui/navigation/route_args.dart';
import 'package:coffee_timer/ui/recipe_edit/recipe_edit_controller.dart';
import 'package:coffee_timer/ui/recipe_list/recipe_list_controller.dart';

/// Shared recipe infrastructure plus factories for page-scoped controllers.
/// Registered once by `AppRoot` into `Cells`.
///
/// Depends on the narrow ports, not the concrete SQLite repository, so a test
/// can wire it to an in-memory fake. Grows a `newDetailsController` as the
/// details/timer feature migrates off the event bus (see `docs/pivot-spec.md`).
class RecipeModule {
  RecipeModule({required this.read, required this.write, required this.ids});

  final RecipeReadRepository read;
  final RecipeWriteRepository write;
  final RecipeIds ids;

  RecipeListController newListController() =>
      RecipeListController(read: read, write: write);

  RecipeEditController newEditController(RecipeEditArgs args) =>
      RecipeEditController(
        save: SaveRecipe(write),
        delete: DeleteRecipe(write),
        ids: ids,
        args: args,
      );
}
