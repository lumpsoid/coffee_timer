import 'dart:async';

import 'package:coffee_timer/core/recipe/recipe.dart';
import 'package:coffee_timer/recipe_list/pages/pages.dart';
import 'package:flutter/material.dart' hide Step;
import 'package:uuid/uuid.dart';

class RecipeEditPage extends StatefulWidget {
  const RecipeEditPage({
    super.key,
    this.initial = const Recipe.empty(),
  });
  final Recipe initial;

  static MaterialPageRoute<void> route({
    Recipe initial = const Recipe.empty(),
  }) {
    return MaterialPageRoute(
      builder: (context) => RecipeEditPage(
        initial: initial,
      ),
    );
  }

  @override
  State<RecipeEditPage> createState() => _RecipeEditPageState();
}

class _RecipeEditPageState extends State<RecipeEditPage> {
  final RecipeEventBus _bus = RecipeHub.instance.bus;

  late final StreamSubscription<RecipeEvent> _sub;

  final _formKey = GlobalKey<FormState>();
  late TextEditingController _nameController;
  late TextEditingController _descController;

  // controllers per step
  final List<TextEditingController> _stepNameControllers = [];
  final List<TextEditingController> _stepDurationControllers = [];

  @override
  void initState() {
    super.initState();

    _nameController = TextEditingController(text: widget.initial.name);
    _descController = TextEditingController(text: widget.initial.description);

    for (final s in widget.initial.steps) {
      _stepNameControllers.add(TextEditingController(text: s.name));
      _stepDurationControllers.add(
        TextEditingController(text: s.durationSeconds.toString()),
      );
    }

    // ensure at least one empty step if none
    if (_stepNameControllers.isEmpty) {
      _addEmptyStep();
    }

    _sub = _bus.stream.listen((event) {
      switch (event) {
        // handle events if needed
        default:
          break;
      }
    });
  }

  @override
  void dispose() {
    unawaited(_sub.cancel());
    _nameController.dispose();
    _descController.dispose();
    for (final c in _stepNameControllers) {
      c.dispose();
    }
    for (final c in _stepDurationControllers) {
      c.dispose();
    }
    super.dispose();
  }

  void _addEmptyStep([String? name]) {
    final nameCtrl = TextEditingController(text: name);
    setState(() {
      _stepNameControllers.add(nameCtrl);
      _stepDurationControllers.add(TextEditingController(text: '0'));
    });
  }

  void _removeStep(int index) {
    setState(() {
      _stepNameControllers[index].dispose();
      _stepDurationControllers[index].dispose();
      _stepNameControllers.removeAt(index);
      _stepDurationControllers.removeAt(index);
      if (_stepNameControllers.isEmpty) {
        _addEmptyStep();
      }
    });
  }

  void _save() {
    if (!_formKey.currentState!.validate()) return;

    final steps = <Step>[];
    for (var i = 0; i < _stepNameControllers.length; i++) {
      final name = _stepNameControllers[i].text.trim();
      final durText = _stepDurationControllers[i].text.trim();
      final dur = int.tryParse(durText) ?? 0;
      final index = i + 1;
      final originalId = widget.initial.isEmpty
          ? const Uuid().v7()
          : widget.initial.steps[i].id;
      steps.add(
        Step(
          id: originalId,
          name: name.isEmpty ? 'Step $index' : name,
          durationSeconds: dur,
        ),
      );
    }

    final recipe = Recipe(
      id: widget.initial.id.isEmpty ? const Uuid().v7() : widget.initial.id,
      name: _nameController.text.trim(),
      description: _descController.text.trim(),
      steps: steps,
    );

    if (widget.initial.isEmpty) {
      _bus.fire(RecipeCreate(recipe));
    } else {
      _bus.fire(RecipeUpdate(recipe));
    }

    Navigator.of(context).pop(recipe);
  }

