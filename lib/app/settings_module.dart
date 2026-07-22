import 'package:coffee_timer/infrastructure/settings/sqlite_settings_repository.dart';
import 'package:coffee_timer/ui/settings/settings_controller.dart';
import 'package:coffee_timer/ui/settings/theme_controller.dart';

/// Shared settings infrastructure plus a factory for page-scoped
/// [SettingsController]s. Registered once by `AppRoot` into `Cells`.
class SettingsModule {
  SettingsModule({required this.repository, required ThemeController theme})
    : _theme = theme;

  final SqliteSettingsRepository repository;
  final ThemeController _theme;

  SettingsController newController() =>
      SettingsController(read: repository, write: repository, theme: _theme);
}
