import 'dart:async';
import 'package:coffee_timer/core/recipe/recipe.dart';
import 'package:coffee_timer/recipe_details/widgets/circular_timer.dart';
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
  final RecipeEventBus _bus = RecipeHub.instance.bus;

  late final StreamSubscription<RecipeEvent> _sub;

  final stepName = ValueNotifier<String>('');
  final recipe = ValueNotifier<Recipe>(const Recipe.empty());
  final remaining = ValueNotifier<int>(0);
  final progress = ValueNotifier<double>(0);
  final started = ValueNotifier<bool>(false);

  @override
  void initState() {
    super.initState();

    _sub = _bus.stream.listen((event) {
      switch (event) {
        case RecipeModified():
          recipe.value = event.recipe;

        case RecipeSelect():
          recipe.value = event.recipe;

        case RecipeTimerTick():
          remaining.value = event.remainingSeconds;
          progress.value = event.progress;

        case RecipeTimerStepChanged():
          stepName.value = event.step.name;

        case RecipeTimerStartFromStep():
        case RecipeTimerStart():
          started.value = true;

        case RecipeTimerEnded():
        case RecipeTimerReset():
        case RecipeTimerPause():
          started.value = false;

        default:
          break;
      }
    });
  }

  @override
  void dispose() {
    unawaited(_sub.cancel());
    stepName.dispose();
    started.dispose();
    progress.dispose();
    remaining.dispose();
    recipe.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      onPopInvokedWithResult: (poped, result) {
        _bus
          ..fire(const RecipeTimerReset())
          ..fire(const RecipeDeselect());
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
                            _bus.fire(const RecipeTimerReset());
                          } else {
                            _bus.fire(const RecipeTimerStart());
                          }
                        },
                        child: LayoutBuilder(
                          builder: (context, constrains) {
                            return CircularTimerWidget(
                              controller: progress,
                              constraints: BoxConstraints(
                                minWidth: constrains.maxWidth,
                                minHeight: constrains.maxWidth,
                              ),
                              child: isStarted
                                  ? Column(
                                      children: [
                                        ValueListenableBuilder(
                                          valueListenable: stepName,
                                          builder: (context, name, _) {
                                            print('Step name build');
                                            return Text(name);
                                          },
                                        ),
                                        ValueListenableBuilder(
                                          valueListenable: remaining,
                                          builder: (context, seconds, _) {
                                            return TimerDigits(
                                              remainingSeconds: seconds,
                                            );
                                          },
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
                                              emoji: '☕',
                                              text: '${r.totalCoffeeGrams}g',
                                            ),
                                            const SizedBox(height: 4),
                                            _RecipeInfoText(
                                              emoji: '💧',
                                              text: '${r.totalWaterGrams}g',
                                            ),
                                            const SizedBox(height: 4),
                                            _RecipeInfoText(
                                              emoji: '⏱️',
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
                    child: ListView.builder(
                      itemCount: r.steps.length,
                      itemBuilder: (_, i) {
                        final step = r.steps[i];
                        return ListTile(
                          leading: Text('${i + 1}'),
                          title: Text(step.name),
                          trailing: Text('${step.durationSeconds}s'),
                          onLongPress: () => _bus.fire(
                            RecipeTimerStartFromStep(i),
                          ),
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

class TimerDigits extends StatelessWidget {
  const TimerDigits({
    super.key,
    required this.remainingSeconds,
  });

  final int remainingSeconds;

  @override
  Widget build(BuildContext context) {
    final textStyle = TextTheme.of(context).titleLarge;
    final minutes = remainingSeconds ~/ 60;
    final seconds = remainingSeconds % 60;

    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        _fixedDigit(
          minutes.toString().padLeft(2, '0'),
          textStyle,
        ),
        const SizedBox(width: 4),
        Text(
          ':',
          style: textStyle,
        ),
        const SizedBox(width: 4),
        _fixedDigit(
          seconds.toString().padLeft(2, '0'),
          textStyle,
        ),
      ],
    );
  }

  Widget _fixedDigit(
    String time,
    TextStyle? style,
  ) {
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
    required this.emoji,
    required this.text,
  });

  final String emoji;
  final String text;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
      children: [
        Text(emoji, style: const TextStyle(fontSize: 18)),
        const SizedBox(width: 4),
        Text(
          text,
          style: const TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
        ),
      ],
    );
  }
}
