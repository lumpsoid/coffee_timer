import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/coffee_tape.dart';
import 'package:coffee_timer/widgets/dose_sheet.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../fixtures.dart';

final _colors = LabelColors.of(label(), Brightness.light);

/// What the sheet reported, in order.
class _Intents {
  final List<BrewRatioChange> ratio = [];
  final List<int> coffee = [];
  int resets = 0;
  int saves = 0;
}

/// Opens the sheet over a page, as the brew page does, for `recipe`.
Future<_Intents> _openSheet(WidgetTester tester, BrewRecipe recipe) async {
  final intents = _Intents();
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: Builder(
          builder: (context) => TextButton(
            onPressed: () => showModalBottomSheet<void>(
              context: context,
              isScrollControlled: true,
              builder: (_) => DoseSheet(
                recipe: recipe,
                colors: _colors,
                captionStyle: const TextStyle(fontSize: 11),
                figureStyle: const TextStyle(fontSize: 56),
                onRatio: intents.ratio.add,
                onCoffee: intents.coffee.add,
                onReset: () => intents.resets++,
                onSave: () => intents.saves++,
              ),
            ),
            child: const Text('open'),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.text('open'));
  await tester.pumpAndSettle();
  return intents;
}

BrewRecipe _changed() => brewRecipe().copyWith(
  coffeeLabel: '12g',
  waterLabel: '60g',
  coffee: const BrewCoffee(
    grams: 12,
    minGrams: 1,
    maxGrams: 200,
    setting: BrewCoffeeSetting.adjusted,
  ),
  adjusted: true,
);

ButtonStyleButton _button(WidgetTester tester, String text) =>
    tester.widget<ButtonStyleButton>(
      find.ancestor(
        of: find.text(text),
        matching: find.byWidgetPredicate((w) => w is ButtonStyleButton),
      ),
    );

void main() {
  testWidgets('shows the ratio, the coffee and the dose they make', (
    tester,
  ) async {
    await _openSheet(tester, brewRecipe());
    expect(find.text('1:5'), findsOneWidget);
    expect(find.text('20g'), findsOneWidget);
    expect(find.text('— 20g coffee · 100g water —'), findsOneWidget);
  });

  testWidgets('at the recipe dose, Reset and Save are there but disabled', (
    tester,
  ) async {
    await _openSheet(tester, brewRecipe());
    expect(_button(tester, 'Reset').enabled, isFalse);
    expect(_button(tester, 'Save').enabled, isFalse);
  });

  testWidgets('Reset and Save sit in one row', (tester) async {
    await _openSheet(tester, _changed());
    final reset = tester.getCenter(find.text('Reset'));
    final save = tester.getCenter(find.text('Save'));
    expect(reset.dy, save.dy);
    expect(reset.dx, lessThan(save.dx));
  });

  testWidgets('a changed dose enables Reset, which reports it', (
    tester,
  ) async {
    final intents = await _openSheet(tester, _changed());
    expect(find.text('— 12g coffee · 60g water —'), findsOneWidget);
    await tester.tap(find.text('Reset'));
    await tester.pump();
    expect(intents.resets, 1);
    expect(find.byType(DoseSheet), findsOneWidget, reason: 'stays open');
  });

  testWidgets('Save closes the sheet, then reports it', (tester) async {
    final intents = await _openSheet(tester, _changed());
    await tester.tap(find.text('Save'));
    await tester.pumpAndSettle();
    expect(intents.saves, 1);
    expect(find.byType(DoseSheet), findsNothing);
  });

  testWidgets('the water buttons report less and more water', (tester) async {
    final intents = await _openSheet(tester, brewRecipe());
    await tester.tap(find.byTooltip('Less water'));
    await tester.tap(find.byTooltip('More water'));
    expect(intents.ratio, [
      BrewRatioChange.lessWater,
      BrewRatioChange.moreWater,
    ]);
  });

  testWidgets('swiping the coffee tape reports each gram', (tester) async {
    final intents = await _openSheet(tester, brewRecipe());
    final gesture = await tester.startGesture(
      tester.getCenter(find.byType(CoffeeTape)),
    );
    var at = Duration.zero;
    for (var i = 0; i < 12; i++) {
      at += const Duration(milliseconds: 50);
      await gesture.moveBy(const Offset(-6, 0), timeStamp: at);
      await tester.pump();
    }
    await gesture.up(timeStamp: at);
    await tester.pump();
    // 72 px slowly, 1 g per 24 px, left is more.
    expect(intents.coffee, [21, 22, 23]);
  });
}
