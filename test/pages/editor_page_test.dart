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

  /// A home page with the editor pushed over it, on a phone-sized screen
  /// (411 × 891), so a card that overflows a phone's width fails the test.
  Future<void> pumpEditor(WidgetTester tester) async {
    tester.view
      ..physicalSize = const Size(1080, 2340)
      ..devicePixelRatio = 2.625;
    addTearDown(tester.view.reset);
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
          pourSpeed: '40',
          pourSpeedError: '0.5–30 g/s',
          totalLabel: '0:00',
          steps: [
            EditorStep(
              id: 's1',
              nameLabel: 'Step 1 name',
              name: '',
              durationText: 'x',
              durationLabel: '00:00',
              durationError: 'Enter seconds',
              durationRevision: 0,
              waterText: '0',
              waterError: '1–2000 g',
            ),
          ],
          pourHint: 'Step pours add up to 0 g of 250 g',
          addStepButtons: [],
          durationNudgeButtons: [],
        ),
      ),
    );
    await tester.pump();

    expect(find.text('Enter a recipe name'), findsOneWidget);
    expect(find.text('1–200 g'), findsOneWidget);
    expect(find.text('Enter grams'), findsOneWidget);
    expect(find.text('Enter seconds'), findsOneWidget);
    expect(find.text('1–2000 g'), findsOneWidget);
    expect(find.text('0.5–30 g/s'), findsOneWidget);
    expect(find.text('Step pours add up to 0 g of 250 g'), findsOneWidget);
    expect(find.byTooltip('Delete'), findsNothing);
  });

  // The reorderable list re-creates a moved card (its item key includes the
  // index), so a field keyed by position would keep the old row's text here.
  testWidgets('a step order from Rust moves each card with its own fields', (
    tester,
  ) async {
    EditorStep step(String id, int position, String name) => EditorStep(
      id: id,
      nameLabel: 'Step $position name',
      name: name,
      durationText: '0',
      durationLabel: '00:00',
      durationRevision: 0,
      waterText: '',
    );
    FormFieldState<String> nameField(String id) => tester.state(
      find
          .descendant(
            of: find.byKey(ValueKey(id)),
            matching: find.byType(TextFormField),
          )
          .first,
    );

    await pumpEditor(tester);
    editor.apply(
      EditorUpdateOpen(
        draft: draft().copyWith(
          steps: [step('s1', 1, 'Bloom'), step('s2', 2, 'Pour')],
        ),
      ),
    );
    await tester.pump();
    expect(find.byIcon(Icons.drag_handle), findsNWidgets(2));

    editor.apply(
      EditorUpdateOpen(
        draft: draft().copyWith(
          steps: [step('s2', 1, 'Pour'), step('s1', 2, 'Bloom')],
        ),
      ),
    );
    await tester.pump();

    expect(nameField('s1').value, 'Bloom');
    expect(nameField('s2').value, 'Pour');
    final labels = tester
        .widgetList<TextField>(find.byType(TextField))
        .map((field) => field.decoration?.labelText)
        .where((label) => label?.endsWith(' name') ?? false)
        .toList();
    expect(labels, ['Recipe name', 'Step 1 name', 'Step 2 name']);
    expect(
      tester.getTopLeft(find.byKey(const ValueKey('s2'))).dy,
      lessThan(tester.getTopLeft(find.byKey(const ValueKey('s1'))).dy),
    );
  });

  testWidgets('renders the duration nudge buttons from state, in order', (
    tester,
  ) async {
    await pumpEditor(tester);
    editor.apply(EditorUpdateOpen(draft: draft()));
    await tester.pump();

    final labels = tester
        .widgetList<Text>(
          find.descendant(
            of: find.byType(Tooltip),
            matching: find.byType(Text),
          ),
        )
        .map((text) => text.data);
    expect(labels, ['−15', '−5', '+5', '+15']);
    expect(find.byTooltip('Add 15 seconds'), findsOneWidget);
  });

  testWidgets('a nudged duration from Rust replaces the field text, typing '
      'does not', (tester) async {
    EditorStep step(String text, int revision) => EditorStep(
      id: 's1',
      nameLabel: 'Step 1 name',
      name: 'Bloom',
      durationText: text,
      durationLabel: '00:00',
      durationRevision: revision,
      waterText: '',
    );
    Finder durationField() =>
        find.widgetWithText(TextFormField, 'Duration (seconds)');
    String? durationValue() =>
        tester.state<FormFieldState<String>>(durationField()).value;

    await pumpEditor(tester);
    editor.apply(
      EditorUpdateOpen(draft: draft().copyWith(steps: [step('30', 0)])),
    );
    await tester.pump();
    expect(durationValue(), '30');

    // Typing (as the field's own change, so nothing is sent): Rust echoes
    // the text at the same revision; the field keeps what it holds, even
    // when an echo lags behind the keystrokes.
    tester.state<FormFieldState<String>>(durationField()).didChange('305');
    editor.apply(
      EditorUpdateOpen(draft: draft().copyWith(steps: [step('30', 0)])),
    );
    await tester.pump();
    expect(durationValue(), '305');

    // A nudge: new text at a new revision.
    editor.apply(
      EditorUpdateOpen(draft: draft().copyWith(steps: [step('320', 1)])),
    );
    await tester.pump();
    expect(durationValue(), '320');
    expect(find.text('320'), findsOneWidget);
  });

  testWidgets('a pour timed by Rust reloads the duration, not the grams', (
    tester,
  ) async {
    EditorStep step(String grams, String seconds, int revision) => EditorStep(
      id: 's1',
      nameLabel: 'Step 1 name',
      name: 'Pour',
      durationText: seconds,
      durationLabel: '00:00',
      durationRevision: revision,
      waterText: grams,
    );
    String? value(String label) => tester
        .state<FormFieldState<String>>(
          find.widgetWithText(TextFormField, label),
        )
        .value;

    await pumpEditor(tester);
    editor.apply(
      EditorUpdateOpen(draft: draft().copyWith(steps: [step('', '0', 0)])),
    );
    await tester.pump();
    expect(value('Pour speed (g/s)'), '5');

    // Typing the grams; Rust answers with the pour's time at a new revision.
    tester
        .state<FormFieldState<String>>(
          find.widgetWithText(TextFormField, 'Pour (g)'),
        )
        .didChange('250');
    editor.apply(
      EditorUpdateOpen(draft: draft().copyWith(steps: [step('250', '50', 1)])),
    );
    await tester.pump();
    expect(value('Duration (seconds)'), '50');
    expect(value('Pour (g)'), '250');
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
      find.text('Delete "V60"?'),
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
