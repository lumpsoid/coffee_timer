import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/brew_dial.dart';
import 'package:coffee_timer/widgets/label_colors.dart';
import 'package:coffee_timer/widgets/recipe_summary_tile.dart';
import 'package:coffee_timer/widgets/step_list_tile.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../fixtures.dart';

Widget _host(Widget child) => MaterialApp(home: Scaffold(body: child));

final _colors = LabelColors.of(label(), Brightness.light);

void main() {
  group('StepListTile', () {
    Future<void> pumpTile(WidgetTester tester, BrewStepStatus status) =>
        tester.pumpWidget(
          _host(
            StepListTile(
              step: BrewStep(
                numberLabel: '2',
                name: 'Pour',
                durationLabel: '30s',
                status: status,
                trackStart: 0,
                trackEnd: 1,
              ),
              colors: _colors,
              onLongPress: () {},
            ),
          ),
        );

    testWidgets('a pending step shows its number', (tester) async {
      await pumpTile(tester, BrewStepStatus.pending);
      expect(find.text('2'), findsOneWidget);
      expect(find.text('Pour'), findsOneWidget);
      expect(find.text('30s'), findsOneWidget);
      expect(find.byIcon(Icons.play_arrow), findsNothing);
      expect(find.byIcon(Icons.check), findsNothing);
    });

    testWidgets('the current step shows a play arrow', (tester) async {
      await pumpTile(tester, BrewStepStatus.current);
      expect(find.byIcon(Icons.play_arrow), findsOneWidget);
      expect(find.text('2'), findsNothing);
    });

    testWidgets('a paused step shows a pause sign', (tester) async {
      await pumpTile(tester, BrewStepStatus.paused);
      expect(find.byIcon(Icons.pause), findsOneWidget);
      expect(find.byIcon(Icons.play_arrow), findsNothing);
      expect(find.text('2'), findsNothing);
    });

    testWidgets('a done step shows a check', (tester) async {
      await pumpTile(tester, BrewStepStatus.done);
      expect(find.byIcon(Icons.check), findsOneWidget);
      expect(find.text('2'), findsNothing);
    });

    testWidgets('long press calls back', (tester) async {
      var pressed = 0;
      await tester.pumpWidget(
        _host(
          StepListTile(
            step: const BrewStep(
              numberLabel: '1',
              name: 'Bloom',
              durationLabel: '30s',
              status: BrewStepStatus.pending,
              trackStart: 0,
              trackEnd: 1,
            ),
            colors: _colors,
            onLongPress: () => pressed++,
          ),
        ),
      );
      await tester.longPress(find.text('Bloom'));
      expect(pressed, 1);
    });
  });

  group('RecipeSummaryTile', () {
    testWidgets('lays the labels under UI captions', (tester) async {
      var taps = 0;
      var longPresses = 0;
      await tester.pumpWidget(
        _host(
          RecipeSummaryTile(
            name: 'V60',
            description: 'clean',
            coffeeLabel: '20 g',
            waterLabel: '100 g',
            timeLabel: '1:35',
            label: label(),
            onTap: () => taps++,
            onLongPress: () => longPresses++,
          ),
        ),
      );

      for (final text in ['COFFEE', '20 g', 'WATER', '100 g', 'TIME', '1:35']) {
        expect(find.text(text), findsOneWidget);
      }
      expect(find.text('V60'.toUpperCase()), findsOneWidget);
      expect(find.text('— clean —'), findsOneWidget);

      await tester.tap(find.text('1:35'));
      await tester.longPress(find.text('1:35'));
      expect((taps, longPresses), (1, 1));
    });

    testWidgets('leaves out an empty description', (tester) async {
      await tester.pumpWidget(
        _host(
          RecipeSummaryTile(
            name: 'V60',
            description: '',
            coffeeLabel: '20 g',
            waterLabel: '100 g',
            timeLabel: '1:35',
            label: label(tone: LabelTone.crema),
          ),
        ),
      );
      expect(find.textContaining('—'), findsNothing);
    });

    testWidgets('paints every tone in light and dark', (tester) async {
      for (final brightness in Brightness.values) {
        for (final tone in LabelTone.values) {
          await tester.pumpWidget(
            MaterialApp(
              theme: ThemeData(brightness: brightness),
              home: Scaffold(
                body: RecipeSummaryTile(
                  name: 'A very long recipe name that cannot fit',
                  description: 'clean',
                  coffeeLabel: '20 g',
                  waterLabel: '100 g',
                  timeLabel: '1:35',
                  label: label(tone: tone),
                ),
              ),
            ),
          );
          expect(tester.takeException(), isNull);
        }
      }
    });
  });

  group('BrewDial', () {
    double ring(WidgetTester tester) =>
        (tester
                    .widget<CustomPaint>(
                      find.descendant(
                        of: find.byType(BrewDial),
                        matching: find.byType(CustomPaint),
                      ),
                    )
                    .painter!
                as BrewDialPainter)
            .progress;

    Widget timer({
      required Duration? sweep,
      required Object restartKey,
      BrewStepStatus first = BrewStepStatus.current,
    }) => _host(
      BrewDial(
        sweep: sweep,
        restartKey: restartKey,
        steps: brewRecipe(
          statuses: [first, BrewStepStatus.pending],
        ).steps,
        colors: _colors,
        child: const Text('inside'),
      ),
    );

    testWidgets('sweeps over the duration and repeats', (tester) async {
      await tester.pumpWidget(
        timer(sweep: const Duration(seconds: 10), restartKey: 0),
      );
      expect(find.text('inside'), findsOneWidget);
      expect(ring(tester), 0);

      await tester.pump(const Duration(seconds: 5));
      expect(ring(tester), closeTo(0.5, 0.01));

      await tester.pump(const Duration(seconds: 6));
      expect(ring(tester), closeTo(0.1, 0.01));
    });

    testWidgets('holds still without a sweep', (tester) async {
      await tester.pumpWidget(timer(sweep: null, restartKey: 0));
      await tester.pump(const Duration(seconds: 5));
      expect(ring(tester), 0);
    });

    testWidgets('a new restart key starts the sweep over, even for the same '
        'duration', (tester) async {
      const tenSeconds = Duration(seconds: 10);
      await tester.pumpWidget(timer(sweep: tenSeconds, restartKey: (0, 0)));
      await tester.pump(const Duration(seconds: 5));
      expect(ring(tester), closeTo(0.5, 0.01));

      await tester.pumpWidget(timer(sweep: tenSeconds, restartKey: (0, 1)));
      expect(ring(tester), 0);
      await tester.pump(const Duration(seconds: 2));
      expect(ring(tester), closeTo(0.2, 0.01));
    });

    testWidgets('the same restart key keeps the sweep going', (tester) async {
      const tenSeconds = Duration(seconds: 10);
      await tester.pumpWidget(timer(sweep: tenSeconds, restartKey: (0, 0)));
      await tester.pump(const Duration(seconds: 5));

      await tester.pumpWidget(timer(sweep: tenSeconds, restartKey: (0, 0)));
      await tester.pump(const Duration(seconds: 1));
      expect(ring(tester), closeTo(0.6, 0.01));
    });

    testWidgets('a pause holds the sweep and a resume goes on from there', (
      tester,
    ) async {
      const tenSeconds = Duration(seconds: 10);
      await tester.pumpWidget(timer(sweep: tenSeconds, restartKey: (1, 0)));
      await tester.pump(const Duration(seconds: 3));

      await tester.pumpWidget(
        timer(
          sweep: tenSeconds,
          restartKey: (1, 0),
          first: BrewStepStatus.paused,
        ),
      );
      await tester.pump(const Duration(seconds: 5));
      expect(ring(tester), closeTo(0.3, 0.01));

      await tester.pumpWidget(timer(sweep: tenSeconds, restartKey: (1, 0)));
      await tester.pump(const Duration(seconds: 2));
      expect(ring(tester), closeTo(0.5, 0.01));
    });
  });
}
