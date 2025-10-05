import 'package:coffee_timer/core/settings/entities/entities.dart';
import 'package:coffee_timer/core/settings/events/settings_event.dart';

class SettingsLoad extends SettingsEvent {
  const SettingsLoad();
}

class SettingsLoaded extends SettingsEvent {
  const SettingsLoaded(this.settings);
  final Settings settings;
}

class SettingsUpdate extends SettingsEvent {
  const SettingsUpdate(this.settings);
  final Settings settings;
}

class SettingsUpdated extends SettingsEvent {
  const SettingsUpdated(this.settings);
  final Settings settings;
}

class SettingsThemeChanged extends SettingsEvent {
  const SettingsThemeChanged(this.theme);
  final String theme;
}

class SettingsAccentColorChanged extends SettingsEvent {
  const SettingsAccentColorChanged(this.accentColor);
  final int accentColor;
}
