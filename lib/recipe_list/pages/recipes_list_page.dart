import 'dart:async';

import 'package:coffee_timer/core/recipe/recipe.dart';
import 'package:coffee_timer/recipe_details/pages/pages.dart';
import 'package:coffee_timer/recipe_edit/ui/ui.dart';
import 'package:flutter/material.dart';

class RecipeListPage extends StatefulWidget {
  const RecipeListPage({super.key});

  static MaterialPageRoute<void> route() {
    return MaterialPageRoute(
      builder: (context) => const RecipeListPage(),
    );
  }

  @override
  State<RecipeListPage> createState() => _RecipeListPageState();
}

class _RecipeListPageState extends State<RecipeListPage> {
  final RecipeEventBus _bus = RecipeHub.instance.bus;
  late final StreamSubscription<RecipeEvent> _sub;
  List<Recipe> _recipes = [];

  @override
  void initState() {
    super.initState();

    _sub = _bus.stream.listen((event) {
      switch (event) {
        case RecipesLoaded():
          setState(() {
            _recipes = event.recipes;
          });

        case RecipeDeleted():
          final updatedRecipes = _recipes;
          updatedRecipes.removeWhere((r) => r.id == event.id);
          setState(() {
            _recipes = updatedRecipes;
          });

        case RecipeModified():
          final oldRecipes = _recipes;
          final indexRecipe = oldRecipes.indexWhere(
            (recipe) => recipe.id == event.recipe.id,
          );
          // new recipe
          if (indexRecipe == -1) {
            oldRecipes.add(event.recipe);
            setState(() {
              _recipes = oldRecipes;
            });
          } else {
            // old one modified
            oldRecipes[indexRecipe] = event.recipe;
            setState(() {
              _recipes = oldRecipes;
            });
          }
        default:
          return;
      }
    });

    _bus.fire(const RecipesLoad());
  }

  @override
  void dispose() {
    unawaited(_sub.cancel());
    super.dispose();
  }

  void _onRecipeTap(Recipe recipe) {
    WidgetsBinding.instance.addPostFrameCallback((_) {
      unawaited(
        Navigator.push(
          context,
          MaterialPageRoute<void>(builder: (_) => const RecipeDetailPage()),
        ),
      );
      WidgetsBinding.instance.addPostFrameCallback(
        (_) => _bus.fire(RecipeSelect(recipe)),
      );
    });
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Recipes')),
      floatingActionButton: FloatingActionButton(
        tooltip: 'New recipe',
        onPressed: () =>
            Navigator.of(
              context,
            ).push(
              RecipeEditPage.route(),
            ),
        child: const Icon(Icons.add),
      ),
      body: ListView.builder(
        itemCount: _recipes.length,
        itemBuilder: (_, index) {
          final r = _recipes[index];
          return ListTile(
            title: Text(r.name),
            subtitle: Text(r.description),
            onTap: () => _onRecipeTap(r),
            onLongPress: () => _showRecipeOptionsSheet(context, r),
          );
        },
      ),
    );
  }

  void _showRecipeOptionsSheet(BuildContext context, Recipe recipe) {
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
                    Navigator.of(ctx).pop(); // close sheet
                    unawaited(
                      Navigator.of(
                        context,
                      ).push(RecipeEditPage.route(initial: recipe)),
                    );
                  },
                ),
                ListTile(
                  leading: const Icon(Icons.delete, color: Colors.red),
                  title: const Text(
                    'Delete',
                    style: TextStyle(color: Colors.red),
                  ),
                  onTap: () {
                    Navigator.of(ctx).pop(); // close sheet
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

  void _confirmDelete(BuildContext context, Recipe recipe) {
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
                Navigator.of(ctx).pop(); // close dialog
                _bus.fire(RecipeDelete(id: recipe.id));
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