  String _totalDuration() {
    var totalSeconds = 0;
    for (var i = 0; i < _stepNameControllers.length; i++) {
      final durText = _stepDurationControllers[i].text;
      final dur = int.tryParse(durText) ?? 0;
      totalSeconds += dur;
    }
    return Recipe.totalDurationFormatted(totalSeconds);
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Edit Recipe'),
        actions: [
          if (widget.initial.isNotEmpty)
            IconButton(
              onPressed: () => _openConfirmationDelete(
                context,
                _nameController.text,
                widget.initial.id,
              ),
              icon: const Icon(Icons.delete),
            ),
          IconButton(
            onPressed: _save,
            icon: const Icon(Icons.save),
          ),
        ],
      ),
      body: Form(
        key: _formKey,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            TextFormField(
              controller: _nameController,
              decoration: const InputDecoration(labelText: 'Recipe name'),
              validator: (v) =>
                  (v == null || v.trim().isEmpty) ? 'Enter a name' : null,
            ),
            const SizedBox(height: 12),
            TextFormField(
              controller: _descController,
              decoration: const InputDecoration(labelText: 'Description'),
              maxLines: 3,
            ),
            const SizedBox(height: 20),
            Text('Total duration: ${_totalDuration()}'),
            const SizedBox(height: 20),
            const Text('Steps', style: TextStyle(fontSize: 16)),
            const SizedBox(height: 8),
            ...List.generate(_stepNameControllers.length + 1, (index) {
              if (index == _stepNameControllers.length) {
                return Padding(
                  padding: const EdgeInsets.symmetric(vertical: 8),
                  child: Wrap(
                    runSpacing: 4,
                    spacing: 4,
                    children: [
                      TextButton.icon(
                        onPressed: () => _addEmptyStep('Add coffee'),
                        icon: const Icon(Icons.add),
                        label: const Text('Add coffee'),
                      ),
                      TextButton.icon(
                        onPressed: () => _addEmptyStep('Add water'),
                        icon: const Icon(Icons.add),
                        label: const Text('Add water'),
                      ),
                      TextButton.icon(
                        onPressed: () => _addEmptyStep('Wait'),
                        icon: const Icon(Icons.add),
                        label: const Text('Wait'),
                      ),
                      TextButton.icon(
                        onPressed: _addEmptyStep,
                        icon: const Icon(Icons.add),
                        label: const Text('Add Step'),
                      ),
                    ],
                  ),
                );
              }
              final nameCtrl = _stepNameControllers[index];
              final durCtrl = _stepDurationControllers[index];
              final durValue = int.tryParse(durCtrl.text) ?? 0;
              final mmss = Recipe.totalDurationFormatted(durValue);

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
                              controller: nameCtrl,
                              decoration: InputDecoration(
                                labelText: 'Step ${index + 1} name',
                              ),
                            ),
                          ),
                          const SizedBox(width: 12),
                          IconButton(
                            onPressed: () => _removeStep(index),
                            icon: const Icon(Icons.delete, color: Colors.red),
                          ),
                        ],
                      ),
                      const SizedBox(height: 8),
                      TextFormField(
                        controller: durCtrl,
                        decoration: InputDecoration(
                          labelText: 'Duration (seconds)',
                          helperText: 'MM:SS — $mmss',
                        ),
                        keyboardType: TextInputType.number,
                        validator: (v) {
                          if (v == null || v.trim().isEmpty) {
                            return 'Enter seconds';
                          }
                          final n = int.tryParse(v);
                          if (n == null || n < 0) {
                            return 'Enter a non-negative integer';
                          }
                          return null;
                        },
                        onChanged: (_) =>
                            setState(() {}), // update MM:SS helper
                      ),
                    ],
                  ),
                ),
              );
            }),
            const SizedBox(height: 20),
            ElevatedButton(
              onPressed: _save,
              child: const Text('Save Recipe'),
            ),
          ],
        ),
      ),
    );
  }

  void _openConfirmationDelete(
    BuildContext context,
    String name,
    String id,
  ) {
    unawaited(
      showDialog<void>(
        context: context,
        builder: (ctx) => AlertDialog(
          title: const Text('Delete recipe'),
          content: Text(
            'Delete "$name"? This action cannot be undone.',
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(ctx).pop(),
              child: const Text('Cancel'),
            ),
            TextButton(
              onPressed: () {
                Navigator.of(ctx).pop(); // close dialog
                _bus.fire(RecipeDelete(id: id));
                Navigator.of(ctx).popUntil((r) => false);
                unawaited(
                  Navigator.of(ctx).push(RecipeListPage.route()),
                );
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
