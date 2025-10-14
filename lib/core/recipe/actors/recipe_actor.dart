import 'dart:async';

import 'package:coffee_timer/core/audio_asset.dart';
import 'package:coffee_timer/core/events/events.dart';
import 'package:coffee_timer/core/recipe/buses/buses.dart';
import 'package:coffee_timer/core/recipe/entities/entities.dart';
import 'package:coffee_timer/core/recipe/events/events.dart';

class RecipeActor {
  RecipeActor({
    required RecipeEventBus bus,
  }) : _bus = bus {
    _subscription = _bus.stream.listen(_handleEvent);
  }

  final RecipeEventBus _bus;
  late final StreamSubscription<Event> _subscription;

  final _recipes = <Recipe>[];
  Recipe? _currentRecipe;
  RecipeStep? _currentStep;

  void _handleEvent(Event event) {
    switch (event) {
      case RecipesLoad _:
        if (_recipes.isEmpty) {
          _bus.fire(const RecipesLoadStorage());
        } else {
          _bus.fire(RecipesLoaded(_recipes));
        }
      case RecipesLoadedStorage(:final recipes):
        _recipes.addAll(recipes);
        _bus.fire(RecipesLoaded(recipes));

      case RecipeCreatedStorage(:final recipe):
        _recipes.add(event.recipe);
        _bus.fire(RecipesLoaded(_recipes));
        if (_currentRecipe != null) {
          _bus.fire(RecipeModified(recipe));
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
        _bus.fire(RecipesLoaded(_recipes));
        if (_currentRecipe != null) {
          _bus.fire(RecipeModified(recipe));
        }

      case RecipeDeletedStorage(:final id):
        _recipes.removeWhere((r) => r.id == id);
        _bus.fire(RecipesLoaded(_recipes));
        if (_currentRecipe != null && _currentRecipe!.id == id) {
          _bus.fire(const RecipeModified(Recipe.empty));
        }

      case RecipeSelect _:
        _currentRecipe = event.recipe;
        _bus.fire(const AudioLoadAsset(AudioAsset.timer));

      case RecipeDeselect _:
        _currentRecipe = null;
        _resetTimerState();
        _bus.fire(const WakelockDisable());
        _bus.fire(const AudioUnloadAssetLatest());

      case RecipeCreate(:final recipe):
        _bus.fire(RecipeCreateStorage(recipe));
      case RecipeUpdate _:
        _currentRecipe = event.recipe;
        _bus.fire(RecipeUpdateStorage(event.recipe));
      case RecipeDelete(:final id):
        _bus.fire(RecipeDeleteStorage(id));

      case RecipeTimerStart():
        _startTimer(initialStepIndex: 0);
      case RecipeTimerStartFromStep():
        _startTimer(initialStepIndex: event.stepIndex);

      case RecipeTimerReset():
        _resetTimer();
      default:
        break;
    }
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
    _bus
      ..fire(const WakelockEnable())
      ..fire(
        RecipeTimerStarted(
          index: _currentStepIndex,
          step: _currentStep!,
        ),
      )
      ..fire(RecipeTimerTick(minutes: minutes, seconds: seconds));

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
      _bus.fire(RecipeTimerTick(minutes: minutes, seconds: seconds));

      // Check if step is complete
      if (_stepRemainingSeconds <= 0) {
        _currentStepIndex++;

        if (_currentStepIndex >= _currentRecipe!.steps.length) {
          // All steps done
          _resetTimer();
          _bus.fire(const Vibrate());
          return;
        }

        // Move to next step
        _currentStep = _currentRecipe!.steps[_currentStepIndex];
        _stepRemainingSeconds = _currentStep!.durationSeconds;

        final (newMinutes, newSeconds) = _formatRemainingSeconds(
          _stepRemainingSeconds,
        );
        _bus
          ..fire(const Vibrate())
          ..fire(const AudioPlayTimerSound())
          ..fire(RecipeTimerTick(minutes: newMinutes, seconds: newSeconds))
          ..fire(
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
    _bus
      ..fire(const WakelockDisable())
      ..fire(const RecipeTimerEnded());
  }

  void dispose() {
    unawaited(
      _subscription.cancel(),
    );
  }
}
