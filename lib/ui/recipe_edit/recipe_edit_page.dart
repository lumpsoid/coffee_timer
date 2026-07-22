import 'dart:async';

import 'package:coffee_timer/app/cells.dart';
import 'package:coffee_timer/domain/recipe/recipe_name.dart';
import 'package:coffee_timer/domain/recipe/step_duration.dart';
import 'package:coffee_timer/ui/navigation/route_args.dart';
import 'package:coffee_timer/ui/recipe_edit/recipe_edit_controller.dart';
import 'package:flutter/material.dart';

/// Create/edit screen for a recipe. Creates its [RecipeEditController] on mount
/// and disposes it on unmount (the page-scoped lifecycle from
/// `docs/pivot-spec.md`). Saving/deleting persists straight through the use
/// cases — no event bus — and pops; the list reloads on return.
class RecipeEditPage extends StatefulWidget {
  const RecipeEditPage({required this.args, super.key});

  final RecipeEditArgs args;

  static MaterialPageRoute<void> route(RecipeEditArgs args) =>
      MaterialPageRoute(builder: (_) => RecipeEditPage(args: args));

  @override
  State<RecipeEditPage> createState() => _RecipeEditPageState();
}

class _RecipeEditPageState extends State<RecipeEditPage> {
  final _formKey = GlobalKey<FormState>();
  late final RecipeEditController _controller;

  @override
  void initState() {
    super.initState();
    _controller = Cells.recipeModule.value.newEditController(widget.args);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  Future<void> _save() async {
    if (!_formKey.currentState!.validate()) return;
    final saved = await _controller.save();
    if (!mounted) return;
    if (saved == null) {
      _showError();
      return;
    }
    Navigator.of(context).pop();
  }

  void _showError() {
    final error = _controller.error;
    if (error == null) return;
    ScaffoldMessenger.of(context).showSnackBar(SnackBar(content: Text(error)));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: Text(_controller.isEditing ? 'Edit Recipe' : 'New Recipe'),
        actions: [
          if (_controller.isEditing)
            IconButton(
              tooltip: 'Delete',
              onPressed: _confirmDelete,
              icon: const Icon(Icons.delete),
            ),
          IconButton(
            tooltip: 'Save',
            onPressed: () => unawaited(_save()),
            icon: const Icon(Icons.save),
          ),
        ],
      ),
      body: Form(
        key: _formKey,
        child: ListenableBuilder(
          listenable: _controller,
          builder: (context, _) {
            return ListView(
              padding: const EdgeInsets.all(16),
              children: [
                TextFormField(
                  initialValue: _controller.name,
                  decoration: const InputDecoration(labelText: 'Recipe name'),
                  onChanged: (v) => _controller.name = v,
                  validator: (v) => RecipeName.tryParse(v ?? '') == null
                      ? 'Enter a recipe name'
                      : null,
                ),
                const SizedBox(height: 12),
                TextFormField(
                  initialValue: _controller.description,
                  decoration: const InputDecoration(labelText: 'Description'),
                  maxLines: 3,
                  onChanged: (v) => _controller.description = v,
                ),
                const SizedBox(height: 20),
                Text('Total duration: ${_controller.totalDurationLabel}'),
                const SizedBox(height: 20),
                const Text('Steps', style: TextStyle(fontSize: 16)),
                const SizedBox(height: 8),
                for (var i = 0; i < _controller.rows.length; i++)
                  _StepCard(
                    key: ObjectKey(_controller.rows[i]),
                    index: i,
                    row: _controller.rows[i],
                    onNameChanged: (v) => _controller.setStepName(i, v),
                    onDurationChanged: (v) => _controller.setStepDuration(i, v),
                    onRemove: () => _controller.removeStepAt(i),
                  ),
                _AddStepButtons(onAdd: _controller.addStep),
                const SizedBox(height: 20),
                ElevatedButton(
                  onPressed: () => unawaited(_save()),
                  child: const Text('Save Recipe'),
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  void _confirmDelete() {
    unawaited(
      showDialog<void>(
        context: context,
        builder: (ctx) => AlertDialog(
          title: const Text('Delete recipe'),
          content: Text(
            'Delete "${_controller.name}"? This action cannot be undone.',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: const Text('Cancel'),
            ),
            TextButton(
              onPressed: () async {
                Navigator.of(ctx).pop();
                final deleted = await _controller.deleteExisting();
                if (!mounted) return;
                if (deleted) {
                  Navigator.of(context).pop();
                } else {
                  _showError();
                }
              },
              child: const Text('Delete', style: TextStyle(color: Colors.red)),
            ),
          ],
        ),
      ),
    );
  }
}

/// A single step's name + duration inputs. Keyed by its [StepRow] identity so
/// its field state survives add/remove of other steps.
class _StepCard extends StatelessWidget {
  const _StepCard({
    required this.index,
    required this.row,
    required this.onNameChanged,
    required this.onDurationChanged,
    required this.onRemove,
    super.key,
  });

  final int index;
  final StepRow row;
  final ValueChanged<String> onNameChanged;
  final ValueChanged<String> onDurationChanged;
  final VoidCallback onRemove;

  @override
  Widget build(BuildContext context) {
    final mmss =
        StepDuration.clamped(int.tryParse(row.duration.trim()) ?? 0).mmss;
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
                    initialValue: row.name,
                    decoration: InputDecoration(
                      labelText: 'Step ${index + 1} name',
                    ),
                    onChanged: onNameChanged,
                  ),
                ),
                const SizedBox(width: 12),
                IconButton(
                  tooltip: 'Remove step',
                  onPressed: onRemove,
                  icon: const Icon(Icons.delete, color: Colors.red),
                ),
              ],
            ),
            const SizedBox(height: 8),
            TextFormField(
              initialValue: row.duration,
              decoration: InputDecoration(
                labelText: 'Duration (seconds)',
                helperText: 'MM:SS — $mmss',
              ),
              keyboardType: TextInputType.number,
              onChanged: onDurationChanged,
              validator: (v) {
                final n = int.tryParse((v ?? '').trim());
                if (n == null) return 'Enter seconds';
                if (StepDuration.tryParse(n) == null) {
                  return '0–${StepDuration.maxStepSeconds} seconds';
                }
                return null;
              },
            ),
          ],
        ),
      ),
    );
  }
}

/// The preset "add step" buttons.
class _AddStepButtons extends StatelessWidget {
  const _AddStepButtons({required this.onAdd});

  final ValueChanged<String> onAdd;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: Wrap(
        runSpacing: 4,
        spacing: 4,
        children: [
          for (final preset in const ['Add coffee', 'Add water', 'Wait'])
            TextButton.icon(
              onPressed: () => onAdd(preset),
              icon: const Icon(Icons.add),
              label: Text(preset),
            ),
          TextButton.icon(
            onPressed: () => onAdd(''),
            icon: const Icon(Icons.add),
            label: const Text('Add Step'),
          ),
        ],
      ),
    );
  }
}
