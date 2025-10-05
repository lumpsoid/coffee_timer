import 'package:coffee_timer/core/settings/entities/entities.dart';
import 'package:coffee_timer/core/settings/events/events.dart';

class SettingsLoadStorage extends SettingsEvent {
  const SettingsLoadStorage();
}

class SettingsLoadedStorage extends SettingsEvent {
  const SettingsLoadedStorage(this.settings);
  final Settings settings;
}

class SettingsUpdateStorage extends SettingsEvent {
  const SettingsUpdateStorage(this.settings);
  final Settings settings;
}

class SettingsUpdatedStorage extends SettingsEvent {
  const SettingsUpdatedStorage(this.settings);
  final Settings settings;
}
