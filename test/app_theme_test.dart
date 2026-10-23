import 'package:coffee_timer/app_theme.dart';
import 'package:coffee_timer/widgets/label_parts.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

void main() {
  for (final brightness in Brightness.values) {
    testWidgets('$brightness chrome is set in the label type', (tester) async {
      await tester.pumpWidget(
        MaterialApp(
          theme: appTheme(const Color(0xFF795548), brightness),
          home: Scaffold(
            appBar: AppBar(title: const Text('RECIPES')),
            body: Column(
              children: [
                const Text('Body'),
                TextButton(onPressed: () {}, child: const Text('Undo')),
                const RuledHeading(text: 'Steps', ink: Colors.black),
              ],
            ),
          ),
        ),
      );

      TextStyle styleOf(String text) => tester
          .widget<RichText>(
            find.descendant(
              of: find.text(text),
              matching: find.byType(RichText),
            ),
          )
          .text
          .style!;

      final title = styleOf('RECIPES');
      expect(title.fontFamily, 'Oswald');
      expect(title.letterSpacing, 4);
      expect(styleOf('Body').fontFamily, 'Fraunces');
      expect(styleOf('Undo').fontFamily, 'Oswald');
      expect(find.text('STEPS'), findsOneWidget);
      expect(tester.widget<AppBar>(find.byType(AppBar)).centerTitle, isNull);
      expect(
        Theme.of(tester.element(find.text('Body'))).appBarTheme.centerTitle,
        isTrue,
      );
    });
  }
}
