import 'package:coffee_timer/application/settings/settings_ports.dart';
import 'package:coffee_timer/domain/settings/accent_color.dart';
import 'package:coffee_timer/domain/settings/settings.dart';
import 'package:coffee_timer/domain/settings/theme_selection.dart';
import 'package:flutter/material.dart';

/// App-lifetime controller that drives [MaterialApp]'s theming. Lives in a
/// static cell; loaded once at startup and updated live by
/// `SettingsController` when the user changes a preference.
class ThemeController extends ChangeNotifier {
  ThemeController(this._read);

  final SettingsReadRepository _read;

  Settings _settings = Settings.fallback;

  ThemeSelection get selection => _settings.theme;

  AccentColor get accent => _settings.accent;

  /// Loads persisted settings and notifies listeners.
  Future<void> load() async {
    _settings = await _read.load();
    notifyListeners();
  }

  /// Applies already-decided settings (e.g. after an edit) and notifies.
  void apply(Settings settings) {
    _settings = settings;
    notifyListeners();
  }

  ThemeMode get mode => switch (_settings.theme) {
    ThemeSelection.light => ThemeMode.light,
    ThemeSelection.dark => ThemeMode.dark,
    ThemeSelection.system => ThemeMode.system,
  };

  ThemeData get light => ThemeData.light().copyWith(
    colorScheme: ColorScheme.fromSeed(seedColor: _seed),
  );

  ThemeData get dark => ThemeData.dark().copyWith(
    colorScheme: ColorScheme.fromSeed(
      seedColor: _seed,
      brightness: Brightness.dark,
    ),
  );

  Color get _seed => Color(_settings.accent.argb);
}
