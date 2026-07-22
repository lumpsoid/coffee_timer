import 'package:coffee_timer/app/cells.dart';
import 'package:coffee_timer/app/recipe_module.dart';
import 'package:coffee_timer/application/recipe/recipe_ports.dart';
import 'package:coffee_timer/domain/recipe/recipe.dart';
import 'package:coffee_timer/domain/recipe/recipe_id.dart';
import 'package:coffee_timer/infrastructure/recipe/uuid_recipe_ids.dart';
import 'package:coffee_timer/ui/recipe_list/recipe_list_page.dart';
import 'package:coffee_timer/ui/recipe_list/recipe_summary_tile.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';

/// In-memory stand-in for the SQLite repository. Widget tests mock plugins
/// (sqlite runs on isolates that don't settle under the test clock); the real
/// SQLite path is covered by `sqlite_recipe_repository_test.dart`. This keeps
/// the drive here on the *UI*: real pages, real navigation, real controllers.
class _InMemoryRecipeRepository
    implements RecipeReadRepository, RecipeWriteRepository {
  final Map<String, Recipe> store = {};

  @override
  Future<List<Recipe>> all() async => store.values.toList();

  @override
  Future<Recipe?> byId(RecipeId id) async => store[id.value];

  @override
  Future<Recipe> upsert(Recipe recipe) async {
    store[recipe.id.value] = recipe;
    return recipe;
  }

  @override
  Future<void> delete(RecipeId id) async {
    store.remove(id.value);
  }
}

void main() {
  late _InMemoryRecipeRepository repo;

  setUp(() {
    repo = _InMemoryRecipeRepository();
    Cells.recipeModule.value = RecipeModule(
      read: repo,
      write: repo,
      ids: const UuidRecipeIds(),
    );
  });

  RecipeSummaryTile onlyTile(WidgetTester tester) =>
      tester.widget<RecipeSummaryTile>(find.byType(RecipeSummaryTile));

  testWidgets('create, edit, then delete a recipe from the list', (
    tester,
  ) async {
    await tester.pumpWidget(const MaterialApp(home: RecipeListPage()));
    await tester.pumpAndSettle();
    expect(find.byType(RecipeSummaryTile), findsNothing);

    // --- CREATE ---
    await tester.tap(find.byTooltip('New recipe'));
    await tester.pumpAndSettle();
    expect(find.text('New Recipe'), findsOneWidget);

    // Fields in order: name, description, step-0 name, step-0 duration.
    final fields = find.byType(TextFormField);
    await tester.enterText(fields.at(0), 'Chemex');
    await tester.enterText(fields.at(2), 'Bloom');
    await tester.enterText(fields.at(3), '90');
    await tester.tap(find.byTooltip('Save'));
    await tester.pumpAndSettle();

    // Back on the list, and persisted.
    expect(find.byType(RecipeSummaryTile), findsOneWidget);
    expect(onlyTile(tester).name, 'Chemex');
    final created = repo.store.values.single;
    final recipeId = created.id;
    final stepId = created.steps.single.id;
    expect(created.name.value, 'Chemex');
    expect(created.steps.single.name, 'Bloom');
    expect(created.steps.single.duration.seconds, 90);

    // --- EDIT (long-press tile → Edit) ---
    await tester.longPress(find.byType(RecipeSummaryTile));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Edit'));
    await tester.pumpAndSettle();
    expect(find.text('Edit Recipe'), findsOneWidget);

    final editFields = find.byType(TextFormField);
    await tester.enterText(editFields.at(0), 'Chemex v2');
    await tester.enterText(editFields.at(3), '120');
    await tester.tap(find.byTooltip('Save'));
    await tester.pumpAndSettle();

    expect(onlyTile(tester).name, 'Chemex v2');
    final edited = repo.store.values.single;
    expect(edited.id, recipeId, reason: 'recipe id preserved on edit');
    expect(edited.steps.single.id, stepId, reason: 'step id preserved on edit');
    expect(edited.name.value, 'Chemex v2');
    expect(edited.steps.single.duration.seconds, 120);

    // --- DELETE (long-press tile → Edit → delete → confirm) ---
    await tester.longPress(find.byType(RecipeSummaryTile));
    await tester.pumpAndSettle();
    await tester.tap(find.text('Edit'));
    await tester.pumpAndSettle();
    await tester.tap(find.byTooltip('Delete')); // app-bar delete
    await tester.pumpAndSettle();
    expect(find.text('Delete recipe'), findsOneWidget);
    await tester.tap(find.widgetWithText(TextButton, 'Delete'));
    await tester.pumpAndSettle();

    expect(repo.store, isEmpty);
    expect(find.byType(RecipeSummaryTile), findsNothing);
  });

  testWidgets('a blank name blocks save and shows the field error', (
    tester,
  ) async {
    await tester.pumpWidget(const MaterialApp(home: RecipeListPage()));
    await tester.pumpAndSettle();

    await tester.tap(find.byTooltip('New recipe'));
    await tester.pumpAndSettle();

    // Give the one step a valid duration but leave the name blank.
    await tester.enterText(find.byType(TextFormField).at(3), '30');
    await tester.tap(find.byTooltip('Save'));
    await tester.pumpAndSettle();

    // Still on the editor, nothing written, error shown.
    expect(find.text('Edit Recipe'), findsNothing);
    expect(find.text('New Recipe'), findsOneWidget);
    expect(find.text('Enter a recipe name'), findsOneWidget);
    expect(repo.store, isEmpty);
  });
}
