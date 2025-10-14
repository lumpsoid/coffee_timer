import 'dart:async';

import 'package:coffee_timer/core/events/events.dart';
import 'package:coffee_timer/core/recipe/recipe.dart';
import 'package:coffee_timer/recipe_details/widgets/widgets.dart';
import 'package:coffee_timer/recipe_edit/ui/ui.dart';
import 'package:flutter/material.dart';

class RecipeDetailPage extends StatefulWidget {
  const RecipeDetailPage({super.key});

  static MaterialPageRoute<void> route() {
    return MaterialPageRoute(
      builder: (context) => const RecipeDetailPage(),
    );
  }

  @override
  State<RecipeDetailPage> createState() => _RecipeDetailPageState();
}

class _RecipeDetailPageState extends State<RecipeDetailPage> {
  late final StreamSubscription<Event> _sub;

  final stepNotifier = ValueNotifier<RecipeStep>(const RecipeStep.empty());
  final stepDuration = ValueNotifier<Duration?>(null);
  final currentStepIndex = ValueNotifier<int>(0);
  late final runAndIndex = Listenable.merge([started, currentStepIndex]);
  final recipe = ValueNotifier<Recipe>(Recipe.empty);
  final timerMinutes = ValueNotifier<String>('');
  final timerSeconds = ValueNotifier<String>('');
  final started = ValueNotifier<bool>(false);

  @override
  void initState() {
    super.initState();

    _sub = context.recipeEventBus.stream.listen(_handleEvent);
  }

  void _handleEvent(Event event) {
    switch (event) {
      case RecipeModified():
        recipe.value = event.recipe;

      case RecipeSelect():
        recipe.value = event.recipe;

      case RecipeTimerTick():
        timerMinutes.value = event.minutes;
        timerSeconds.value = event.seconds;

      case RecipeTimerStarted(:final index, :final step):
      case RecipeTimerStepChanged(:final index, :final step):
        stepNotifier.value = step;
        currentStepIndex.value = index ?? 0;
        stepDuration.value = step.durationSeconds == 0
            ? null
            : Duration(seconds: step.durationSeconds);

      case RecipeTimerStartFromStep():
      case RecipeTimerStart():
        started.value = true;

      case RecipeTimerEnded():
        started.value = false;
        stepDuration.value = null;

        stepNotifier.value = const RecipeStep.empty();
        currentStepIndex.value = 0;
        timerMinutes.value = '';
        timerSeconds.value = '';

      case RecipeTimerReset():
      case RecipeTimerPause():
        started.value = false;
        stepDuration.value = null;

      default:
        return;
    }
  }

