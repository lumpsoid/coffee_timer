import 'package:coffee_timer/holders/editor_holder.dart';
import 'package:coffee_timer/holders/recipe_list_holder.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_parts.dart';
import 'package:coffee_timer/widgets/texture_gallery.dart';
import 'package:material_ui/material_ui.dart';

/// The recipe editor. Once subscribed it sends its opening intent ([open]:
/// [NewDraft], [EditRecipe] or [DuplicateRecipe]) and shows the drafts that
/// follow. Every keystroke, button and step drag is an intent; labels,
/// errors, the total and the step order come back in the draft.
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

  /// The editor on a new draft copied from the saved recipe with
  /// [recipeId]; Rust loads the recipe and names the copy.
  static MaterialPageRoute<void> duplicate(
    String recipeId, {
    required EditorHolder editor,
    required RecipeListHolder recipes,
  }) => MaterialPageRoute(
    builder: (_) => EditorPage(
      editor: editor,
      recipes: recipes,
      open: () => DuplicateRecipe(recipeId: recipeId).sendSignalToRust(),
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
        title: Text(draft.title.toUpperCase()),
        actions: [
          if (draft.canDelete)
            IconButton(
              tooltip: 'Delete',
              onPressed: () => _confirmDelete(draft.id),
              icon: const Icon(Icons.delete_outline),
            ),
          IconButton(
            tooltip: 'Save',
            onPressed: () => const SaveRecipe().sendSignalToRust(),
            icon: const Icon(Icons.save_outlined),
          ),
        ],
      ),
      body: CustomScrollView(
        // A new draft gets fresh fields; within a draft, fields keep their
        // own text while Rust echoes it back.
        key: ValueKey(draft.id),
        slivers: [
          SliverPadding(
            padding: const EdgeInsets.fromLTRB(16, 16, 16, 0),
            sliver: SliverList.list(children: _header(draft)),
          ),
          SliverPadding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            sliver: SliverReorderableList(
              itemCount: draft.steps.length,
              // Keyed by row id. The list re-creates a card that moved (its
              // item key includes the index); the new fields start from the
              // text Rust holds for that row, so typed text follows its row.
              itemBuilder: (context, index) {
                final step = draft.steps[index];
                return _StepCard(
                  key: ValueKey(step.id),
                  step: step,
                  index: index,
                  nudges: draft.durationNudgeButtons,
                );
              },
              onReorderItem: (from, to) => _moveStep(draft, from, to),
              // The dragged card floats in an overlay, outside the page's
              // Material; its text fields need one of their own.
              proxyDecorator: (child, _, _) => Material(
                type: MaterialType.transparency,
                child: child,
              ),
            ),
          ),
          SliverPadding(
            padding: const EdgeInsets.fromLTRB(16, 0, 16, 16),
            sliver: SliverList.list(children: _footer(draft)),
          ),
        ],
      ),
    );
  }

  /// Shows the dropped card in its new place at once, then asks Rust for the
  /// move. Waiting for Rust's draft would show the old order for a frame or
  /// more after the drop; Rust's draft still replaces this one when it
  /// comes, with its order, labels and any refusal.
  void _moveStep(EditorDraft draft, int from, int to) {
    final steps = [...draft.steps];
    final moved = steps.removeAt(from);
    steps.insert(to, moved);
    setState(() => _draft = draft.copyWith(steps: steps));
    MoveStep(row: moved.id, to: to).sendSignalToRust();
  }

  /// The recipe's own fields, the live total and the steps heading.
  List<Widget> _header(EditorDraft draft) => [
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
      onChanged: (raw) => SetRecipeDescription(raw: raw).sendSignalToRust(),
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
            onChanged: (raw) => SetRecipeCoffee(raw: raw).sendSignalToRust(),
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
            onChanged: (raw) => SetRecipeWater(raw: raw).sendSignalToRust(),
          ),
        ),
      ],
    ),
    const SizedBox(height: 12),
    TextFormField(
      initialValue: draft.pourSpeed,
      decoration: InputDecoration(
        labelText: 'Pour speed (g/s)',
        helperText: 'Times each step pour as you type its grams',
        errorText: draft.pourSpeedError,
      ),
      keyboardType: const TextInputType.numberWithOptions(decimal: true),
      onChanged: (raw) => SetRecipePourSpeed(raw: raw).sendSignalToRust(),
    ),
    const SizedBox(height: 12),
    TextFormField(
      initialValue: draft.grind,
      decoration: InputDecoration(
        labelText: 'Grind (clicks)',
        helperText: 'Optional: where to set the grinder',
        errorText: draft.grindError,
      ),
      keyboardType: TextInputType.number,
      onChanged: (raw) => SetRecipeGrind(raw: raw).sendSignalToRust(),
    ),
    const SizedBox(height: 20),
    RuledHeading(
      text: 'Label paper',
      ink: Theme.of(context).colorScheme.onSurface,
    ),
    const SizedBox(height: 8),
    TextureGallery(
      textures: draft.textures,
      onPick: (texture) =>
          SetRecipeTexture(texture: texture).sendSignalToRust(),
    ),
    const SizedBox(height: 20),
    Text('Total duration: ${draft.totalLabel}'),
    if (draft.pourHint case final hint?)
      Padding(
        padding: const EdgeInsets.only(top: 4),
        child: Text(
          hint,
          style: TextStyle(
            fontStyle: FontStyle.italic,
            color: Theme.of(context).colorScheme.onSurfaceVariant,
          ),
        ),
      ),
    const SizedBox(height: 20),
    RuledHeading(
      text: 'Steps',
      ink: Theme.of(context).colorScheme.onSurface,
    ),
    const SizedBox(height: 8),
  ];

  /// The add-step buttons and the save button, below the steps.
  List<Widget> _footer(EditorDraft draft) => [
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
      child: const Text('SAVE RECIPE'),
    ),
  ];

  /// Asks before deleting, naming the recipe as saved, not as typed.
  void _confirmDelete(String recipeId) {
    final saved = widget.recipes.byId(recipeId);
    if (saved == null) return;
    showDialog<void>(
      context: context,
      builder: (ctx) => AlertDialog(
        title: const Text('Delete recipe'),
        content: Text(
          'Delete "${saved.name}"?',
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
    );
  }
}

/// One step's name, duration and pour inputs, keyed by its row id so its fields
/// keep their row's text as rows come, go and move. Its drag handle starts a
/// move; its quick buttons nudge the duration.
class _StepCard extends StatelessWidget {
  const _StepCard({
    required this.step,
    required this.index,
    required this.nudges,
    super.key,
  });

  final EditorStep step;

  /// The card's position in the list, for the drag handle.
  final int index;

  /// The quick duration buttons, in order.
  final List<DurationNudgeButton> nudges;

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
                ReorderableDragStartListener(
                  index: index,
                  child: const Padding(
                    padding: EdgeInsets.only(right: 8),
                    child: Icon(
                      Icons.drag_handle,
                      semanticLabel: 'Drag to reorder step',
                    ),
                  ),
                ),
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
                  icon: const Icon(Icons.delete_outline, color: Colors.red),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Expanded(
                  child: Column(
                    children: [
                      TextFormField(
                        // A new revision is text Rust replaced (a nudge):
                        // a fresh field starts from it. Typing keeps the
                        // revision, so the field keeps its own text.
                        key: ValueKey(step.durationRevision),
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
                      const SizedBox(height: 4),
                      // The buttons share the field's width, and a label
                      // too wide for its share shrinks rather than overflow.
                      Row(
                        children: [
                          for (final button in nudges)
                            Expanded(
                              child: Tooltip(
                                message: button.description,
                                child: TextButton(
                                  style: TextButton.styleFrom(
                                    minimumSize: const Size(0, 32),
                                    padding: const EdgeInsets.symmetric(
                                      horizontal: 2,
                                    ),
                                    tapTargetSize:
                                        MaterialTapTargetSize.shrinkWrap,
                                    visualDensity: VisualDensity.compact,
                                  ),
                                  onPressed: () => NudgeStepDuration(
                                    row: step.id,
                                    nudge: button.nudge,
                                  ).sendSignalToRust(),
                                  child: FittedBox(
                                    fit: BoxFit.scaleDown,
                                    child: Text(button.label),
                                  ),
                                ),
                              ),
                            ),
                        ],
                      ),
                    ],
                  ),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: TextFormField(
                    initialValue: step.waterText,
                    decoration: InputDecoration(
                      labelText: 'Pour (g)',
                      helperText: 'Blank: no pour',
                      errorText: step.waterError,
                    ),
                    keyboardType: TextInputType.number,
                    onChanged: (raw) => SetStepWater(
                      row: step.id,
                      raw: raw,
                    ).sendSignalToRust(),
                  ),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
