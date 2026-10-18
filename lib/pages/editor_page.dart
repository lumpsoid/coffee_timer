import 'dart:async';

import 'package:coffee_timer/holders/editor_holder.dart';
import 'package:coffee_timer/holders/recipe_list_holder.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter/material.dart';

/// The recipe editor. Once subscribed it sends its opening intent ([open]:
/// [NewDraft] or [EditRecipe]) and shows the drafts that follow. Every
/// keystroke and button is an intent; labels, errors and the total come
/// back in the draft.
///
/// The page pops when Rust sends `Closed` (after a save or delete, or in
/// answer to [CloseEditor] when the user backs out), so Rust decides when
/// the editor is done. A draft's `notice` shows as a snackbar each time one
/// arrives.
class EditorPage extends StatefulWidget {
  const EditorPage({
    required this.editor,
    required this.recipes,
    required this.open,
    super.key,
  });

  final EditorHolder editor;
  final RecipeListHolder recipes;

  /// Sends the intent that opens this page's draft.
  final VoidCallback open;

  /// The editor on a new draft.
  static MaterialPageRoute<void> newRecipe({
    required EditorHolder editor,
    required RecipeListHolder recipes,
  }) => MaterialPageRoute(
    builder: (_) => EditorPage(
      editor: editor,
      recipes: recipes,
      open: () => const NewDraft().sendSignalToRust(),
    ),
  );

  /// The editor on a draft of the saved recipe with [recipeId].
  static MaterialPageRoute<void> edit(
    String recipeId, {
    required EditorHolder editor,
    required RecipeListHolder recipes,
  }) => MaterialPageRoute(
    builder: (_) => EditorPage(
      editor: editor,
      recipes: recipes,
      open: () => EditRecipe(recipeId: recipeId).sendSignalToRust(),
    ),
  );

  @override
  State<EditorPage> createState() => _EditorPageState();
}

class _EditorPageState extends State<EditorPage> {
  /// The latest draft since this page opened; null until Rust answers
  /// [EditorPage.open], so a draft left from before is never shown.
  EditorDraft? _draft;

  @override
  void initState() {
    super.initState();
    widget.editor.addListener(_onUpdate);
    widget.open();
  }

  @override
  void dispose() {
    widget.editor.removeListener(_onUpdate);
    super.dispose();
  }

  /// Reacts to each update that arrives while the page is open. On
  /// `Closed` the page stops listening and removes itself.
  void _onUpdate() {
    switch (widget.editor.state) {
      case EditorUpdateClosed():
        widget.editor.removeListener(_onUpdate);
        _removeOwnRoute();
      case EditorUpdateOpen(:final draft):
        setState(() => _draft = draft);
        final notice = draft.notice;
        if (notice != null) {
          ScaffoldMessenger.of(
            context,
          ).showSnackBar(SnackBar(content: Text(notice)));
        }
    }
  }

  /// Takes this page's own route away: an animated pop when it is on top,
  /// otherwise a removal, so a dialog over the page (the delete
  /// confirmation) is never what goes.
  void _removeOwnRoute() {
    final route = ModalRoute.of(context);
    if (route == null) return;
    final navigator = Navigator.of(context);
    if (route.isCurrent) {
      navigator.pop();
    } else {
      navigator.removeRoute(route);
    }
  }

