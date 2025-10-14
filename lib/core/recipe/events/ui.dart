import 'package:coffee_timer/core/recipe/entities/entities.dart';
import 'package:coffee_timer/core/recipe/events/recipe_event.dart';

class RecipesTest extends RecipeEvent {
  const RecipesTest();
}

class RecipesLoad extends RecipeEvent {
  const RecipesLoad();
}

class RecipesLoaded extends RecipeEvent {
  const RecipesLoaded(this.recipes);
  final List<Recipe> recipes;
}

class RecipeSelect extends RecipeEvent {
  const RecipeSelect(this.recipe);
  final Recipe recipe;
}

class RecipeDeselect extends RecipeEvent {
  const RecipeDeselect();
}

class RecipeCreate extends RecipeEvent {
  const RecipeCreate(this.recipe);
  final Recipe recipe;
}

class RecipeUpdate extends RecipeEvent {
  const RecipeUpdate(this.recipe);
  final Recipe recipe;
}

class RecipeDelete extends RecipeEvent {
  const RecipeDelete({required this.id});
  final String id;
}

class RecipeTimerStepChanged extends RecipeEvent {
  const RecipeTimerStepChanged({
    required this.index,
    required this.step,
  });

  final int? index;
  final RecipeStep step;
}

class RecipeTimerStartFromStep extends RecipeEvent {
  const RecipeTimerStartFromStep(this.stepIndex);
  final int stepIndex;
}

class RecipeTimerStart extends RecipeEvent {
  const RecipeTimerStart();
}

class RecipeTimerStarted extends RecipeEvent {
  const RecipeTimerStarted({
    required this.index,
    required this.step,
  });

  final int? index;
  final RecipeStep step;
}

class RecipeTimerTick extends RecipeEvent {
  const RecipeTimerTick({
    required this.minutes,
    required this.seconds,
  });

  const RecipeTimerTick.empty() : minutes = '', seconds = '';

  final String minutes;
  final String seconds;
}

class RecipeTimerEnded extends RecipeEvent {
  const RecipeTimerEnded();
}

class RecipeTimerPause extends RecipeEvent {
  const RecipeTimerPause();
}

class RecipeTimerResume extends RecipeEvent {
  const RecipeTimerResume();
}

class RecipeTimerReset extends RecipeEvent {
  const RecipeTimerReset();
}

class RecipeModified extends RecipeEvent {
  const RecipeModified(this.recipe);
  final Recipe recipe;
}

class RecipeDeleted extends RecipeEvent {
  const RecipeDeleted(this.id);
  final String id;
}
