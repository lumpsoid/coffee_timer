import 'dart:async';

import 'package:coffee_timer/app/cells.dart';
import 'package:coffee_timer/domain/settings/accent_color.dart';
import 'package:coffee_timer/domain/settings/theme_selection.dart';
import 'package:coffee_timer/ui/settings/settings_controller.dart';
import 'package:flutter/material.dart';

class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  static MaterialPageRoute<void> route() =>
      MaterialPageRoute(builder: (_) => const SettingsPage());

  @override
  State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  late final SettingsController _controller;

  @override
  void initState() {
    super.initState();
    _controller = Cells.settingsModule.value.newController();
    unawaited(_controller.load());
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _onThemeChanged(ThemeSelection? value) {
    if (value == null) return;
    unawaited(_controller.setTheme(value));
  }

  void _onAccentChanged(Color color) {
    final accent = AccentColor.tryFromArgb(color.toARGB32());
    if (accent == null) return;
    unawaited(_controller.setAccent(accent));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: ListenableBuilder(
          listenable: _controller,
          builder: (context, _) {
            final settings = _controller.settings;
            return Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                const Text(
                  'Theme',
                  style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
                ),
                const SizedBox(height: 8),
                DropdownButtonFormField<ThemeSelection>(
                  initialValue: settings.theme,
                  decoration: const InputDecoration(
                    border: OutlineInputBorder(),
                    labelText: 'Select Theme',
                  ),
                  items: const [
                    DropdownMenuItem(
                      value: ThemeSelection.light,
                      child: Text('Light'),
                    ),
                    DropdownMenuItem(
                      value: ThemeSelection.dark,
                      child: Text('Dark'),
                    ),
                    DropdownMenuItem(
                      value: ThemeSelection.system,
                      child: Text('System'),
                    ),
                  ],
                  onChanged: _onThemeChanged,
                ),
                const SizedBox(height: 20),
                ListTile(
                  title: const Text('Accent Color'),
                  trailing: AnimatedContainer(
                    duration: const Duration(milliseconds: 300),
                    width: 14,
                    height: 14,
                    decoration: BoxDecoration(
                      color: Color(settings.accent.argb),
                      borderRadius: BorderRadius.circular(7),
                    ),
                  ),
                  onTap: () => _showAccentColorPicker(context),
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  void _showAccentColorPicker(BuildContext context) {
    final selected = Color(_controller.settings.accent.argb);
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
              children: predefinedAccentColors.map((option) {
                final isSelected =
                    option.color.toARGB32() == selected.toARGB32();
                return GestureDetector(
                  onTap: () {
                    Navigator.of(context).pop();
                    _onAccentChanged(option.color);
                  },
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Stack(
                        children: [
                          CircleAvatar(
                            radius: 20,
                            backgroundColor: option.color,
                          ),
                          if (isSelected)
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
                );
              }).toList(),
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

class AccentColorOption {
  const AccentColorOption(this.name, this.color);

  final String name;
  final Color color;
}

const List<AccentColorOption> predefinedAccentColors = [
  AccentColorOption('Blue', Colors.blue),
  AccentColorOption('Red', Colors.red),
  AccentColorOption('Green', Colors.green),
  AccentColorOption('Purple', Colors.purple),
  AccentColorOption('Orange', Colors.orange),
  AccentColorOption('Pink', Colors.pink),
  AccentColorOption('Indigo', Colors.indigo),
  AccentColorOption('Teal', Colors.teal),
  AccentColorOption('Cyan', Colors.cyan),
  AccentColorOption('Lime', Colors.lime),
];
