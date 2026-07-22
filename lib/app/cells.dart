import 'package:coffee_timer/app/cell.dart';
import 'package:coffee_timer/app/settings_module.dart';
import 'package:coffee_timer/ui/settings/theme_controller.dart';

/// Static roots the UI reads to reach shared modules and long-lived
/// controllers. Populated once by `AppRoot`. See `docs/pivot-spec.md`.
///
/// Recipe modules are added here as those features migrate off the event bus.
abstract final class Cells {
  /// App-lifetime theme controller that drives `MaterialApp`.
  static final theme = Cell<ThemeController>();

  /// Settings infrastructure + page-controller factory.
  static final settingsModule = Cell<SettingsModule>();
}
