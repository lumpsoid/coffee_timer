import 'dart:async';

import 'package:coffee_timer/app/cells.dart';
import 'package:coffee_timer/domain/recipe/brew_metrics.dart';
import 'package:coffee_timer/domain/recipe/recipe.dart';
import 'package:coffee_timer/ui/navigation/route_args.dart';
import 'package:coffee_timer/ui/recipe_edit/recipe_edit_page.dart';
import 'package:coffee_timer/ui/recipe_list/legacy_recipe_nav.dart';
import 'package:coffee_timer/ui/recipe_list/recipe_list_controller.dart';
import 'package:coffee_timer/ui/recipe_list/recipe_summary_tile.dart';
import 'package:coffee_timer/ui/settings/settings_page.dart';
import 'package:flutter/material.dart';

class RecipeListPage extends StatefulWidget {
  const RecipeListPage({super.key});

  static MaterialPageRoute<void> route() =>
      MaterialPageRoute(builder: (_) => const RecipeListPage());

  @override
  State<RecipeListPage> createState() => _RecipeListPageState();
}

class _RecipeListPageState extends State<RecipeListPage> {
  static const _metrics = BrewMetrics();

  late final RecipeListController _controller;

  @override
  void initState() {
    super.initState();
    _controller = Cells.recipeModule.value.newListController();
    unawaited(_controller.load());
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  /// Refreshes the list after returning from the legacy detail/edit pages,
  /// which persist through the bus. Guarded because the legacy edit page can
  /// pop this page off the stack (its delete flow does `popUntil`), disposing
  /// the controller before the awaited navigation future completes.
  void _reloadIfMounted() {
    if (!mounted) return;
    unawaited(_controller.load());
  }

  Future<void> _openDetails(Recipe recipe) async {
    await LegacyRecipeNav.openDetails(context, RecipeDetailsArgs(recipe.id));
    _reloadIfMounted();
  }

  Future<void> _openCreate() async {
    await Navigator.of(
      context,
    ).push(RecipeEditPage.route(const RecipeEditArgs.create()));
    _reloadIfMounted();
  }

  Future<void> _openEdit(Recipe recipe) async {
    await Navigator.of(
      context,
    ).push(RecipeEditPage.route(RecipeEditArgs.edit(recipe)));
    _reloadIfMounted();
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
                unawaited(Navigator.of(context).push(SettingsPage.route())),
            icon: const Icon(Icons.settings),
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton(
        tooltip: 'New recipe',
        onPressed: () => unawaited(_openCreate()),
        child: const Icon(Icons.add),
      ),
      body: ListenableBuilder(
        listenable: _controller,
        builder: (context, _) {
          final error = _controller.error;
          if (error != null) {
            return Center(child: Text(error));
          }
          if (_controller.isLoading && _controller.recipes.isEmpty) {
            return const Center(child: CircularProgressIndicator());
          }
          final recipes = _controller.recipes;
          return ListView.builder(
            itemCount: recipes.length,
            itemBuilder: (_, index) {
              final r = recipes[index];
              return RecipeSummaryTile(
                name: r.name.value,
                description: r.description,
                coffeeGrams: _metrics.coffeeGrams(r),
                waterGrams: _metrics.waterGrams(r),
                duration: _metrics.durationLabel(r),
                onTap: () => unawaited(_openDetails(r)),
                onLongPress: () => _showRecipeOptionsSheet(r),
              );
            },
          );
        },
      ),
    );
  }

  void _showRecipeOptionsSheet(Recipe recipe) {
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
                    unawaited(_openEdit(recipe));
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
                    _confirmDelete(recipe);
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

  void _confirmDelete(Recipe recipe) {
    unawaited(
      showDialog<void>(
        context: context,
        builder: (ctx) => AlertDialog(
          title: const Text('Delete recipe'),
          content: Text(
            'Delete "${recipe.name.value}"? This action cannot be undone.',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: const Text('Cancel'),
            ),
            TextButton(
              onPressed: () {
                Navigator.of(ctx).pop();
                unawaited(_controller.delete(recipe.id));
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
