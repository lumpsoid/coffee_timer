import 'package:coffee_timer/core/recipe/entities/entities.dart';
import 'package:coffee_timer/core/recipe/events/recipe_event.dart';

class RecipesLoadStorage extends RecipeEvent {
  const RecipesLoadStorage();
}

class RecipesLoadedStorage extends RecipeEvent {
  const RecipesLoadedStorage(this.recipes);
  final List<Recipe> recipes;
}

class RecipeCreateStorage extends RecipeEvent {
  const RecipeCreateStorage(this.recipe);
  final Recipe recipe;
}

class RecipeUpdateStorage extends RecipeEvent {
  const RecipeUpdateStorage(this.recipe);
  final Recipe recipe;
}

class RecipeDeleteStorage extends RecipeEvent {
  const RecipeDeleteStorage(this.id);
  final String id;
}
