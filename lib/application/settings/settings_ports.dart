import 'package:coffee_timer/domain/settings/settings.dart';

/// Reads persisted settings. Narrow — read only.
abstract interface class SettingsReadRepository {
  Future<Settings> load();
}

/// Persists settings. Narrow — write only.
abstract interface class SettingsWriteRepository {
  Future<void> save(Settings settings);
}
