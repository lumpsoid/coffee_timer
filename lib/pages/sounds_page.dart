import 'package:coffee_timer/holders/settings_holder.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_parts.dart';
import 'package:flutter/material.dart';

/// The two sound pickers, between steps and at the end of a brew, over the
/// sounds Rust offers, each row with the sound's length. Tapping a row
/// plays it and makes it the choice; the page keeps nothing of its own.
class SoundsPage extends StatelessWidget {
  const SoundsPage({required this.settings, super.key});

  final SettingsHolder settings;

  static MaterialPageRoute<void> route(SettingsHolder settings) =>
      MaterialPageRoute(builder: (_) => SoundsPage(settings: settings));

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('SOUNDS')),
      body: ListenableBuilder(
        listenable: settings,
        builder: (context, _) => switch (settings.state) {
          final SettingsUpdate state => _sounds(context, state),
          null => const Center(child: CircularProgressIndicator()),
        },
      ),
    );
  }

  Widget _sounds(BuildContext context, SettingsUpdate state) {
    final ink = Theme.of(context).colorScheme.onSurface;
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        RuledHeading(text: 'Between steps', ink: ink),
        const SizedBox(height: 8),
        for (final sound in state.sounds)
          _SoundRow(
            key: ValueKey(('step', sound.id)),
            sound: sound,
            selected: sound.id == state.stepSound,
            select: () => SetStepSound(sound: sound.id).sendSignalToRust(),
          ),
        const SizedBox(height: 20),
        RuledHeading(text: 'End of brew', ink: ink),
        const SizedBox(height: 8),
        for (final sound in state.sounds)
          _SoundRow(
            key: ValueKey(('finish', sound.id)),
            sound: sound,
            selected: sound.id == state.finishSound,
            select: () => SetFinishSound(sound: sound.id).sendSignalToRust(),
          ),
      ],
    );
  }
}

/// One sound: radio mark, name and length. A tap previews it, and selects
/// it unless it already is.
class _SoundRow extends StatelessWidget {
  const _SoundRow({
    required this.sound,
    required this.selected,
    required this.select,
    super.key,
  });

  final SoundChoice sound;
  final bool selected;
  final VoidCallback select;

  @override
  Widget build(BuildContext context) {
    return ListTile(
      selected: selected,
      leading: Icon(
        selected ? Icons.radio_button_checked : Icons.radio_button_unchecked,
      ),
      title: Text(sound.label),
      trailing: Text(sound.lengthLabel),
      onTap: () {
        PreviewSound(sound: sound.id).sendSignalToRust();
        if (!selected) select();
      },
    );
  }
}
