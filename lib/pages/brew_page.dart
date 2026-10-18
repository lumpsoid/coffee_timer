import 'dart:async';

import 'package:coffee_timer/holders/brew_holder.dart';
import 'package:coffee_timer/holders/editor_holder.dart';
import 'package:coffee_timer/holders/recipe_list_holder.dart';
import 'package:coffee_timer/pages/editor_page.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/brew_dial.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:coffee_timer/widgets/label_paper_painter.dart';
import 'package:coffee_timer/widgets/label_parts.dart';
import 'package:coffee_timer/widgets/roast_meter.dart';
import 'package:coffee_timer/widgets/step_list_tile.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// The brew page: the open recipe printed on its own label, its timer dial
/// and its method. Once
/// subscribed it opens [recipeId] ([OpenBrew]) and shows the brew states
/// that follow. Tapping the dial starts, pauses or resumes the brew, or,
/// once it is done, brews again; the Reset button under it (shown while a
/// brew runs, is paused or is done) goes back to idle; long-pressing a step
/// starts from it. Leaving the page tells Rust
/// ([LeaveBrew]).
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

  /// The ratio controls, for this brew only; they follow Rust's updates
  /// while open.
  void _showRatioSheet(LabelColors colors) {
    unawaited(
      showModalBottomSheet<void>(
        context: context,
        backgroundColor: colors.paper,
        builder: (_) => ListenableBuilder(
          listenable: widget.brew,
          builder: (_, _) => switch (widget.brew.state) {
            BrewUpdateIdle(:final recipe) ||
            BrewUpdateRunning(:final recipe) ||
            BrewUpdatePaused(:final recipe) ||
            BrewUpdateFinished(:final recipe) => _RatioSheet(
              recipe: recipe,
              colors: colors,
            ),
            _ => const SizedBox.shrink(),
          },
        ),
      ),
    );
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
        BrewUpdateIdle(:final recipe) => _page(
          recipe,
          null,
          resettable: false,
          onDial: () => _start(0),
          face: (colors) => _IdleFace(recipe: recipe, colors: colors),
        ),
        BrewUpdateFinished(:final recipe, :final finished) => _page(
          recipe,
          null,
          resettable: true,
          onDial: () => _start(0),
          face: (colors) => _DoneFace(finished: finished, colors: colors),
        ),
        BrewUpdateRunning(:final recipe, :final timer) => _page(
          recipe,
          timer,
          resettable: true,
          onDial: () => const PauseBrew().sendSignalToRust(),
          face: (colors) =>
              _TimerFace(timer: timer, colors: colors, paused: false),
        ),
        BrewUpdatePaused(:final recipe, :final timer) => _page(
          recipe,
          timer,
          resettable: true,
          onDial: () => const ResumeBrew().sendSignalToRust(),
          face: (colors) =>
              _TimerFace(timer: timer, colors: colors, paused: true),
        ),
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

  /// The open recipe, printed on its own label paper; [timer] is null
  /// while idle or done, and otherwise also gives the next-step line under
  /// the dial. Tapping the dial calls [onDial]; [face] is its
  /// centre. The Reset button shows when [resettable].
  Widget _page(
    BrewRecipe recipe,
    BrewTimer? timer, {
    required bool resettable,
    required VoidCallback onDial,
    required Widget Function(LabelColors colors) face,
  }) {
    final colors = LabelColors.of(recipe.label, Theme.of(context).brightness);
    final paperBrightness = ThemeData.estimateBrightnessForColor(colors.paper);
    final sweep = timer?.sweepSeconds;
    final adjusted = recipe.ratio == BrewRatioSetting.adjusted;
    return CustomPaint(
      painter: LabelPaperPainter(
        colors: colors,
        texture: recipe.label.texture,
        stain: recipe.label.stain,
      ),
      child: Scaffold(
        backgroundColor: Colors.transparent,
        appBar: AppBar(
          backgroundColor: Colors.transparent,
          surfaceTintColor: Colors.transparent,
          foregroundColor: colors.ink,
          systemOverlayStyle: paperBrightness == Brightness.dark
              ? SystemUiOverlayStyle.light
              : SystemUiOverlayStyle.dark,
          centerTitle: true,
          title: RoastMeter.of(recipe.label.tone, colors),
          actions: [
            IconButton(
              icon: const Icon(Icons.edit),
              onPressed: () => _edit(recipe.id),
            ),
            const SizedBox(width: 8),
          ],
        ),
        body: SafeArea(
          top: false,
          child: SingleChildScrollView(
            padding: const EdgeInsets.fromLTRB(22, 0, 22, 22),
            child: Column(
              children: [
                LabelBadge(name: recipe.name, colors: colors, fontSize: 26),
                if (recipe.description.isNotEmpty)
                  Padding(
                    padding: const EdgeInsets.only(top: 6),
                    child: LabelTagline(
                      text: recipe.description,
                      colors: colors,
                    ),
                  ),
                const SizedBox(height: 12),
                _Rule(colors: colors),
                GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTap: () => _showRatioSheet(colors),
                  child: Padding(
                    padding: const EdgeInsets.symmetric(vertical: 8),
                    child: LabelSpecs(
                      colors: colors,
                      specs: [
                        (
                          caption: 'COFFEE',
                          value: recipe.coffeeLabel,
                          highlight: false,
                        ),
                        (
                          caption: 'WATER',
                          value: recipe.waterLabel,
                          highlight: adjusted,
                        ),
                        (
                          caption: 'RATIO',
                          value: recipe.ratioLabel,
                          highlight: adjusted,
                        ),
                      ],
                    ),
                  ),
                ),
                _Rule(colors: colors),
                const SizedBox(height: 14),
                ConstrainedBox(
                  constraints: const BoxConstraints(maxWidth: 300),
                  child: GestureDetector(
                    behavior: HitTestBehavior.opaque,
                    onTap: onDial,
                    child: BrewDial(
                      sweep: sweep == null ? null : Duration(seconds: sweep),
                      restartKey: (timer?.run, timer?.stepIndex),
                      steps: recipe.steps,
                      colors: colors,
                      child: face(colors),
                    ),
                  ),
                ),
                // Both slots stay while idle, so the page does not jump when
                // a brew starts.
                SizedBox(
                  height: 28,
                  child: timer == null
                      ? null
                      : Center(
                          child: Text(
                            // Not upper-cased like the captions: `g` is a
                            // unit.
                            timer.nextLabel,
                            maxLines: 1,
                            overflow: TextOverflow.ellipsis,
                            style: _caption(colors, size: 12),
                          ),
                        ),
                ),
                SizedBox(
                  height: 44,
                  child: !resettable
                      ? null
                      : TextButton.icon(
                          onPressed: () => const ResetBrew().sendSignalToRust(),
                          style: TextButton.styleFrom(
                            foregroundColor: colors.ink,
                          ),
                          icon: const Icon(Icons.replay, size: 18),
                          label: Text(
                            'RESET',
                            style: _caption(colors, size: 12).copyWith(
                              color: colors.ink,
                            ),
                          ),
                        ),
                ),
                _MethodHeading(colors: colors),
                const SizedBox(height: 6),
                for (final (i, step) in recipe.steps.indexed)
                  StepListTile(
                    step: step,
                    colors: colors,
                    onLongPress: () => _start(i),
                  ),
                const SizedBox(height: 14),
                Text(
                  'Tap the figures to change the ratio\n'
                  'Hold a step to start from there',
                  textAlign: TextAlign.center,
                  style: TextStyle(
                    fontFamily: 'Fraunces',
                    fontStyle: FontStyle.italic,
                    fontSize: 13,
                    color: colors.ink.withValues(alpha: 0.6),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

TextStyle _caption(LabelColors colors, {double size = 10}) => TextStyle(
  fontFamily: 'Oswald',
  fontSize: size,
  height: 1,
  letterSpacing: size * 0.24,
  fontWeight: FontWeight.w500,
  fontVariations: const [FontVariation.weight(500)],
  color: colors.ink.withValues(alpha: 0.7),
);

TextStyle _bigFigure(LabelColors colors) => TextStyle(
  fontFamily: 'Fraunces',
  fontSize: 56,
  height: 1.05,
  fontWeight: FontWeight.w500,
  fontVariations: const [
    FontVariation.weight(500),
    FontVariation.opticalSize(144),
  ],
  // Fixed-width digits, so the countdown does not jitter.
  fontFeatures: const [FontFeature.tabularFigures()],
  color: colors.ink,
);

/// The dial's centre while idle: the total time and how to start.
class _IdleFace extends StatelessWidget {
  const _IdleFace({required this.recipe, required this.colors});

  final BrewRecipe recipe;
  final LabelColors colors;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text('TOTAL', style: _caption(colors)),
        const SizedBox(height: 4),
        Text(recipe.durationLabel, style: _bigFigure(colors)),
        const SizedBox(height: 6),
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.play_arrow, size: 14, color: colors.accent),
            const SizedBox(width: 4),
            Text(
              'TAP TO BREW',
              style: _caption(colors, size: 11).copyWith(color: colors.accent),
            ),
          ],
        ),
      ],
    );
  }
}

/// The dial's centre once the brew is done: how long it brewed, what the
/// scale should read if any step poured, and that a tap brews again.
class _DoneFace extends StatelessWidget {
  const _DoneFace({required this.finished, required this.colors});

  final BrewFinished finished;
  final LabelColors colors;

  @override
  Widget build(BuildContext context) {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text('DONE', style: _caption(colors)),
        const SizedBox(height: 4),
        Text(finished.totalLabel, style: _bigFigure(colors)),
        if (finished.pouredLabel case final poured?)
          Padding(
            padding: const EdgeInsets.only(bottom: 4),
            child: Text(
              // Not upper-cased like the captions: `g` is a unit.
              poured,
              style: _caption(colors, size: 12).copyWith(
                color: colors.accent,
              ),
            ),
          ),
        const SizedBox(height: 6),
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.replay, size: 14, color: colors.accent),
            const SizedBox(width: 4),
            Text(
              'TAP TO BREW AGAIN',
              style: _caption(colors, size: 11).copyWith(color: colors.accent),
            ),
          ],
        ),
      ],
    );
  }
}

