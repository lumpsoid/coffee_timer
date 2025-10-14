import 'package:coffee_timer/core/events/events.dart';
import 'package:coffee_timer/core/settings/settings.dart';
import 'package:sqlite_async/sqlite_async.dart';

class SettingsStorageSqlite {
  SettingsStorageSqlite({
    required SqliteDatabase db,
    required this.emit,
  }) : _db = db;

  final SqliteDatabase _db;

  void Function(Event) emit;

  Future<void> initialize() async {
    // Create key-value table
    await _db.execute('''
      CREATE TABLE IF NOT EXISTS ${SettingsTable.tableName} (
        ${SettingsTable.keyColumn} TEXT PRIMARY KEY,
        ${SettingsTable.valueColumn} TEXT NOT NULL
      )
    ''');

    // Insert defaults if missing
    await _ensureSettingExists(
      SettingsTable.themeKey,
      SettingsTable.defaultTheme,
    );
    await _ensureSettingExists(
      SettingsTable.accentColorKey,
      SettingsTable.defaultAccentColor,
    );
  }

  Future<void> _ensureSettingExists(String key, String defaultValue) async {
    final result = await _db.getOptional(
      'SELECT ${SettingsTable.valueColumn} FROM ${SettingsTable.tableName} WHERE ${SettingsTable.keyColumn} = ?',
      [key],
    );
    if (result == null) {
      await _db.execute(
        'INSERT INTO ${SettingsTable.tableName} (${SettingsTable.keyColumn}, ${SettingsTable.valueColumn}) VALUES (?, ?)',
        [key, defaultValue],
      );
    }
  }

  Future<void> handleEvent(Event event) async {
    switch (event) {
      case SettingsLoadStorage _:
        final settings = await _loadSettings();
        emit(SettingsLoadedStorage(settings));
      case SettingsUpdateStorage _:
        await _updateSettings(event.settings);
        emit(SettingsUpdatedStorage(event.settings));
      default:
        break;
    }
  }

  Future<Settings> _loadSettings() async {
    final rows = await _db.getAll('SELECT * FROM ${SettingsTable.tableName}');

    final settingsMap = <String, dynamic>{};
    for (final row in rows) {
      final key = row[SettingsTable.keyColumn] as String;
      final value = row[SettingsTable.valueColumn] as String;

      // Map DB key to Settings field name
      if (key == SettingsTable.themeKey) {
        settingsMap['theme'] = value;
      } else if (key == SettingsTable.accentColorKey) {
        // Parse color string like "#FF2196F3" to int
        settingsMap['accent_color'] = _parseColorString(value);
      }
    }

    return Settings.fromMap(settingsMap);
  }

  int _parseColorString(String colorStr) {
    // Remove #
    var hex = colorStr.startsWith('#') ? colorStr.substring(1) : colorStr;
    // Ensure it's 8 digits (ARGB)
    if (hex.length == 6) {
      hex = 'FF$hex'; // opaque
    }
    return int.parse(hex, radix: 16);
  }

  Future<void> _updateSettings(Settings settings) async {
    final themeValue = settings.theme;
    final accentColorValue =
        '#${settings.accentColor.toRadixString(16).padLeft(8, '0')}';

    // Use INSERT OR REPLACE to upsert
    await _db.execute(
      'INSERT OR REPLACE INTO ${SettingsTable.tableName} (${SettingsTable.keyColumn}, ${SettingsTable.valueColumn}) VALUES (?, ?)',
      [SettingsTable.themeKey, themeValue],
    );

    await _db.execute(
      'INSERT OR REPLACE INTO ${SettingsTable.tableName} (${SettingsTable.keyColumn}, ${SettingsTable.valueColumn}) VALUES (?, ?)',
      [SettingsTable.accentColorKey, accentColorValue],
    );
  }
}
