import 'dart:async';

import 'package:coffee_timer/holders/brew_holder.dart';
import 'package:coffee_timer/holders/editor_holder.dart';
import 'package:coffee_timer/holders/recipe_list_holder.dart';
import 'package:coffee_timer/holders/settings_holder.dart';
import 'package:coffee_timer/holders/undo_offer.dart';
import 'package:coffee_timer/pages/brew_page.dart';
import 'package:coffee_timer/pages/editor_page.dart';
import 'package:coffee_timer/pages/settings_page.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/recipe_summary_tile.dart';
import 'package:flutter/material.dart';

/// The home page: every recipe, in the order Rust lists them (pinned
/// first), opening its brew page on tap and offering edit / duplicate /
/// pin or unpin / delete on long press. A delete, here or in the editor, shows a
/// snackbar with an Undo when Rust offers one; a failed delete or undo shows
/// as a snackbar too.
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

  /// How long an undo offer stays up: the visual duration only. Rust keeps
  /// the removed recipe until the next delete either way.
  static const undoDuration = Duration(seconds: 5);

  @override
  State<RecipeListPage> createState() => _RecipeListPageState();
}

class _RecipeListPageState extends State<RecipeListPage> {
  late final StreamSubscription<String> _failures;
  late final StreamSubscription<UndoOffer> _undoOffers;

  RecipeListHolder get recipes => widget.recipes;
  BrewHolder get brew => widget.brew;
  EditorHolder get editor => widget.editor;
  SettingsHolder get settings => widget.settings;

  @override
  void initState() {
    super.initState();
    _failures = recipes.failures.listen(_showFailure);
    _undoOffers = recipes.undoOffers.listen(_offerUndo);
  }

  void _showFailure(String message) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }

  void _offerUndo(UndoOffer offer) {
    ScaffoldMessenger.of(context)
      ..hideCurrentSnackBar()
      ..showSnackBar(
        SnackBar(
          content: Text(offer.notice),
          duration: RecipeListPage.undoDuration,
          // An action would keep it up until dismissed; the offer times out.
          persist: false,
          action: SnackBarAction(
            label: 'Undo',
            onPressed: () => UndoRemoveRecipe(
              recipeId: offer.recipeId,
            ).sendSignalToRust(),
          ),
        ),
      );
  }

  @override
  void dispose() {
    unawaited(_failures.cancel());
    unawaited(_undoOffers.cancel());
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

  void _openDuplicate(BuildContext context, RecipeSummary recipe) {
    Navigator.of(context).push(
      EditorPage.duplicate(recipe.id, editor: editor, recipes: recipes),
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
                pinned: r.pinned,
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
                  leading: const Icon(Icons.copy),
                  title: const Text('Duplicate'),
                  onTap: () {
                    Navigator.of(ctx).pop();
                    _openDuplicate(context, recipe);
                  },
                ),
                ListTile(
                  leading: Icon(
                    recipe.pinned ? Icons.push_pin_outlined : Icons.push_pin,
                  ),
                  title: Text(recipe.pinned ? 'Unpin' : 'Pin'),
                  onTap: () {
                    Navigator.of(ctx).pop();
                    if (recipe.pinned) {
                      UnpinRecipe(recipeId: recipe.id).sendSignalToRust();
                    } else {
                      PinRecipe(recipeId: recipe.id).sendSignalToRust();
                    }
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
            'Delete "${recipe.name}"?',
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