/// The dial's centre while running or [paused]: which step, its name, its
/// `MM:SS` countdown (faded while paused), the scale reading to pour to if
/// the step pours, and what a tap does.
class _TimerFace extends StatelessWidget {
  const _TimerFace({
    required this.timer,
    required this.colors,
    required this.paused,
  });

  final BrewTimer timer;
  final LabelColors colors;
  final bool paused;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 56),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(timer.positionLabel.toUpperCase(), style: _caption(colors)),
          const SizedBox(height: 6),
          Text(
            timer.stepName.toUpperCase(),
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              fontFamily: 'Oswald',
              fontSize: 20,
              height: 1.1,
              letterSpacing: 3.6,
              fontWeight: FontWeight.w600,
              fontVariations: const [FontVariation.weight(600)],
              color: colors.ink,
            ),
          ),
          Text(
            timer.remainingLabel,
            style: _bigFigure(colors).copyWith(
              color: paused ? colors.ink.withValues(alpha: 0.45) : colors.ink,
            ),
          ),
          if (timer.pourToLabel case final pourTo?)
            Padding(
              padding: const EdgeInsets.only(bottom: 4),
              child: Text(
                // Not upper-cased like the captions: `g` is a unit.
                pourTo,
                maxLines: 1,
                overflow: TextOverflow.ellipsis,
                style: _caption(colors, size: 12).copyWith(
                  color: colors.accent,
                ),
              ),
            ),
          const SizedBox(height: 4),
          if (paused)
            Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(Icons.play_arrow, size: 14, color: colors.accent),
                const SizedBox(width: 4),
                Text(
                  'TAP TO RESUME',
                  style: _caption(
                    colors,
                    size: 11,
                  ).copyWith(color: colors.accent),
                ),
              ],
            )
          else
            Text(
              'TAP TO PAUSE',
              style: _caption(colors, size: 11).copyWith(
                color: colors.ink.withValues(alpha: 0.55),
              ),
            ),
        ],
      ),
    );
  }
}

