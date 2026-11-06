import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/coffee_tape.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:coffee_timer/widgets/label_parts.dart';
import 'package:flutter/material.dart';

/// Less water, the ratio, more water; the coffee on its tape; then the
/// dose it makes; then Reset (back to the recipe's own dose) and Save (make
/// it the recipe's own), enabled while the dose is changed. Saving closes
/// the sheet: the page then shows the
/// saved recipe, or a snackbar says why it could not be saved.
///
/// It only reports the user's intents through its callbacks; the page sends
/// them to Rust, whose next update rebuilds the sheet.
class DoseSheet extends StatelessWidget {
  const DoseSheet({
    required this.recipe,
    required this.colors,
    required this.captionStyle,
    required this.figureStyle,
    required this.onRatio,
    required this.onCoffee,
    required this.onReset,
    required this.onSave,
    super.key,
  });

  final BrewRecipe recipe;
  final LabelColors colors;

  /// The section captions' style.
  final TextStyle captionStyle;

  /// The ratio figure's style; the coffee figure is a smaller cut of it.
  final TextStyle figureStyle;

  /// One press of less or more water.
  final ValueChanged<BrewRatioChange> onRatio;

  /// Each whole gram the coffee tape reaches.
  final ValueChanged<int> onCoffee;

  /// Back to the recipe's own dose.
  final VoidCallback onReset;

  /// Make the dose the recipe's own; the sheet closes first.
  final VoidCallback onSave;

  @override
  Widget build(BuildContext context) {
    final ratioAdjusted = recipe.ratio == BrewRatioSetting.adjusted;
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(22, 20, 22, 12),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Text('RATIO FOR THIS BREW', style: captionStyle),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                IconButton.outlined(
                  tooltip: 'Less water',
                  color: colors.ink,
                  icon: const Icon(Icons.remove),
                  onPressed: () => onRatio(BrewRatioChange.lessWater),
                ),
                SizedBox(
                  width: 180,
                  child: Text(
                    recipe.ratioLabel,
                    textAlign: TextAlign.center,
                    style: figureStyle.copyWith(
                      color: ratioAdjusted ? colors.accent : colors.ink,
                    ),
                  ),
                ),
                IconButton.outlined(
                  tooltip: 'More water',
                  color: colors.ink,
                  icon: const Icon(Icons.add),
                  onPressed: () => onRatio(BrewRatioChange.moreWater),
                ),
              ],
            ),
            const SizedBox(height: 16),
            Text('COFFEE FOR THIS BREW', style: captionStyle),
            const SizedBox(height: 4),
            CoffeeTape(
              coffee: recipe.coffee,
              label: recipe.coffeeLabel,
              labelStyle: figureStyle.copyWith(fontSize: 40),
              colors: colors,
              onGrams: onCoffee,
            ),
            const SizedBox(height: 10),
            LabelTagline(
              text: '${recipe.coffeeLabel} coffee · ${recipe.waterLabel} water',
              colors: colors,
            ),
            const SizedBox(height: 16),
            // Always laid out, enabled only while the dose is changed, so
            // the sheet does not jump.
            Row(
              children: [
                Expanded(
                  child: OutlinedButton.icon(
                    onPressed: recipe.adjusted ? onReset : null,
                    style: OutlinedButton.styleFrom(
                      foregroundColor: colors.accent,
                      disabledForegroundColor: colors.ink.withValues(
                        alpha: 0.38,
                      ),
                      side: BorderSide(
                        color: colors.ink.withValues(alpha: 0.3),
                      ),
                    ),
                    icon: const Icon(Icons.restart_alt),
                    label: const Text('Reset'),
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: FilledButton.icon(
                    onPressed: recipe.adjusted
                        ? () {
                            Navigator.of(context).pop();
                            onSave();
                          }
                        : null,
                    style: FilledButton.styleFrom(
                      backgroundColor: colors.accent,
                      foregroundColor: colors.paper,
                      disabledBackgroundColor: colors.ink.withValues(
                        alpha: 0.12,
                      ),
                      disabledForegroundColor: colors.ink.withValues(
                        alpha: 0.38,
                      ),
                    ),
                    icon: const Icon(Icons.save_outlined),
                    label: const Text('Save'),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
