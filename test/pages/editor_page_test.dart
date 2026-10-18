import 'package:coffee_timer/holders/editor_holder.dart';
import 'package:coffee_timer/holders/recipe_list_holder.dart';
import 'package:coffee_timer/pages/editor_page.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../fixtures.dart';

/// The editor page over real holders, driven by hand-built updates. Nothing
/// here sends a signal: the page only sends on user input.
void main() {
  late EditorHolder editor;
  late RecipeListHolder recipes;
  late int opens;

  setUp(() {
    opens = 0;
    editor = EditorHolder()..apply(const EditorUpdateClosed());
    recipes = RecipeListHolder()
      ..apply(RecipeListUpdateSnapshot(recipes: [summary('r1')]));
  });

  /// A home page with the editor pushed over it.
  Future<void> pumpEditor(WidgetTester tester) async {
    await tester.pumpWidget(
      MaterialApp(
        home: Builder(
          builder: (context) => Scaffold(
            body: TextButton(
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => EditorPage(
                    editor: editor,
                    recipes: recipes,
                    open: () => opens++,
                  ),
                ),
              ),
              child: const Text('home'),
            ),
          ),
        ),
      ),
    );
    await tester.tap(find.text('home'));
    // The spinner never settles; let the push transition finish.
    await tester.pump();
    await tester.pump(const Duration(seconds: 1));
  }

  testWidgets('opens its draft, waits for it, then shows it', (
    tester,
  ) async {
    // A draft left open from before is not this page's.
    editor.apply(
      EditorUpdateOpen(
        draft: draft(id: 'old', name: 'Old'),
      ),
    );
    await pumpEditor(tester);
    expect(opens, 1);
    expect(find.byType(CircularProgressIndicator), findsOneWidget);
    expect(find.text('Old'), findsNothing);

    editor.apply(EditorUpdateOpen(draft: draft()));
    await tester.pump();
    expect(find.text('Edit Recipe'), findsOneWidget);
    expect(find.text('Total duration: 0:30'), findsOneWidget);
    expect(find.text('Step 1 name'), findsOneWidget);
    expect(find.text('MM:SS — 00:30'), findsOneWidget);
    expect(find.widgetWithText(TextFormField, 'Coffee (g)'), findsOneWidget);
    expect(find.text('15'), findsOneWidget);
    expect(find.widgetWithText(TextFormField, 'Water (g)'), findsOneWidget);
    expect(find.text('250'), findsOneWidget);
  });

  testWidgets('renders the add-step buttons from state, in order', (
    tester,
  ) async {
    await pumpEditor(tester);
    editor.apply(EditorUpdateOpen(draft: draft()));
    await tester.pump();

    final labels = tester
        .widgetList<Text>(
          find.descendant(of: find.byType(Wrap), matching: find.byType(Text)),
        )
        .map((text) => text.data);
    expect(labels, ['Add coffee', 'Add water', 'Wait', 'Add Step']);
  });

  testWidgets('shows field errors from the draft', (tester) async {
    await pumpEditor(tester);
    editor.apply(
      const EditorUpdateOpen(
        draft: EditorDraft(
          id: 'r2',
          title: 'New Recipe',
          canDelete: false,
          name: '',
          nameError: 'Enter a recipe name',
          description: '',
          coffee: '0',
          coffeeError: '1–200 g',
          water: 'x',
          waterError: 'Enter grams',
          totalLabel: '0:00',
          steps: [
            EditorStep(
              id: 's1',
              nameLabel: 'Step 1 name',
              name: '',
              durationText: 'x',
              durationLabel: '00:00',
              durationError: 'Enter seconds',
              waterText: '0',
              waterError: '1–2000 g',
            ),
          ],
          pourHint: 'Step pours add up to 0 g of 250 g',
          addStepButtons: [],
        ),
      ),
    );
    await tester.pump();

    expect(find.text('Enter a recipe name'), findsOneWidget);
    expect(find.text('1–200 g'), findsOneWidget);
    expect(find.text('Enter grams'), findsOneWidget);
    expect(find.text('Enter seconds'), findsOneWidget);
    expect(find.text('1–2000 g'), findsOneWidget);
    expect(find.text('Step pours add up to 0 g of 250 g'), findsOneWidget);
    expect(find.byTooltip('Delete'), findsNothing);
  });

  testWidgets('pops when Rust answers its opening with Closed', (
    tester,
  ) async {
    await pumpEditor(tester);
    editor.apply(const EditorUpdateClosed());
    await tester.pumpAndSettle();
    expect(find.byType(EditorPage), findsNothing);
    expect(find.text('home'), findsOneWidget);
  });

  testWidgets('pops when Rust closes the editor', (tester) async {
    await pumpEditor(tester);
    editor.apply(EditorUpdateOpen(draft: draft()));
    await tester.pump();

    editor.apply(const EditorUpdateClosed());
    await tester.pumpAndSettle();
    expect(find.byType(EditorPage), findsNothing);
    expect(find.text('home'), findsOneWidget);

    // A second Closed after the page is gone pops nothing else.
    editor.apply(const EditorUpdateClosed());
    await tester.pumpAndSettle();
    expect(find.text('home'), findsOneWidget);
  });

  testWidgets('shows a snackbar for each notice, repeats included', (
    tester,
  ) async {
    await pumpEditor(tester);
    editor.apply(EditorUpdateOpen(draft: draft(notice: 'Failed to save')));
    await tester.pump();
    expect(find.text('Failed to save'), findsOneWidget);

    // The same notice again is a second snackbar, queued behind the first.
    editor.apply(EditorUpdateOpen(draft: draft(notice: 'Failed to save')));
    await tester.pump();
    final messenger = tester.state<ScaffoldMessengerState>(
      find.byType(ScaffoldMessenger),
    )..hideCurrentSnackBar();
    await tester.pumpAndSettle();
    expect(find.text('Failed to save'), findsOneWidget);

    messenger.hideCurrentSnackBar();
    await tester.pumpAndSettle();
    expect(find.text('Failed to save'), findsNothing);
  });

  testWidgets('the delete dialog names the saved recipe, not the typed '
      'name', (tester) async {
    await pumpEditor(tester);
    editor.apply(EditorUpdateOpen(draft: draft(name: 'V60 typed  ')));
    await tester.pump();

    await tester.tap(find.byTooltip('Delete'));
    await tester.pumpAndSettle();
    expect(
      find.text('Delete "V60"? This action cannot be undone.'),
      findsOneWidget,
    );
  });

  testWidgets('a Closed under the delete dialog removes the editor, not the '
      'dialog', (tester) async {
    await pumpEditor(tester);
    editor.apply(EditorUpdateOpen(draft: draft()));
    await tester.pump();
    await tester.tap(find.byTooltip('Delete'));
    await tester.pumpAndSettle();

    editor.apply(const EditorUpdateClosed());
    await tester.pumpAndSettle();
    expect(find.byType(EditorPage), findsNothing);
    expect(find.byType(AlertDialog), findsOneWidget);
  });

  testWidgets('a Closed on top animates the editor out', (tester) async {
    await pumpEditor(tester);
    editor.apply(EditorUpdateOpen(draft: draft()));
    await tester.pump();

    // The pop that follows sends no CloseEditor: a send would fail here,
    // with no Rust library loaded.
    editor.apply(const EditorUpdateClosed());
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 50));
    expect(find.byType(EditorPage), findsOneWidget);

    await tester.pumpAndSettle();
    expect(find.byType(EditorPage), findsNothing);
    expect(find.text('home'), findsOneWidget);
  });
}
