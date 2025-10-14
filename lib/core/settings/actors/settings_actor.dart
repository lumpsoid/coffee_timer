import 'package:coffee_timer/core/events/events.dart';
import 'package:coffee_timer/core/settings/entities/entities.dart';
import 'package:coffee_timer/core/settings/events/events.dart';

class SettingsActor {
  SettingsActor({required this.emit});

  void Function(Event) emit;

  Settings? _currentSettings;

  void handleEvent(Event event) {
    switch (event) {
      case SettingsLoad _:
        if (_currentSettings != null) {
          emit(SettingsLoaded(_currentSettings!));
          return;
        }
        emit(const SettingsLoadStorage());
      case SettingsLoadedStorage(:final settings):
        _currentSettings = settings;
        emit(SettingsLoaded(settings));
      case SettingsUpdate _:
        _currentSettings = event.settings;
        emit(SettingsUpdateStorage(event.settings));
      case SettingsUpdatedStorage _:
        _currentSettings = event.settings;
        emit(SettingsUpdated(event.settings));
      case SettingsThemeChanged _:
        print('SettingsActor: settins theme changed ${event.theme}');
        if (_currentSettings != null) {
          final newSettings = _currentSettings!.copyWith(theme: event.theme);
          emit(SettingsUpdate(newSettings));
        }
      case SettingsAccentColorChanged(:final accentColor):
        print('SettingsActor: changed accentColor $accentColor');
        if (_currentSettings != null) {
          final newSettings = _currentSettings!.copyWith(
            accentColor: accentColor,
          );
          emit(SettingsUpdate(newSettings));
        }
      default:
        break;
    }
  }
}
