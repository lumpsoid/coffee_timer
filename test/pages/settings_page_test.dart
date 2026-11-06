import 'package:coffee_timer/holders/settings_holder.dart';
import 'package:coffee_timer/pages/settings_page.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../fixtures.dart';

/// The settings page over a real holder, driven by hand-built updates.
/// Nothing here sends a signal: the page only sends on user input, and a
/// send would fail without Rust.
void main() {
  late SettingsHolder holder;

  setUp(() => holder = SettingsHolder()..apply(settings()));

  Future<void> pumpPage(WidgetTester tester) async {
    await tester.pumpWidget(MaterialApp(home: SettingsPage(settings: holder)));
  }

  bool switchValue(WidgetTester tester, String title) => tester
      .widget<SwitchListTile>(find.widgetWithText(SwitchListTile, title))
      .value;

  testWidgets('shows the cues Rust sent: sound, vibration, pre-alert', (
    tester,
  ) async {
    await pumpPage(tester);
    expect(switchValue(tester, 'Sound'), isTrue);
    expect(switchValue(tester, 'Vibration'), isTrue);
    expect(find.text('Pre-alert before each step ends'), findsOneWidget);
    expect(find.text('3 s'), findsOneWidget);
  });

  testWidgets('shows the version Rust sent', (tester) async {
    await pumpPage(tester);
    expect(find.text('Version 0.1.0 (2391c1d)'), findsOneWidget);
  });

  testWidgets('follows each new update from Rust', (tester) async {
    await pumpPage(tester);
    holder.apply(settings(sound: false, vibration: false, preAlertSeconds: 0));
    await tester.pump();
    expect(switchValue(tester, 'Sound'), isFalse);
    expect(switchValue(tester, 'Vibration'), isFalse);
    expect(find.text('Off'), findsOneWidget);
    expect(find.text('3 s'), findsNothing);

    holder.apply(settings(vibration: false, preAlertSeconds: 10));
    await tester.pump();
    expect(switchValue(tester, 'Sound'), isTrue);
    expect(switchValue(tester, 'Vibration'), isFalse);
    expect(find.text('10 s'), findsOneWidget);
  });

  testWidgets('the pre-alert picker offers the options Rust sent', (
    tester,
  ) async {
    await pumpPage(tester);
    await tester.tap(find.text('3 s'));
    await tester.pumpAndSettle();
    for (final label in ['Off', '5 s', '10 s']) {
      expect(find.text(label), findsOneWidget, reason: label);
    }
  });
}
