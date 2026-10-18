import 'package:coffee_timer/holders/brew_holder.dart';
import 'package:coffee_timer/holders/editor_holder.dart';
import 'package:coffee_timer/holders/recipe_list_holder.dart';
import 'package:coffee_timer/holders/settings_holder.dart';
import 'package:coffee_timer/pages/recipe_list_page.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/recipe_summary_tile.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

import '../fixtures.dart';

/// The list page over a real holder, driven by hand-built updates. Nothing
/// here sends a signal: the page only sends on user input.
void main() {
  late RecipeListHolder recipes;

  setUp(() {
    recipes = RecipeListHolder()
      ..apply(RecipeListUpdateSnapshot(recipes: [summary('a')]));
  });

  Future<void> pumpList(WidgetTester tester) => tester.pumpWidget(
    MaterialApp(
      home: RecipeListPage(
        recipes: recipes,
        brew: BrewHolder(),
        editor: EditorHolder(),
        settings: SettingsHolder(),
      ),
    ),
  );

  testWidgets('follows the holder', (tester) async {
    await pumpList(tester);
    expect(find.byType(RecipeSummaryTile), findsOneWidget);

    recipes.apply(RecipeListUpdateUpserted(recipe: summary('b'), index: 1));
    await tester.pump();
    expect(find.byType(RecipeSummaryTile), findsNWidgets(2));
  });

  testWidgets('a long press offers edit, duplicate and delete', (
    tester,
  ) async {
    await pumpList(tester);
    await tester.longPress(find.byType(RecipeSummaryTile));
    await tester.pumpAndSettle();
    expect(find.widgetWithText(ListTile, 'Edit'), findsOneWidget);
    expect(find.widgetWithText(ListTile, 'Duplicate'), findsOneWidget);
    expect(find.widgetWithText(ListTile, 'Delete'), findsOneWidget);
    expect(find.widgetWithText(ListTile, 'Cancel'), findsOneWidget);
  });

  testWidgets('offers an undo for a removal, then lets it go', (
    tester,
  ) async {
    await pumpList(tester);

    recipes.apply(
      const RecipeListUpdateRemoved(id: 'a', undoNotice: 'Deleted V60'),
    );
    await tester.pump();
    expect(find.byType(RecipeSummaryTile), findsNothing);
    expect(find.text('Deleted V60'), findsOneWidget);
    expect(find.widgetWithText(SnackBarAction, 'Undo'), findsOneWidget);

    await tester.pumpAndSettle(); // the entrance; the duration starts after
    await tester.pump(RecipeListPage.undoDuration);
    await tester.pumpAndSettle();
    expect(find.text('Deleted V60'), findsNothing);
  });

  testWidgets('a removal without an undo notice shows no snackbar', (
    tester,
  ) async {
    await pumpList(tester);
    recipes.apply(const RecipeListUpdateRemoved(id: 'a'));
    await tester.pump();
    expect(find.byType(SnackBar), findsNothing);
  });

  testWidgets('shows a snackbar for each failed delete, repeats included', (
    tester,
  ) async {
    await pumpList(tester);
    const message = 'Failed to delete recipe: Storage failed: disk full';

    recipes
      ..apply(const RecipeListUpdateFailed(message: message))
      ..apply(const RecipeListUpdateFailed(message: message));
    await tester.pump();
    expect(find.text(message), findsOneWidget);
    expect(find.byType(RecipeSummaryTile), findsOneWidget);

    final messenger = tester.state<ScaffoldMessengerState>(
      find.byType(ScaffoldMessenger),
    )..hideCurrentSnackBar();
    await tester.pumpAndSettle();
    expect(find.text(message), findsOneWidget);

    messenger.hideCurrentSnackBar();
    await tester.pumpAndSettle();
    expect(find.text(message), findsNothing);
  });
}