  @override
  void dispose() {
    unawaited(_sub.cancel());
    started.dispose();
    recipe.dispose();
    timerSeconds.dispose();
    timerMinutes.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      onPopInvokedWithResult: (poped, result) {
        context.recipeEventBus.fire(const RecipeDeselect());
      },
      child: Scaffold(
        appBar: AppBar(
          title: ValueListenableBuilder(
            valueListenable: recipe,
            builder: (context, r, _) {
              return Text(r.name);
            },
          ),
          actions: [
            IconButton(
              icon: const Icon(Icons.edit),
              onPressed: () => unawaited(
                Navigator.push(
                  context,
                  RecipeEditPage.route(
                    initial: recipe.value,
                  ),
                ),
              ),
            ),
          ],
        ),
        body: ValueListenableBuilder(
          valueListenable: recipe,
          builder: (context, r, _) {
            if (r.isEmpty) {
              return const Center(child: Text('Empty recipe'));
            }

            return Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    r.description,
                    style: const TextStyle(fontSize: 18),
                    maxLines: 1,
                    overflow: TextOverflow.ellipsis,
                  ),
                  const SizedBox(height: 16),
                  ValueListenableBuilder(
                    valueListenable: started,
                    builder: (context, isStarted, _) {
                      return GestureDetector(
                        onTap: () {
                          if (isStarted) {
                            context.recipeEventBus.fire(
                              const RecipeTimerReset(),
                            );
                          } else {
                            context.recipeEventBus.fire(
                              const RecipeTimerStart(),
                            );
                          }
                        },
                        child: LayoutBuilder(
                          builder: (context, constrains) {
                            final textStyle = TextTheme.of(context).titleLarge;
                            return CircularTimer(
                              duration: stepDuration,
                              progressColor: ColorScheme.of(context).primary,
                              constraints: BoxConstraints(
                                minWidth: constrains.maxWidth,
                                minHeight: constrains.maxWidth,
                              ),
                              child: isStarted
                                  ? Column(
                                      children: [
                                        ValueListenableBuilder(
                                          valueListenable: stepNotifier,
                                          builder: (context, s, _) {
                                            return Text(s.name);
                                          },
                                        ),
                                        Row(
                                          mainAxisAlignment:
                                              MainAxisAlignment.center,
                                          children: [
                                            ValueListenableBuilder(
                                              valueListenable: timerMinutes,
                                              builder: (context, minutes, _) {
                                                return FixedDigit(
                                                  time: minutes,
                                                  style: textStyle,
                                                );
                                              },
                                            ),
                                            const SizedBox(width: 4),
                                            Text(
                                              ':',
                                              style: textStyle,
                                            ),
                                            const SizedBox(width: 4),
                                            ValueListenableBuilder(
                                              valueListenable: timerSeconds,
                                              builder: (context, seconds, _) {
                                                return FixedDigit(
                                                  time: seconds,
                                                  style: textStyle,
                                                );
                                              },
                                            ),
                                          ],
                                        ),
                                      ],
                                    )
                                  : ValueListenableBuilder(
                                      valueListenable: recipe,
                                      builder: (context, r, _) {
                                        return Row(
                                          mainAxisAlignment:
                                              MainAxisAlignment.spaceEvenly,
                                          children: [
                                            _RecipeInfoText(
                                              icon: const Icon(Icons.grain),
                                              text: '${r.totalCoffeeGrams}g',
                                            ),
                                            const SizedBox(height: 4),
                                            _RecipeInfoText(
                                              icon: const Icon(
                                                Icons.water_drop_outlined,
                                              ),
                                              text: '${r.totalWaterGrams}g',
                                            ),
                                            const SizedBox(height: 4),
                                            _RecipeInfoText(
                                              icon: const Icon(
                                                Icons.timer_outlined,
                                              ),
                                              text: r.totalDurationString,
                                            ),
                                          ],
                                        );
                                      },
                                    ),
                            );
                          },
                        ),
                      );
                    },
                  ),
                  const SizedBox(height: 20),
                  const Text(
                    'Steps:',
                    style: TextStyle(fontWeight: FontWeight.bold),
                  ),
                  const SizedBox(height: 8),
                  Expanded(
                    child: ListenableBuilder(
                      listenable: runAndIndex,
                      builder: (context, _) {
                        final isTicking = started.value;

                        return ListView.builder(
                          itemCount: r.steps.length,
                          itemBuilder: (_, i) {
                            final step = r.steps[i];

                            return StepListTile(
                              index: i,
                              currentStepIndex: currentStepIndex.value,
                              isTicking: isTicking,
                              stepName: step.name,
                              durationSeconds: step.durationSeconds,
                              onLongPress: () => context.recipeEventBus.fire(
                                RecipeTimerStartFromStep(i),
                              ),
                            );
                          },
                        );
                      },
                    ),
                  ),
                ],
              ),
            );
          },
        ),
      ),
    );
  }
}

class FixedDigit extends StatelessWidget {
  const FixedDigit({
    required this.time,
    required this.style,
    super.key,
  });

  final String time;
  final TextStyle? style;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: time.split('').map((char) {
        return Container(
          width: 24,
          height: 36,
          alignment: Alignment.center,
          margin: const EdgeInsets.symmetric(horizontal: 1),
          child: Text(
            char,
            style: style,
          ),
        );
      }).toList(),
    );
  }
}

class _RecipeInfoText extends StatelessWidget {
  const _RecipeInfoText({
    required this.icon,
    required this.text,
  });

  final Icon icon;
  final String text;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
      children: [
        icon,
        const SizedBox(width: 4),
        Text(
          text,
          style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
      ],
    );
  }
}