/// Less water, the ratio, more water; then the dose it makes, the way
/// back to the recipe's own ratio and, while adjusted, saving it to the
/// recipe. Saving closes the sheet: the page then shows the saved recipe,
/// or a snackbar says why it could not be saved.
class _RatioSheet extends StatelessWidget {
  const _RatioSheet({required this.recipe, required this.colors});

  final BrewRecipe recipe;
  final LabelColors colors;

  void _adjust(BrewRatioChange change) =>
      AdjustBrewRatio(change: change).sendSignalToRust();

  @override
  Widget build(BuildContext context) {
    final adjusted = recipe.ratio == BrewRatioSetting.adjusted;
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(22, 20, 22, 12),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('RATIO FOR THIS BREW', style: _caption(colors, size: 11)),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                IconButton.outlined(
                  tooltip: 'Less water',
                  color: colors.ink,
                  icon: const Icon(Icons.remove),
                  onPressed: () => _adjust(BrewRatioChange.lessWater),
                ),
                SizedBox(
                  width: 180,
                  child: Text(
                    recipe.ratioLabel,
                    textAlign: TextAlign.center,
                    style: _bigFigure(colors).copyWith(
                      color: adjusted ? colors.accent : colors.ink,
                    ),
                  ),
                ),
                IconButton.outlined(
                  tooltip: 'More water',
                  color: colors.ink,
                  icon: const Icon(Icons.add),
                  onPressed: () => _adjust(BrewRatioChange.moreWater),
                ),
              ],
            ),
            const SizedBox(height: 6),
            LabelTagline(
              text: '${recipe.coffeeLabel} coffee · ${recipe.waterLabel} water',
              colors: colors,
            ),
            const SizedBox(height: 8),
            TextButton(
              onPressed: adjusted
                  ? () => _adjust(BrewRatioChange.recipe)
                  : null,
              style: TextButton.styleFrom(foregroundColor: colors.accent),
              child: const Text("Back to the recipe's ratio"),
            ),
            if (adjusted)
              FilledButton.icon(
                onPressed: () {
                  Navigator.of(context).pop();
                  const SaveBrewRatio().sendSignalToRust();
                },
                style: FilledButton.styleFrom(
                  backgroundColor: colors.accent,
                  foregroundColor: colors.paper,
                ),
                icon: const Icon(Icons.save_outlined),
                label: const Text('Save to recipe'),
              ),
          ],
        ),
      ),
    );
  }
}

/// "METHOD" between two hairlines, over the step list.
class _MethodHeading extends StatelessWidget {
  const _MethodHeading({required this.colors});

  final LabelColors colors;

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(child: _Rule(colors: colors, alpha: 0.35)),
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 10),
          child: Text('METHOD', style: _caption(colors, size: 11)),
        ),
        Expanded(child: _Rule(colors: colors, alpha: 0.35)),
      ],
    );
  }
}

class _Rule extends StatelessWidget {
  const _Rule({required this.colors, this.alpha = 0.3});

  final LabelColors colors;
  final double alpha;

  @override
  Widget build(BuildContext context) {
    return Divider(
      height: 1,
      thickness: 1,
      color: colors.ink.withValues(alpha: alpha),
    );
  }
}
