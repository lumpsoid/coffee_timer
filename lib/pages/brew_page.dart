import 'package:coffee_timer/holders/brew_holder.dart';
import 'package:coffee_timer/holders/editor_holder.dart';
import 'package:coffee_timer/holders/recipe_list_holder.dart';
import 'package:coffee_timer/pages/editor_page.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/circular_timer.dart';
import 'package:coffee_timer/widgets/fixed_digit.dart';
import 'package:coffee_timer/widgets/step_list_tile.dart';
import 'package:flutter/material.dart';

/// The brew page: the open recipe, its timer ring and its steps. Once
/// subscribed it opens [recipeId] ([OpenBrew]) and shows the brew states
/// that follow. Tapping the ring starts or resets the brew; long-pressing a
/// step starts from it. Leaving the page tells Rust ([LeaveBrew]).
class BrewPage extends StatefulWidget {
  const BrewPage({
    required this.recipeId,
    required this.brew,
    required this.editor,
    required this.recipes,
    super.key,
  });

  final String recipeId;
  final BrewHolder brew;
  final EditorHolder editor;
  final RecipeListHolder recipes;

  static MaterialPageRoute<void> route(
    String recipeId, {
    required BrewHolder brew,
    required EditorHolder editor,
    required RecipeListHolder recipes,
  }) => MaterialPageRoute(
    builder: (_) => BrewPage(
      recipeId: recipeId,
      brew: brew,
      editor: editor,
      recipes: recipes,
    ),
  );

  @override
  State<BrewPage> createState() => _BrewPageState();
}

class _BrewPageState extends State<BrewPage> {
  /// The latest brew state since this page opened; null until Rust answers
  /// [OpenBrew], so a state left from before is never shown.
  BrewUpdate? _state;

  void _start(int from) => StartBrew(from: from).sendSignalToRust();

  @override
  void initState() {
    super.initState();
    widget.brew.addListener(_onUpdate);
    OpenBrew(recipeId: widget.recipeId).sendSignalToRust();
  }

  @override
  void dispose() {
    widget.brew.removeListener(_onUpdate);
    super.dispose();
  }

  void _onUpdate() => setState(() => _state = widget.brew.state);

  /// Stops following Rust, so the page keeps its last state on screen while
  /// it animates away, then tells Rust the page closed.
  void _leave() {
    widget.brew.removeListener(_onUpdate);
    const LeaveBrew().sendSignalToRust();
  }

  void _edit(String recipeId) {
    Navigator.of(context).push(
      EditorPage.edit(
        recipeId,
        editor: widget.editor,
        recipes: widget.recipes,
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      onPopInvokedWithResult: (_, _) => _leave(),
      child: switch (_state) {
        BrewUpdateIdle(:final recipe) => _page(recipe, null),
        BrewUpdateRunning(:final recipe, :final timer) => _page(recipe, timer),
        BrewUpdateEmpty(:final label) => Scaffold(
          appBar: AppBar(),
          body: Center(child: Text(label)),
        ),
        _ => const Scaffold(
          body: Center(child: CircularProgressIndicator()),
        ),
      },
    );
  }

  /// The open recipe; [timer] is null while idle.
  Widget _page(BrewRecipe recipe, BrewTimer? timer) {
    return Scaffold(
      appBar: AppBar(
        title: Text(recipe.name),
        actions: [
          IconButton(
            icon: const Icon(Icons.edit),
            onPressed: () => _edit(recipe.id),
          ),
        ],
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              recipe.description,
              style: const TextStyle(fontSize: 18),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
            const SizedBox(height: 16),
            GestureDetector(
              onTap: timer == null
                  ? () => _start(0)
                  : () => const ResetBrew().sendSignalToRust(),
              child: LayoutBuilder(
                builder: (context, constraints) {
                  final sweep = timer?.sweepSeconds;
                  return CircularTimer(
                    sweep: sweep == null ? null : Duration(seconds: sweep),
                    restartKey: (timer?.run, timer?.stepIndex),
                    progressColor: ColorScheme.of(context).primary,
                    constraints: BoxConstraints(
                      minWidth: constraints.maxWidth,
                      minHeight: constraints.maxWidth,
                    ),
                    child: timer == null
                        ? _RecipeInfo(recipe: recipe)
                        : _Countdown(timer: timer),
                  );
                },
              ),
            ),
            const SizedBox(height: 20),
            const Text(
              'Steps:',
              style: TextStyle(fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            Expanded(
              child: ListView.builder(
                itemCount: recipe.steps.length,
                itemBuilder: (_, i) => StepListTile(
                  step: recipe.steps[i],
                  onLongPress: () => _start(i),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// The running step's name over its `MM : SS` countdown.
class _Countdown extends StatelessWidget {
  const _Countdown({required this.timer});

  final BrewTimer timer;

  @override
  Widget build(BuildContext context) {
    final textStyle = TextTheme.of(context).titleLarge;
    // `remainingLabel` is `MM:SS`; each half gets its own fixed cells.
    final parts = timer.remainingLabel.split(':');
    return Column(
      children: [
        Text(timer.stepName),
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            FixedDigit(time: parts.first, style: textStyle),
            const SizedBox(width: 4),
            Text(':', style: textStyle),
            const SizedBox(width: 4),
            FixedDigit(time: parts.last, style: textStyle),
          ],
        ),
      ],
    );
  }
}

/// Coffee, water and total time while idle.
class _RecipeInfo extends StatelessWidget {
  const _RecipeInfo({required this.recipe});

  final BrewRecipe recipe;

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceEvenly,
      children: [
        _RecipeInfoText(
          icon: const Icon(Icons.grain),
          text: recipe.coffeeLabel,
        ),
        const SizedBox(height: 4),
        _RecipeInfoText(
          icon: const Icon(Icons.water_drop_outlined),
          text: recipe.waterLabel,
        ),
        const SizedBox(height: 4),
        _RecipeInfoText(
          icon: const Icon(Icons.timer_outlined),
          text: recipe.durationLabel,
        ),
      ],
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