  @override
  Widget build(BuildContext context) {
    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, _) {
        // A pop Rust asked for has happened already; a back gesture asks.
        if (!didPop) const CloseEditor().sendSignalToRust();
      },
      child: switch (_draft) {
        final EditorDraft draft => _page(draft),
        null => const Scaffold(
          body: Center(child: CircularProgressIndicator()),
        ),
      },
    );
  }

  Widget _page(EditorDraft draft) {
    return Scaffold(
      appBar: AppBar(
        title: Text(draft.title),
        actions: [
          if (draft.canDelete)
            IconButton(
              tooltip: 'Delete',
              onPressed: () => _confirmDelete(draft.id),
              icon: const Icon(Icons.delete),
            ),
          IconButton(
            tooltip: 'Save',
            onPressed: () => const SaveRecipe().sendSignalToRust(),
            icon: const Icon(Icons.save),
          ),
        ],
      ),
      body: ListView(
        // A new draft gets fresh fields; within a draft, fields keep their
        // own text while Rust echoes it back.
        key: ValueKey(draft.id),
        padding: const EdgeInsets.all(16),
        children: [
          TextFormField(
            initialValue: draft.name,
            decoration: InputDecoration(
              labelText: 'Recipe name',
              errorText: draft.nameError,
            ),
            onChanged: (raw) => SetRecipeName(raw: raw).sendSignalToRust(),
          ),
          const SizedBox(height: 12),
          TextFormField(
            initialValue: draft.description,
            decoration: const InputDecoration(labelText: 'Description'),
            maxLines: 3,
            onChanged: (raw) =>
                SetRecipeDescription(raw: raw).sendSignalToRust(),
          ),
          const SizedBox(height: 12),
          Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(
                child: TextFormField(
                  initialValue: draft.coffee,
                  decoration: InputDecoration(
                    labelText: 'Coffee (g)',
                    errorText: draft.coffeeError,
                  ),
                  keyboardType: TextInputType.number,
                  onChanged: (raw) =>
                      SetRecipeCoffee(raw: raw).sendSignalToRust(),
                ),
              ),
              const SizedBox(width: 12),
              Expanded(
                child: TextFormField(
                  initialValue: draft.water,
                  decoration: InputDecoration(
                    labelText: 'Water (g)',
                    errorText: draft.waterError,
                  ),
                  keyboardType: TextInputType.number,
                  onChanged: (raw) =>
                      SetRecipeWater(raw: raw).sendSignalToRust(),
                ),
              ),
            ],
          ),
          const SizedBox(height: 20),
          Text('Total duration: ${draft.totalLabel}'),
          const SizedBox(height: 20),
          const Text('Steps', style: TextStyle(fontSize: 16)),
          const SizedBox(height: 8),
          for (final step in draft.steps)
            _StepCard(key: ValueKey(step.id), step: step),
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 8),
            child: Wrap(
              runSpacing: 4,
              spacing: 4,
              children: [
                for (final button in draft.addStepButtons)
                  TextButton.icon(
                    onPressed: () =>
                        AddStep(preset: button.preset).sendSignalToRust(),
                    icon: const Icon(Icons.add),
                    label: Text(button.label),
                  ),
              ],
            ),
          ),
          const SizedBox(height: 20),
          ElevatedButton(
            onPressed: () => const SaveRecipe().sendSignalToRust(),
            child: const Text('Save Recipe'),
          ),
        ],
      ),
    );
  }

  /// Asks before deleting, naming the recipe as saved, not as typed.
  void _confirmDelete(String recipeId) {
    final saved = widget.recipes.byId(recipeId);
    if (saved == null) return;
    unawaited(
      showDialog<void>(
        context: context,
        builder: (ctx) => AlertDialog(
          title: const Text('Delete recipe'),
          content: Text(
            'Delete "${saved.name}"? This action cannot be undone.',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: const Text('Cancel'),
            ),
            TextButton(
              onPressed: () {
                Navigator.of(ctx).pop();
                const DeleteRecipe().sendSignalToRust();
              },
              child: const Text('Delete', style: TextStyle(color: Colors.red)),
            ),
          ],
        ),
      ),
    );
  }
}

/// One step's name and duration inputs, keyed by its row id so its fields
/// keep their text as other rows come and go.
class _StepCard extends StatelessWidget {
  const _StepCard({required this.step, super.key});

  final EditorStep step;

  @override
  Widget build(BuildContext context) {
    return Card(
      margin: const EdgeInsets.symmetric(vertical: 8),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          children: [
            Row(
              children: [
                Expanded(
                  child: TextFormField(
                    initialValue: step.name,
                    decoration: InputDecoration(labelText: step.nameLabel),
                    onChanged: (raw) => SetStepName(
                      row: step.id,
                      raw: raw,
                    ).sendSignalToRust(),
                  ),
                ),
                const SizedBox(width: 12),
                IconButton(
                  tooltip: 'Remove step',
                  onPressed: () => RemoveStep(row: step.id).sendSignalToRust(),
                  icon: const Icon(Icons.delete, color: Colors.red),
                ),
              ],
            ),
            const SizedBox(height: 8),
            TextFormField(
              initialValue: step.durationText,
              decoration: InputDecoration(
                labelText: 'Duration (seconds)',
                helperText: 'MM:SS — ${step.durationLabel}',
                errorText: step.durationError,
              ),
              keyboardType: TextInputType.number,
              onChanged: (raw) => SetStepDuration(
                row: step.id,
                raw: raw,
              ).sendSignalToRust(),
            ),
          ],
        ),
      ),
    );
  }
}
