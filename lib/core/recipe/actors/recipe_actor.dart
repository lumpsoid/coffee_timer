import 'dart:async';

import 'package:coffee_timer/core/audio_asset.dart';
import 'package:coffee_timer/core/di/hub.dart';
import 'package:coffee_timer/core/events/events.dart';
import 'package:coffee_timer/core/recipe/entities/entities.dart';
import 'package:coffee_timer/core/recipe/events/events.dart';

class RecipeActor extends Leaf {
  RecipeActor();

  final _recipes = <Recipe>[];
  Recipe? _currentRecipe;
  RecipeStep? _currentStep;

  @override
  Future<void> handleEvent(Event event) {
    switch (event) {
      case RecipesLoad _:
        if (_recipes.isEmpty) {
          emit(const RecipesLoadStorage());
        } else {
          emit(RecipesLoaded(_recipes));
        }
      case RecipesLoadedStorage(:final recipes):
        _recipes.addAll(recipes);
        emit(RecipesLoaded(recipes));

      case RecipeCreatedStorage(:final recipe):
        _recipes.add(event.recipe);
        emit(RecipesLoaded(_recipes));
        if (_currentRecipe != null) {
          emit(RecipeModified(recipe));
        }

      case RecipeUpdatedStorage(:final recipe):
        final oldRecipes = _recipes;
        final indexRecipe = oldRecipes.indexWhere(
          (recipe) => recipe.id == event.recipe.id,
        );
        // new recipe
        if (indexRecipe == -1) {
          _recipes.add(event.recipe);
        } else {
          // old one modified
          _recipes[indexRecipe] = event.recipe;
        }
        emit(RecipesLoaded(_recipes));
        if (_currentRecipe != null) {
          emit(RecipeModified(recipe));
        }

      case RecipeDeletedStorage(:final id):
        _recipes.removeWhere((r) => r.id == id);
        emit(RecipesLoaded(_recipes));
        if (_currentRecipe != null && _currentRecipe!.id == id) {
          emit(const RecipeModified(Recipe.empty));
        }

      case RecipeSelect _:
        _currentRecipe = event.recipe;
        emit(const AudioLoadAsset(AudioAsset.timer));

      case RecipeDeselect _:
        _currentRecipe = null;
        _resetTimerState();
        emit(const WakelockDisable());
        emit(const AudioUnloadAssetLatest());

      case RecipeCreate(:final recipe):
        emit(RecipeCreateStorage(recipe));
      case RecipeUpdate _:
        _currentRecipe = event.recipe;
        emit(RecipeUpdateStorage(event.recipe));
      case RecipeDelete(:final id):
        emit(RecipeDeleteStorage(id));

      case RecipeTimerStart():
        _startTimer(initialStepIndex: 0);
      case RecipeTimerStartFromStep():
        _startTimer(initialStepIndex: event.stepIndex);

      case RecipeTimerReset():
        _resetTimer();
      default:
        break;
    }
    return Future.value();
  }

  Timer? _secondTimer;
  int _currentStepIndex = 0;
  int _stepRemainingSeconds = 0;
  int _totalRemainingSeconds = 0;

  void _startTimer({required int initialStepIndex}) {
    _resetTimerState();

    if (_currentRecipe == null || _currentRecipe!.steps.isEmpty) return;

    _currentStepIndex = initialStepIndex;
    _currentStep = _currentRecipe!.steps[_currentStepIndex];
    _stepRemainingSeconds = _currentStep?.durationSeconds ?? 0;
    _totalRemainingSeconds = _currentRecipe!.totalDurationSeconds;

    // Fire initial UI tick
    final (minutes, seconds) = _formatRemainingSeconds(_stepRemainingSeconds);
    emit(const WakelockEnable());
    emit(
      RecipeTimerStarted(
        index: _currentStepIndex,
        step: _currentStep!,
      ),
    );
    emit(RecipeTimerTick(minutes: minutes, seconds: seconds));

    // Tick every second
    _secondTimer = Timer.periodic(const Duration(seconds: 1), (timer) {
      if (_currentStep == null) {
        _secondTimer?.cancel();
        return;
      }

      // Decrement time
      _stepRemainingSeconds--;
      _totalRemainingSeconds = (_totalRemainingSeconds > 0)
          ? _totalRemainingSeconds - 1
          : 0;

      // Fire tick
      final (minutes, seconds) = _formatRemainingSeconds(_stepRemainingSeconds);
      emit(RecipeTimerTick(minutes: minutes, seconds: seconds));

      // Check if step is complete
      if (_stepRemainingSeconds <= 0) {
        _currentStepIndex++;

        if (_currentStepIndex >= _currentRecipe!.steps.length) {
          // All steps done
          _resetTimer();
          emit(const Vibrate());
          return;
        }

        // Move to next step
        _currentStep = _currentRecipe!.steps[_currentStepIndex];
        _stepRemainingSeconds = _currentStep!.durationSeconds;

        final (newMinutes, newSeconds) = _formatRemainingSeconds(
          _stepRemainingSeconds,
        );

        emit(const Vibrate());
        emit(const AudioPlayTimerSound());
        emit(RecipeTimerTick(minutes: newMinutes, seconds: newSeconds));
        emit(
          RecipeTimerStepChanged(
            index: _currentStepIndex,
            step: _currentStep!,
          ),
        );
      }
    });
  }

  (String, String) _formatRemainingSeconds(int remainingSeconds) {
    if (remainingSeconds >= 6000) {
      return ('99', '60'); // fallback cap
    }

    final minutes = (remainingSeconds ~/ 60).toString().padLeft(2, '0');
    final seconds = (remainingSeconds % 60).toString().padLeft(2, '0');

    return (minutes, seconds);
  }

  void _resetTimerState() {
    if (_secondTimer == null) return;

    _secondTimer?.cancel();
    _secondTimer = null;
    _currentStep = null;
    _currentStepIndex = 0;
    _stepRemainingSeconds = 0;
    _totalRemainingSeconds = 0;
  }

  void _resetTimer() {
    _resetTimerState();
    emit(const WakelockDisable());
    emit(const RecipeTimerEnded());
  }
}
