import 'dart:async';

import 'package:coffee_timer/core/settings/settings.dart';
import 'package:coffee_timer/core/theme/theme.dart';
import 'package:flutter/material.dart';

class SettingsPage extends StatefulWidget {
  const SettingsPage({super.key});

  static MaterialPageRoute<void> route() {
    return MaterialPageRoute(builder: (_) => const SettingsPage());
  }

  @override
  State<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends State<SettingsPage> {
  final SettingsHub _settingsHub = SettingsHub.instance;
  final _selectedTheme = ValueNotifier<String>('light');
  final _accentColor = ValueNotifier<Color>(ThemeManager.instance.accentColor);

  @override
  void initState() {
    super.initState();
    _settingsHub.bus.stream.listen(_handleEvent);
    SettingsHub.instance.bus.fire(const SettingsLoad());
  }

  void _handleEvent(SettingsEvent event) {
    switch (event) {
      case SettingsLoaded():
        _accentColor.value = Color(event.settings.accentColor);
        _selectedTheme.value = event.settings.theme;
      case SettingsUpdated(:final settings):
        _selectedTheme.value = settings.theme;
        _accentColor.value = Color(settings.accentColor);
    }
  }

  void _onThemeChanged(String? theme) {
    if (theme == null) return;

    _selectedTheme.value = theme;

    _settingsHub.bus.fire(SettingsThemeChanged(theme));
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Settings'),
      ),
      body: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            const Text(
              'Theme',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            const SizedBox(height: 8),
            ValueListenableBuilder(
              valueListenable: _selectedTheme,
              builder: (context, currentTheme, _) {
                return DropdownButtonFormField<String>(
                  initialValue: currentTheme,
                  decoration: const InputDecoration(
                    border: OutlineInputBorder(),
                    labelText: 'Select Theme',
                  ),
                  items: const [
                    DropdownMenuItem(
                      value: 'light',
                      child: Text('Light'),
                    ),
                    DropdownMenuItem(
                      value: 'dark',
                      child: Text('Dark'),
                    ),
                    DropdownMenuItem(
                      value: 'system',
                      child: Text('System'),
                    ),
                  ],
                  onChanged: _onThemeChanged,
                );
              },
            ),
            const SizedBox(height: 20),
            ElevatedButton(
              onPressed: () {
                _settingsHub.bus.fire(const SettingsLoad());
              },
              child: const Text('Refresh Settings'),
            ),
            // Accent color picker
            ValueListenableBuilder(
              valueListenable: _accentColor,
              builder: (context, c, _) {
                return ListTile(
                  title: const Text('Accent Color'),
                  trailing: AnimatedContainer(
                    duration: const Duration(milliseconds: 300),
                    width: 14,
                    height: 14,
                    decoration: BoxDecoration(
                      color: c,
                      borderRadius: BorderRadius.circular(
                        7,
                      ),
                    ),
                  ),
                  onTap: () => _showAccentColorPicker(context),
                );
              },
            ),
          ],
        ),
      ),
    );
  }

  void _showAccentColorPicker(BuildContext context) {
    unawaited(
      showDialog(
        context: context,
        builder: (context) => AlertDialog(
          title: const Text('Choose Accent Color'),
          content: SizedBox(
            width: double.maxFinite,
            child: Wrap(
              spacing: 12,
              runSpacing: 12,
              children: predefinedAccentColors.map((option) {
                final isSelected = option.color == _accentColor.value;
                return GestureDetector(
                  onTap: () {
                    Navigator.of(context).pop(); // close dialog

                    SettingsHub.instance.bus.fire(
                      SettingsAccentColorChanged(option.color.toARGB32()),
                    );
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
