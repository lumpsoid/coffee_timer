import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_paper.dart';
import 'package:coffee_timer/widgets/texture_gallery.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:material_ui/material_ui.dart';

import '../fixtures.dart';

Widget _host(Widget child) => MaterialApp(home: Scaffold(body: child));

/// The tile for [label], keyed by its label in the gallery.
Finder _tile(String label) => find.byKey(ValueKey(label));

Finder _checkIn(String label) => find.descendant(
  of: _tile(label),
  matching: find.byIcon(Icons.check_circle),
);

/// A tile's semantics: a selectable button that taps.
Matcher _tileSemantics(String label, {required bool chosen}) =>
    matchesSemantics(
      label: label,
      isButton: true,
      hasSelectedState: true,
      isSelected: chosen,
      hasTapAction: true,
    );

void main() {
  final textures = draft().textures;

  Widget gallery(
    List<EditorTexture> textures, {
    ValueChanged<LabelTexture?>? onPick,
  }) => _host(
    TextureGallery(textures: textures, onPick: onPick ?? (_) {}),
  );

  testWidgets('shows a tile per texture, in order', (tester) async {
    // 200 wide fits two tiles a row, so the order is checked across rows too.
    await tester.pumpWidget(
      _host(
        SizedBox(
          width: 200,
          child: TextureGallery(textures: textures, onPick: (_) {}),
        ),
      ),
    );

    for (final texture in textures) {
      expect(find.text(texture.label), findsOneWidget);
    }
    for (var i = 1; i < textures.length; i++) {
      final before = tester.getTopLeft(find.text(textures[i - 1].label));
      final after = tester.getTopLeft(find.text(textures[i].label));
      final sameRow = (after.dy - before.dy).abs() < 1;
      expect(
        sameRow ? after.dx > before.dx : after.dy > before.dy,
        isTrue,
        reason: '${textures[i].label} should follow ${textures[i - 1].label}',
      );
    }
  });

  testWidgets('paints each tile with the paper Rust sent', (tester) async {
    await tester.pumpWidget(gallery(textures));

    for (final texture in textures) {
      final paper = tester.widget<LabelPaper>(
        find.descendant(
          of: _tile(texture.label),
          matching: find.byType(LabelPaper),
        ),
      );
      expect(paper.texture, texture.paper, reason: texture.label);
      expect(paper.stain, isNull, reason: '${texture.label} has no ring');
    }
    final auto = tester.widget<LabelPaper>(
      find.descendant(of: _tile('Auto'), matching: find.byType(LabelPaper)),
    );
    expect(auto.texture, LabelTexture.linen);
    final plain = tester.widget<LabelPaper>(
      find.descendant(of: _tile('Plain'), matching: find.byType(LabelPaper)),
    );
    expect(plain.texture, LabelTexture.plain);
  });

  testWidgets('marks only the tile Rust chose', (tester) async {
    final semantics = tester.ensureSemantics();

    await tester.pumpWidget(gallery(textures));
    expect(find.byIcon(Icons.check_circle), findsOneWidget);
    expect(_checkIn('Auto'), findsOneWidget);
    expect(
      tester.getSemantics(find.bySemanticsLabel('Auto')),
      _tileSemantics('Auto', chosen: true),
    );
    expect(
      tester.getSemantics(find.bySemanticsLabel('Plain')),
      _tileSemantics('Plain', chosen: false),
    );

    // Rust's next draft moves the choice to Plain.
    final moved = [
      for (final texture in textures)
        texture.copyWith(chosen: texture.label == 'Plain'),
    ];
    await tester.pumpWidget(gallery(moved));
    expect(find.byIcon(Icons.check_circle), findsOneWidget);
    expect(_checkIn('Plain'), findsOneWidget);
    expect(_checkIn('Auto'), findsNothing);
    expect(
      tester.getSemantics(find.bySemanticsLabel('Plain')),
      _tileSemantics('Plain', chosen: true),
    );
    expect(
      tester.getSemantics(find.bySemanticsLabel('Auto')),
      _tileSemantics('Auto', chosen: false),
    );
    semantics.dispose();
  });

  testWidgets('tapping a tile reports its texture, and waits for Rust', (
    tester,
  ) async {
    final picks = <LabelTexture?>[];
    await tester.pumpWidget(gallery(textures, onPick: picks.add));

    await tester.tap(find.text('Plain'));
    await tester.tap(find.text('Auto'));
    expect(picks, [LabelTexture.plain, null]);

    // Rust has not answered: the check stays on the tile it chose.
    expect(find.byIcon(Icons.check_circle), findsOneWidget);
    expect(_checkIn('Auto'), findsOneWidget);
    expect(_checkIn('Plain'), findsNothing);
  });

  testWidgets('fits a phone screen', (tester) async {
    tester.view
      ..physicalSize = const Size(1080, 2340)
      ..devicePixelRatio = 2.625;
    addTearDown(tester.view.reset);

    await tester.pumpWidget(gallery(textures));
    expect(tester.takeException(), isNull);
    expect(find.text('Plain'), findsOneWidget);
  });
}
