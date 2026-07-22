import 'package:coffee_timer/application/settings/settings_ports.dart';
import 'package:coffee_timer/domain/settings/accent_color.dart';
import 'package:coffee_timer/domain/settings/settings.dart';
import 'package:coffee_timer/domain/settings/theme_selection.dart';
import 'package:coffee_timer/ui/settings/theme_controller.dart';
import 'package:flutter/foundation.dart';

/// Page-scoped controller for the settings screen. It edits [Settings] through
/// named domain operations, applies the result to the live [ThemeController],
/// and persists it. No business decisions live here — it only orchestrates.
class SettingsController extends ChangeNotifier {
  SettingsController({
    required SettingsReadRepository read,
    required SettingsWriteRepository write,
    required ThemeController theme,
  }) : _read = read,
       _write = write,
       _theme = theme;

  final SettingsReadRepository _read;
  final SettingsWriteRepository _write;
  final ThemeController _theme;

  Settings _settings = Settings.fallback;

  Settings get settings => _settings;

  /// Loads current settings into the page.
  Future<void> load() async {
    _settings = await _read.load();
    notifyListeners();
  }

  Future<void> setTheme(ThemeSelection value) =>
      _apply(_settings.withTheme(value));

  Future<void> setAccent(AccentColor value) =>
      _apply(_settings.withAccent(value));

  Future<void> _apply(Settings next) async {
    _settings = next;
    _theme.apply(next); // live app theme
    notifyListeners(); // page UI
    await _write.save(next); // persist
  }
}
