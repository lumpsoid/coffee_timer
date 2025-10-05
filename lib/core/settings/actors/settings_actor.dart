import 'dart:async';

import 'package:coffee_timer/core/settings/buses/settings_event_bus.dart';
import 'package:coffee_timer/core/settings/entities/entities.dart';
import 'package:coffee_timer/core/settings/events/events.dart';

class SettingsActor {
  SettingsActor({required SettingsEventBus bus}) : _bus = bus {
    _subscription = _bus.stream.listen(_handleEvent);
  }

  final SettingsEventBus _bus;
  late final StreamSubscription<SettingsEvent> _subscription;

  Settings? _currentSettings;

  void _handleEvent(SettingsEvent event) {
    switch (event) {
      case SettingsLoad _:
        if (_currentSettings != null) {
          _bus.fire(SettingsLoaded(_currentSettings!));
          return;
        }
        _bus.fire(const SettingsLoadStorage());
      case SettingsLoadedStorage(:final settings):
        _currentSettings = settings;
        _bus.fire(SettingsLoaded(settings));
      case SettingsUpdate _:
        _currentSettings = event.settings;
        _bus.fire(SettingsUpdateStorage(event.settings));
      case SettingsUpdatedStorage _:
        _currentSettings = event.settings;
        _bus.fire(SettingsUpdated(event.settings));
      case SettingsThemeChanged _:
        print('SettingsActor: settins theme changed ${event.theme}');
        if (_currentSettings != null) {
          final newSettings = _currentSettings!.copyWith(theme: event.theme);
          _bus.fire(SettingsUpdate(newSettings));
        }
      case SettingsAccentColorChanged(:final accentColor):
        print('SettingsActor: changed accentColor $accentColor');
        if (_currentSettings != null) {
          final newSettings = _currentSettings!.copyWith(
            accentColor: accentColor,
          );
          _bus.fire(SettingsUpdate(newSettings));
        }
      default:
        break;
    }
  }

  void dispose() {
    unawaited(_subscription.cancel());
  }
}
