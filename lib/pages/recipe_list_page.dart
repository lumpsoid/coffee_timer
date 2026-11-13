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
import 'package:flutter/services.dart';
import 'package:material_ui/material_ui.dart';

/// The home page: every recipe, in the order Rust lists them (pinned
/// first), opening its brew page on tap and offering edit / duplicate /
/// pin or unpin / export / delete on long press. The app bar's menu
/// exports every recipe to the clipboard, imports the clipboard's text
/// (Rust copies, validates and adds), or enters reorder mode: the pinned
/// recipes stand as one fixed card, each unpinned card gets a drag handle
/// and a drop asks Rust for the move, until Done. A
/// delete, here or in the editor, shows a snackbar with an Undo when Rust
/// offers one; a failure, and an export's or import's notice, shows as a
/// snackbar too.
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
  late final StreamSubscription<String> _notices;
  late final StreamSubscription<UndoOffer> _undoOffers;

  /// Whether the cards show drag handles instead of opening on tap or long
  /// press.
  bool _reordering = false;

  /// The order shown after a drop, over the holder's list it was made
  /// from; the holder's next list, with Rust's order, replaces it.
  ({List<RecipeSummary>? over, List<RecipeSummary> rows})? _moved;

  RecipeListHolder get recipes => widget.recipes;
  BrewHolder get brew => widget.brew;
  EditorHolder get editor => widget.editor;
  SettingsHolder get settings => widget.settings;

  @override
  void initState() {
    super.initState();
    _failures = recipes.failures.listen(_showMessage);
    _notices = recipes.notices.listen(_showMessage);
    _undoOffers = recipes.undoOffers.listen(_offerUndo);
  }

  void _showMessage(String message) {
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
    unawaited(_notices.cancel());
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

  /// Sends the clipboard's text to Rust as it is; no text is empty text,
  /// which Rust refuses (`Can't import: The text is empty`).
  Future<void> _importFromClipboard() async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    ImportRecipes(text: data?.text ?? '').sendSignalToRust();
  }

  /// The holder's rows, or the order shown after a drop until Rust's
  /// answer replaces the holder's list.
  List<RecipeSummary> _rows() {
    final held = recipes.recipes;
    final moved = _moved;
    if (moved != null && identical(moved.over, held)) return moved.rows;
    return held ?? const <RecipeSummary>[];
  }

  /// Shows the dropped card in its new place at once, then asks Rust for the
  /// move. Waiting for Rust's list would show the old order for a frame or
  /// more after the drop; Rust's list still replaces this one when it
  /// comes, with its order (a recipe stays among those that share its pin)
  /// and any refusal.
  void _moveRecipe(List<RecipeSummary> rows, int from, int to) {
    final moved = [...rows];
    final recipe = moved.removeAt(from);
    moved.insert(to, recipe);
    setState(() => _moved = (over: recipes.recipes, rows: moved));
    MoveRecipe(recipeId: recipe.id, to: to).sendSignalToRust();
  }

  void _setReordering(bool reordering) =>
      setState(() => _reordering = reordering);

  @override
  Widget build(BuildContext context) {
    return PopScope(
      // Back leaves reorder mode rather than the app.
      canPop: !_reordering,
      onPopInvokedWithResult: (didPop, _) {
        if (!didPop) _setReordering(false);
      },
      child: _reordering ? _reorderScaffold() : _listScaffold(context),
    );
  }

  Widget _reorderScaffold() {
    return Scaffold(
      appBar: AppBar(
        title: const Text('REORDER'),
        actions: [
          TextButton(
            onPressed: () => _setReordering(false),
            child: const Text('DONE'),
          ),
        ],
      ),
      body: ListenableBuilder(
        listenable: recipes,
        builder: (context, _) {
          // Rust lists pinned recipes first and moves a recipe only among
          // those that share its pin: the pinned stand as one fixed card,
          // and only the unpinned can be dragged.
          final rows = _rows();
          final pinned = rows.where((r) => r.pinned).length;
          final unpinned = rows.sublist(pinned);
          return ReorderableListView.builder(
            buildDefaultDragHandles: false,
            header: pinned == 0 ? null : _PinnedCard(count: pinned),
            itemCount: unpinned.length,
            onReorderItem: (from, to) =>
                _moveRecipe(rows, pinned + from, pinned + to),
            // The dragged card is its own paper; the default lifts it on
            // a rectangle of Material around the row's padding.
            proxyDecorator: (child, _, _) => Material(
              type: MaterialType.transparency,
              child: child,
            ),
            itemBuilder: (_, index) {
              final r = unpinned[index];
              return Row(
                key: ValueKey(r.id),
                children: [
                  ReorderableDragStartListener(
                    index: index,
                    child: const Padding(
                      padding: EdgeInsets.only(left: 16),
                      child: Icon(
                        Icons.drag_handle,
                        semanticLabel: 'Drag to reorder recipe',
                      ),
                    ),
                  ),
                  Expanded(child: _tile(r)),
                ],
              );
            },
          );
        },
      ),
    );
  }

  Widget _listScaffold(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('RECIPES'),
        actions: [
          IconButton(
            tooltip: 'Settings',
            onPressed: () =>
                Navigator.of(context).push(SettingsPage.route(settings)),
            icon: const Icon(Icons.settings_outlined),
          ),
          PopupMenuButton<_ListAction>(
            tooltip: 'More',
            onSelected: (action) => switch (action) {
              _ListAction.exportAll => const ExportRecipes(
                recipeIds: [],
              ).sendSignalToRust(),
              _ListAction.import => unawaited(_importFromClipboard()),
              _ListAction.reorder => _setReordering(true),
            },
            itemBuilder: (_) => const [
              PopupMenuItem(
                value: _ListAction.reorder,
                child: Text('Reorder recipes'),
              ),
              PopupMenuItem(
                value: _ListAction.exportAll,
                child: Text('Export all to clipboard'),
              ),
              PopupMenuItem(
                value: _ListAction.import,
                child: Text('Import from clipboard'),
              ),
            ],
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        tooltip: 'New recipe',
        onPressed: () => _openCreate(context),
        icon: const Icon(Icons.add),
        label: const Text('NEW RECIPE'),
      ),
      body: ListenableBuilder(
        listenable: recipes,
        builder: (context, _) {
          final rows = _rows();
          return ListView.builder(
            itemCount: rows.length,
            // A row that moved keeps its widgets (and its paper snapshot).
            findChildIndexCallback: (key) {
              final index = rows.indexWhere((r) => ValueKey(r.id) == key);
              return index < 0 ? null : index;
            },
            itemBuilder: (_, index) {
              final r = rows[index];
              return _tile(
                r,
                key: ValueKey(r.id),
                onTap: () => _openBrew(context, r),
                onLongPress: () => _showRecipeOptionsSheet(context, r),
              );
            },
          );
        },
      ),
    );
  }

  /// One recipe's card; without callbacks it ignores taps.
  Widget _tile(
    RecipeSummary r, {
    Key? key,
    VoidCallback? onTap,
    VoidCallback? onLongPress,
  }) => RecipeSummaryTile(
    key: key,
    name: r.name,
    description: r.description,
    coffeeLabel: r.coffeeLabel,
    waterLabel: r.waterLabel,
    timeLabel: r.timeLabel,
    grindLabel: r.grindLabel,
    label: r.label,
    pinned: r.pinned,
    onTap: onTap,
    onLongPress: onLongPress,
  );

  void _showRecipeOptionsSheet(BuildContext context, RecipeSummary recipe) {
    showModalBottomSheet<void>(
      context: context,
      builder: (ctx) {
        return SafeArea(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              ListTile(
                leading: const Icon(Icons.edit_outlined),
                title: const Text('Edit'),
                onTap: () {
                  Navigator.of(ctx).pop();
                  _openEdit(context, recipe);
                },
              ),
              ListTile(
                leading: const Icon(Icons.copy_outlined),
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
                leading: const Icon(Icons.ios_share),
                title: const Text('Export'),
                onTap: () {
                  Navigator.of(ctx).pop();
                  ExportRecipes(recipeIds: [recipe.id]).sendSignalToRust();
                },
              ),
              ListTile(
                leading: const Icon(Icons.delete_outline, color: Colors.red),
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
    );
  }

  void _confirmDelete(BuildContext context, RecipeSummary recipe) {
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
    );
  }
}

/// The app bar menu's entries.
enum _ListAction { reorder, exportAll, import }

/// Reorder mode's stand-in for the pinned recipes: they stay on top, in
/// their own order, and cannot be dragged. About as tall as a recipe card
/// without a description, so the list does not jump between modes.
class _PinnedCard extends StatelessWidget {
  const _PinnedCard({required this.count});

  final int count;

  /// The paper height of a [RecipeSummaryTile] without a description,
  /// approximately: its padding, badge, rule and spec row.
  static const height = 132.0;

  @override
  Widget build(BuildContext context) {
    return Card(
      // The recipe card's own inset around its paper.
      margin: const EdgeInsets.symmetric(vertical: 6, horizontal: 16),
      child: SizedBox(
        height: height,
        child: Center(
          child: ListTile(
            leading: const Icon(Icons.push_pin),
            title: Text(
              count == 1 ? '1 pinned recipe' : '$count pinned recipes',
            ),
            subtitle: const Text('Pinned recipes stay on top'),
          ),
        ),
      ),
    );
  }
}
