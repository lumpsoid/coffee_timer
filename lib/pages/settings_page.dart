import 'dart:async';

import 'package:coffee_timer/holders/settings_holder.dart';
import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:coffee_timer/widgets/label_parts.dart';
import 'package:flutter/material.dart';

/// The theme picker, the accent colour picker and the brew's cues (sound,
/// vibration, pre-alert), over the options Rust offers. Every control shows
/// Rust's state and sends the change; none keeps its own.
class SettingsPage extends StatelessWidget {
  const SettingsPage({required this.settings, super.key});

  final SettingsHolder settings;

  static MaterialPageRoute<void> route(SettingsHolder settings) =>
      MaterialPageRoute(builder: (_) => SettingsPage(settings: settings));

  /// The picker's display names (UI text).
  static String themeName(ThemeChoice theme) => switch (theme) {
    ThemeChoice.light => 'Light',
    ThemeChoice.dark => 'Dark',
    ThemeChoice.system => 'System',
  };

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('SETTINGS')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: ListenableBuilder(
          listenable: settings,
          builder: (context, _) => switch (settings.state) {
            final SettingsUpdate state => _settings(context, state),
            null => const Center(child: CircularProgressIndicator()),
          },
        ),
      ),
    );
  }

  Widget _settings(BuildContext context, SettingsUpdate state) {
    final ink = Theme.of(context).colorScheme.onSurface;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        RuledHeading(text: 'Theme', ink: ink),
        const SizedBox(height: 16),
        DropdownButtonFormField<ThemeChoice>(
          // Follows Rust's state, which may differ from what was picked.
          key: ValueKey(state.theme),
          initialValue: state.theme,
          decoration: const InputDecoration(
            border: OutlineInputBorder(),
            labelText: 'Select Theme',
          ),
          items: [
            for (final theme in state.themes)
              DropdownMenuItem(value: theme, child: Text(themeName(theme))),
          ],
          onChanged: (theme) {
            if (theme != null) SetTheme(theme: theme).sendSignalToRust();
          },
        ),
        const SizedBox(height: 20),
        ListTile(
          title: const Text('Accent Color'),
          trailing: AnimatedContainer(
            duration: const Duration(milliseconds: 300),
            width: 14,
            height: 14,
            decoration: BoxDecoration(
              color: Color(state.accent),
              borderRadius: BorderRadius.circular(7),
            ),
          ),
          onTap: () => _showAccentColorPicker(context, state),
        ),
        const SizedBox(height: 20),
        RuledHeading(text: 'Brew cues', ink: ink),
        const SizedBox(height: 8),
        SwitchListTile(
          title: const Text('Sound'),
          subtitle: const Text('Play a sound at each step'),
          value: state.sound,
          onChanged: (on) => SetSound(on: on).sendSignalToRust(),
        ),
        SwitchListTile(
          title: const Text('Vibration'),
          subtitle: const Text('Buzz at each step and before it ends'),
          value: state.vibration,
          onChanged: (on) => SetVibration(on: on).sendSignalToRust(),
        ),
        const SizedBox(height: 8),
        DropdownButtonFormField<int>(
          // Follows Rust's state, which may differ from what was picked.
          key: ValueKey(('preAlert', state.preAlertSeconds)),
          initialValue: state.preAlertSeconds,
          decoration: const InputDecoration(
            border: OutlineInputBorder(),
            labelText: 'Pre-alert before each step ends',
          ),
          items: [
            for (final option in state.preAlerts)
              DropdownMenuItem(
                value: option.seconds,
                child: Text(option.label),
              ),
          ],
          onChanged: (seconds) {
            if (seconds != null) {
              SetPreAlert(seconds: seconds).sendSignalToRust();
            }
          },
        ),
      ],
    );
  }

  void _showAccentColorPicker(BuildContext context, SettingsUpdate state) {
    unawaited(
      showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('Choose Accent Color'),
          content: SizedBox(
            width: double.maxFinite,
            child: Wrap(
              spacing: 12,
              runSpacing: 12,
              children: [
                for (final option in state.accents)
                  GestureDetector(
                    onTap: () {
                      Navigator.of(context).pop();
                      SetAccent(argb: option.argb).sendSignalToRust();
                    },
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        Stack(
                          children: [
                            CircleAvatar(
                              radius: 20,
                              backgroundColor: Color(option.argb),
                            ),
                            if (option.argb == state.accent)
                              const Positioned.fill(
                                child: Icon(
                                  Icons.check,
                                  color: Colors.white,
                                  size: 20,
                                ),
                              ),
                          ],
                        ),
                        const SizedBox(height: 4),
                        Text(
                          option.name,
                          style: const TextStyle(fontSize: 12),
                          textAlign: TextAlign.center,
                        ),
                      ],
                    ),
                  ),
              ],
            ),
          ),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('Close'),
            ),
          ],
        ),
      ),
    );
  }
}
