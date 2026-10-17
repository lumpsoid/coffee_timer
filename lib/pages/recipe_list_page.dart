import 'dart:async';

import 'package:coffee_timer/holders/brew_holder.dart';
import 'package:coffee_timer/holders/editor_holder.dart';
import 'package:coffee_timer/holders/recipe_list_holder.dart';
import 'package:coffee_timer/holders/settings_holder.dart';
import 'package:coffee_timer/pages/brew_page.dart';
import 'package:coffee_timer/pages/editor_page.dart';
import 'package:coffee_timer/pages/settings_page.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/recipe_summary_tile.dart';
import 'package:flutter/material.dart';

/// The home page: every recipe, opening its brew page on tap and offering
/// edit / delete on long press. A failed delete shows as a snackbar.
class RecipeListPage extends StatefulWidget {
  const RecipeListPage({
    required this.recipes,
    required this.brew,
    required this.editor,
    required this.settings,
    super.key,
  });

  final RecipeListHolder recipes;
  final BrewHolder brew;
  final EditorHolder editor;
  final SettingsHolder settings;

  @override
  State<RecipeListPage> createState() => _RecipeListPageState();
}

class _RecipeListPageState extends State<RecipeListPage> {
  late final StreamSubscription<String> _failures;

  RecipeListHolder get recipes => widget.recipes;
  BrewHolder get brew => widget.brew;
  EditorHolder get editor => widget.editor;
  SettingsHolder get settings => widget.settings;

  @override
  void initState() {
    super.initState();
    _failures = recipes.failures.listen(_showFailure);
  }

  void _showFailure(String message) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }

  @override
  void dispose() {
    unawaited(_failures.cancel());
    super.dispose();
  }

  void _openBrew(BuildContext context, RecipeSummary recipe) {
    Navigator.of(context).push(
      BrewPage.route(recipe.id, brew: brew, editor: editor, recipes: recipes),
    );
  }

  void _openCreate(BuildContext context) {
    Navigator.of(context).push(
      EditorPage.newRecipe(editor: editor, recipes: recipes),
    );
  }

  void _openEdit(BuildContext context, RecipeSummary recipe) {
    Navigator.of(context).push(
      EditorPage.edit(recipe.id, editor: editor, recipes: recipes),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Recipes'),
        actions: [
          IconButton(
            tooltip: 'Settings',
            onPressed: () =>
                Navigator.of(context).push(SettingsPage.route(settings)),
            icon: const Icon(Icons.settings),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton(
        tooltip: 'New recipe',
        onPressed: () => _openCreate(context),
        child: const Icon(Icons.add),
      ),
      body: ListenableBuilder(
        listenable: recipes,
        builder: (context, _) {
          final rows = recipes.recipes ?? const <RecipeSummary>[];
          return ListView.builder(
            itemCount: rows.length,
            itemBuilder: (_, index) {
              final r = rows[index];
              return RecipeSummaryTile(
                name: r.name,
                description: r.description,
                coffeeLabel: r.coffeeLabel,
                waterLabel: r.waterLabel,
                timeLabel: r.timeLabel,
                label: r.label,
                onTap: () => _openBrew(context, r),
                onLongPress: () => _showRecipeOptionsSheet(context, r),
              );
            },
          );
        },
      ),
    );
  }

  void _showRecipeOptionsSheet(BuildContext context, RecipeSummary recipe) {
    unawaited(
      showModalBottomSheet<void>(
        context: context,
        builder: (ctx) {
          return SafeArea(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                ListTile(
                  leading: const Icon(Icons.edit),
                  title: const Text('Edit'),
                  onTap: () {
                    Navigator.of(ctx).pop();
                    _openEdit(context, recipe);
                  },
                ),
                ListTile(
                  leading: const Icon(Icons.delete, color: Colors.red),
                  title: const Text(
                    'Delete',
                    style: TextStyle(color: Colors.red),
                  ),
                  onTap: () {
                    Navigator.of(ctx).pop();
                    _confirmDelete(context, recipe);
                  },
                ),
                ListTile(
                  leading: const Icon(Icons.close),
                  title: const Text('Cancel'),
                  onTap: () => Navigator.of(ctx).pop(),
                ),
              ],
            ),
          );
        },
      ),
    );
  }

  void _confirmDelete(BuildContext context, RecipeSummary recipe) {
    unawaited(
      showDialog<void>(
        context: context,
        builder: (ctx) => AlertDialog(
          title: const Text('Delete recipe'),
          content: Text(
            'Delete "${recipe.name}"? This action cannot be undone.',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: const Text('Cancel'),
            ),
            TextButton(
              onPressed: () {
                Navigator.of(ctx).pop();
                RemoveRecipe(recipeId: recipe.id).sendSignalToRust();
              },
              child: const Text(
                'Delete',
                style: TextStyle(color: Colors.red),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
