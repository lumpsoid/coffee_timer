import 'dart:async';

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
  late final StreamSubscription<RecipeEvent> _subscription;

  final _recipes = <Recipe>[];
  Recipe? _currentRecipe;
  Timer? _timer;
  Step? _currentStep;
  int _remainingSeconds = 0;
  int _total = 0;

  void _handleEvent(RecipeEvent event) {
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

      case RecipeSelect _:
        _currentRecipe = event.recipe;
      case RecipeDeselect _:
        _currentRecipe = null;
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

  int _currentStepIndex = 0;

  int _stepRemainingSeconds = 0;
  int _totalRemainingSeconds = 0; // optional: overall remaining

  void _startTimer({
    required int initialStepIndex,
  }) {
    if (_timer?.isActive ?? false) {
      _resetTimerState();
    }
    if (_currentRecipe == null || _currentRecipe!.steps.isEmpty) return;

    _currentStepIndex = initialStepIndex;
    _currentStep = _currentRecipe!.steps[_currentStepIndex];
    _stepRemainingSeconds = _currentStep?.durationSeconds ?? 0;
    _totalRemainingSeconds = _currentRecipe!.totalDurationSeconds;

    // emit initial tick for first step
    final initialStepDuration = _currentStep?.durationSeconds ?? 1;
    final initialStepProgress = initialStepDuration > 0
        ? ((_currentStep?.durationSeconds ?? 0) - _stepRemainingSeconds) /
              initialStepDuration
        : 0.0;
    _bus
      ..fire(
        RecipeTimerTick(
          remainingSeconds: _stepRemainingSeconds,
          progress: initialStepProgress.clamp(0.0, 1.0),
        ),
      )
      ..fire(RecipeTimerStepChanged(_currentStep!));

    _timer = Timer.periodic(const Duration(seconds: 1), (t) {
      if (_currentStep == null) {
        // no more steps
        _resetTimer();
        _bus
          ..fire(const RecipeTimerTick(remainingSeconds: 0, progress: 0))
          ..fire(const RecipeTimerEnded());
        return;
      }

      if (_stepRemainingSeconds <= 1) {
        // finish current step
        _bus.fire(
          const RecipeTimerTick(remainingSeconds: 0, progress: 1),
        );

        // advance to next step
        _currentStepIndex++;
        if (_currentStepIndex >= _currentRecipe!.steps.length) {
          // no more steps
          _resetTimer();
          _currentStep = null;
          _bus
            ..fire(
              const RecipeTimerStepChanged(
                Step(id: '', name: 'Recipe ended', durationSeconds: 0),
              ),
            )
            ..fire(const RecipeTimerEnded());
          return;
        } else {
          _currentStep = _currentRecipe?.steps[_currentStepIndex];
          _bus.fire(RecipeTimerStepChanged(_currentStep!));
          // set up next step
          _stepRemainingSeconds = _currentStep!.durationSeconds;
          // emit tick for start of new step
          final dur = _currentStep!.durationSeconds;
          final prog = dur > 0 ? (dur - _stepRemainingSeconds) / dur : 0.0;
          _bus.fire(
            RecipeTimerTick(
              remainingSeconds: _stepRemainingSeconds,
              progress: prog.clamp(0.0, 1.0),
            ),
          );
          // continue ticking; do not decrement this second (we already emitted start state)
          return;
        }
      } else {
        // normal countdown within current step
        _stepRemainingSeconds--;
        _totalRemainingSeconds = (_totalRemainingSeconds > 0)
            ? _totalRemainingSeconds - 1
            : 0;

        final stepDuration = _currentStep!.durationSeconds;
        final rawStepProgress = stepDuration > 0
            ? (stepDuration - _stepRemainingSeconds) / stepDuration
            : 0.0;
        final stepProgress = rawStepProgress.clamp(0.0, 1.0);

        _bus.fire(
          RecipeTimerTick(
            remainingSeconds: _stepRemainingSeconds,
            progress: stepProgress,
          ),
        );
      }
    });
  }

  void _resetTimerState() {
    _timer?.cancel();
    _timer = null;
    _currentStep = null;
    _currentStepIndex = 0;
    _stepRemainingSeconds = 0;
    _totalRemainingSeconds = 0;
    _remainingSeconds = _currentRecipe?.totalDurationSeconds ?? 0;
  }

  void _resetTimer() {
    _resetTimerState();
    _bus.fire(const RecipeTimerTick(remainingSeconds: 0, progress: 0));
  }

  void dispose() {
    unawaited(
      _subscription.cancel(),
    );
  }
}
