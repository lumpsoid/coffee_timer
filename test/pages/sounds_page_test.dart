import 'package:coffee_timer/holders/settings_holder.dart';
import 'package:coffee_timer/pages/settings_page.dart';
import 'package:coffee_timer/pages/sounds_page.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../fixtures.dart';

/// The sounds page over a real holder, driven by hand-built updates.
/// Nothing here taps a sound: a tap sends a signal, which would fail
/// without Rust.
void main() {
  late SettingsHolder holder;

  setUp(() => holder = SettingsHolder()..apply(settings()));

  Future<void> pumpPage(WidgetTester tester) async {
    await tester.pumpWidget(MaterialApp(home: SoundsPage(settings: holder)));
  }

  /// The row of [id] in the picker [picker] (`step` or `finish`).
  ListTile row(WidgetTester tester, String picker, String id) =>
      tester.widget<ListTile>(
        find.descendant(
          of: find.byKey(ValueKey((picker, id))),
          matching: find.byType(ListTile),
        ),
      );

  /// The ids [picker] marks as chosen, out of [ids].
  List<String> chosen(WidgetTester tester, String picker, List<String> ids) => [
    for (final id in ids)
      if (row(tester, picker, id).selected) id,
  ];

  const stepIds = ['kalimba', 'breath-and-bowl'];
  const finishIds = ['singing-bowl', 'gong'];

  testWidgets('lists each picker its own sounds Rust sent, with lengths', (
    tester,
  ) async {
    await pumpPage(tester);
    expect(find.text('BETWEEN STEPS'), findsOneWidget);
    expect(find.text('END OF BREW'), findsOneWidget);
    for (final id in stepIds) {
      expect(find.byKey(ValueKey(('step', id))), findsOneWidget, reason: id);
      expect(find.byKey(ValueKey(('finish', id))), findsNothing, reason: id);
    }
    for (final id in finishIds) {
      expect(find.byKey(ValueKey(('finish', id))), findsOneWidget, reason: id);
      expect(find.byKey(ValueKey(('step', id))), findsNothing, reason: id);
    }
    for (final label in [
      'Kalimba',
      'Breath and bowl',
      'Singing bowl',
      'Gong',
    ]) {
      expect(find.text(label), findsOneWidget, reason: label);
    }
    expect(find.text('1.0 s'), findsNWidgets(2));
    expect(find.text('3.0 s'), findsOneWidget);
    expect(find.text('4.4 s'), findsOneWidget);
  });

  testWidgets('marks the choices Rust sent', (tester) async {
    await pumpPage(tester);
    expect(chosen(tester, 'step', stepIds), ['breath-and-bowl']);
    expect(chosen(tester, 'finish', finishIds), ['singing-bowl']);
  });

  testWidgets('follows each new update from Rust', (tester) async {
    await pumpPage(tester);
    holder.apply(settings(stepSound: 'kalimba', finishSound: 'gong'));
    await tester.pump();
    expect(chosen(tester, 'step', stepIds), ['kalimba']);
    expect(chosen(tester, 'finish', finishIds), ['gong']);
  });

  testWidgets('opens from the settings page', (tester) async {
    await tester.pumpWidget(MaterialApp(home: SettingsPage(settings: holder)));
    await tester.tap(find.text('Sounds'));
    await tester.pumpAndSettle();
    expect(find.byType(SoundsPage), findsOneWidget);
    expect(find.text('BETWEEN STEPS'), findsOneWidget);
  });
}
