import 'package:coffee_timer/domain/settings/accent_color.dart';
import 'package:coffee_timer/domain/settings/theme_selection.dart';

/// Immutable user settings. State changes only through the named operations
/// below — there are no setters, so a `Settings` can never be half-updated.
class Settings {
  const Settings({required this.theme, required this.accent});

  /// Defaults used when nothing is stored yet.
  static const fallback = Settings(
    theme: ThemeSelection.light,
    accent: AccentColor.fallback,
  );

  final ThemeSelection theme;
  final AccentColor accent;

  Settings withTheme(ThemeSelection value) =>
      Settings(theme: value, accent: accent);

  Settings withAccent(AccentColor value) =>
      Settings(theme: theme, accent: value);
}
