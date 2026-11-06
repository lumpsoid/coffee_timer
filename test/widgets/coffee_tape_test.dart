import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/coffee_tape.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../fixtures.dart';

final _colors = LabelColors.of(label(), Brightness.light);

BrewCoffee _coffee(int grams, {int min = 1, int max = 200}) => BrewCoffee(
  grams: grams,
  minGrams: min,
  maxGrams: max,
  setting: BrewCoffeeSetting.recipe,
);

/// A tape at `coffee`, recording every gram it sends.
Future<List<int>> _pumpTape(WidgetTester tester, BrewCoffee coffee) async {
  final sent = <int>[];
  await tester.pumpWidget(
    MaterialApp(
      home: Scaffold(
        body: Center(
          child: SizedBox(
            width: 360,
            child: CoffeeTape(
              coffee: coffee,
              label: '${coffee.grams}g',
              labelStyle: const TextStyle(fontSize: 40),
              colors: _colors,
              onGrams: sent.add,
            ),
          ),
        ),
      ),
    ),
  );
  return sent;
}

/// Drags the tape by `dx` in `steps` moves, `msPerStep` apart.
Future<void> _drag(
  WidgetTester tester,
  double dx, {
  int steps = 20,
  int msPerStep = 50,
}) async {
  final gesture = await tester.startGesture(
    tester.getCenter(find.byType(CoffeeTape)),
  );
  var at = Duration.zero;
  for (var i = 0; i < steps; i++) {
    at += Duration(milliseconds: msPerStep);
    await gesture.moveBy(Offset(dx / steps, 0), timeStamp: at);
    await tester.pump();
  }
  await gesture.up(timeStamp: at);
  await tester.pump();
}

void main() {
  testWidgets('shows the coffee it is given', (tester) async {
    await _pumpTape(tester, _coffee(15));
    expect(find.text('15g'), findsOneWidget);
  });

  testWidgets('dragging left brings more grams, one at a time', (
    tester,
  ) async {
    final sent = await _pumpTape(tester, _coffee(15));
    // Slowly: 0.1 px/ms, 1 g per 24 px; 120 px is 5 g.
    await _drag(tester, -120);
    expect(sent, [16, 17, 18, 19, 20]);
  });

  testWidgets('dragging right takes grams away', (tester) async {
    final sent = await _pumpTape(tester, _coffee(15));
    await _drag(tester, 120);
    expect(sent.last, 10);
  });

  testWidgets('a fast swipe moves more grams than a slow drag', (
    tester,
  ) async {
    final slow = await _pumpTape(tester, _coffee(100));
    await _drag(tester, -120);
    final slowGrams = slow.last - 100;

    final fast = await _pumpTape(tester, _coffee(100));
    // 120 px in 20 ms: 6 px/ms, past the top of the speed-up.
    await _drag(tester, -120, msPerStep: 1);
    expect(fast.last - 100, greaterThan(slowGrams * 4));
  });

  testWidgets('the tape stops at its bounds', (tester) async {
    final sent = await _pumpTape(tester, _coffee(3));
    await _drag(tester, 240);
    expect(sent.last, 1);
    expect(sent.where((g) => g < 1), isEmpty);
  });

  test('the speed curve is slow below its start and capped above its top', () {
    const slow = 1 / CoffeeTape.slowPixelsPerGram;
    expect(CoffeeTape.gramsPerPixel(0), slow);
    expect(CoffeeTape.gramsPerPixel(CoffeeTape.speedUpFrom), slow);
    expect(
      CoffeeTape.gramsPerPixel(CoffeeTape.speedUpTo),
      closeTo(slow * CoffeeTape.topGain, 1e-9),
    );
    expect(
      CoffeeTape.gramsPerPixel(10),
      CoffeeTape.gramsPerPixel(CoffeeTape.speedUpTo),
    );
  });
}
