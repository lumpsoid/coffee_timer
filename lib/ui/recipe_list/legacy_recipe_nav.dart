import 'package:coffee_timer/app/cells.dart';
import 'package:coffee_timer/core/di.dart';
import 'package:coffee_timer/core/recipe/recipe.dart' as core;
import 'package:coffee_timer/domain/recipe/recipe.dart';
import 'package:coffee_timer/recipe_details/pages/pages.dart';
import 'package:coffee_timer/recipe_edit/ui/ui.dart';
import 'package:coffee_timer/ui/navigation/route_args.dart';
import 'package:flutter/material.dart';

/// TEMPORARY strangler bridge (Phases 2–4).
///
/// The migrated recipe list navigates with [RouteArgs], but the detail and edit
/// pages it opens are still the legacy widgets that consume the old core
/// `Recipe` and the event bus. This file is the one place the new args are
/// adapted to that old contract; it is deleted when those pages migrate. See
/// `docs/pivot-spec.md`.
abstract final class LegacyRecipeNav {
  /// Opens the legacy detail page for [args]. Loads the recipe through the new
  /// repository, then selects it on the bus post-frame — the push is *not*
  /// awaited before scheduling the emit, so the callback runs on the new page's
  /// first frame (its stream subscription is live by then), matching the old
  /// list's contract. The returned future completes when the page is popped.
  static Future<void> openDetails(
    BuildContext context,
    RecipeDetailsArgs args,
  ) async {
    final recipe = await Cells.recipeModule.value.repository.byId(
      args.recipeId,
    );
    if (recipe == null || !context.mounted) return;

    final popped = Navigator.of(context).push(
      MaterialPageRoute<void>(builder: (_) => const RecipeDetailPage()),
    );
    WidgetsBinding.instance.addPostFrameCallback(
      (_) => kernel.emit('recipe', core.RecipeSelect(_toCore(recipe))),
    );
    await popped;
  }

  /// Opens the legacy edit page for [args] (create or edit). The returned
  /// future completes when the page is popped.
  static Future<void> openEdit(BuildContext context, RecipeEditArgs args) {
    final initial = args.initial;
    return Navigator.of(context).push<void>(
      initial == null
          ? RecipeEditPage.route()
          : RecipeEditPage.route(initial: _toCore(initial)),
    );
  }

  static core.Recipe _toCore(Recipe r) => core.Recipe(
    id: r.id.value,
    name: r.name.value,
    description: r.description,
    steps: [
      for (final s in r.steps)
        core.RecipeStep(
          id: s.id.value,
          name: s.name,
          durationSeconds: s.duration.seconds,
        ),
    ],
  );
}
