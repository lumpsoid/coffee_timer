abstract class SettingsTable {
  static const String tableName = 'settings';
  static const String keyColumn = 'key';
  static const String valueColumn = 'value';

  static const String themeKey = 'theme';
  static const String accentColorKey = 'accent_color';

  static const String defaultTheme = 'light';
  static const String defaultAccentColor = '#FF2196F3'; // Blue 500

  // Helper to map Settings fields to DB keys
  static const Map<String, String> keyToField = {
    themeKey: 'theme',
    accentColorKey: 'accentColor',
  };
}
