import 'package:coffee_timer/core/events/events.dart';
import 'package:coffee_timer/core/settings/settings.dart';
import 'package:flutter/material.dart';

class ThemeManager extends ChangeNotifier {
  ThemeManager._internal();

  static final ThemeManager _instance = ThemeManager._internal();
  static ThemeManager get instance => _instance;

  String _currentTheme = 'system';
  Color _accentColor = const Color(0xFF2196F3); // default blue

  ThemeMode get themeMode {
    switch (_currentTheme) {
      case 'dark':
        return ThemeMode.dark;
      case 'light':
        return ThemeMode.light;
      default:
        return ThemeMode.system;
    }
  }

  ThemeData get light {
    final base = ThemeData.light();
    return base.copyWith(
      colorScheme: ColorScheme.fromSeed(
        seedColor: _accentColor,
      ),
    );
  }

  ThemeData get dark {
    final base = ThemeData.dark();
    return base.copyWith(
      colorScheme: ColorScheme.fromSeed(
        seedColor: _accentColor,
        brightness: Brightness.dark,
      ),
    );
  }

  ThemeData get themeData {
    final base = _currentTheme == 'dark' ? ThemeData.dark() : ThemeData.light();
    return base.copyWith(
      colorScheme: ColorScheme.fromSeed(
        seedColor: _accentColor,
        brightness: _currentTheme == 'dark'
            ? Brightness.dark
            : Brightness.light,
      ),
    );
  }

  String get currentTheme => _currentTheme;
  Color get accentColor => _accentColor;

  void handleEvent(Event event) {
    switch (event) {
      case SettingsLoaded(:final settings):
      case SettingsUpdated(:final settings):
        print('ThemeManager: accentColor ${settings.accentColor}');
        _currentTheme = settings.theme;
        _accentColor = Color(settings.accentColor);
        notifyListeners();

      default:
        break;
    }
  }
}
