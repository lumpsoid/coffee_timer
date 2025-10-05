import 'package:coffee_timer/core/settings/buses/buses.dart';
import 'package:coffee_timer/core/settings/entities/entities.dart';
import 'package:coffee_timer/core/settings/events/events.dart';
import 'package:coffee_timer/core/settings/storage/storage.dart';
import 'package:path/path.dart' as p;
import 'package:path_provider/path_provider.dart';
import 'package:sqlite_async/sqlite_async.dart';

class SettingsStorageSqlite {
  SettingsStorageSqlite({required SettingsEventBus bus}) : _bus = bus {
    _bus.stream.listen(_handleEvent);
  }

  final SettingsEventBus _bus;
  late final SqliteDatabase _db;

  Future<void> initialize() async {
    final dir = await getApplicationDocumentsDirectory();
    final dbPath = p.join(dir.path, 'settings.db');
    _db = SqliteDatabase(path: dbPath);

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

  Future<void> _handleEvent(SettingsEvent event) async {
    switch (event) {
      case SettingsLoadStorage _:
        final settings = await _loadSettings();
        _bus.fire(SettingsLoadedStorage(settings));
      case SettingsUpdateStorage _:
        await _updateSettings(event.settings);
        _bus.fire(SettingsUpdatedStorage(event.settings));
      default:
        break;
    }
  }

  Future<Settings> _loadSettings() async {
    final rows = await _db.getAll('SELECT * FROM ${SettingsTable.tableName}');
    print(rows);

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
    print('parse color string $colorStr');
    return int.parse(hex, radix: 16);
  }

  Future<void> _updateSettings(Settings settings) async {
    final themeValue = settings.theme;
    final accentColorValue =
        '#${settings.accentColor.toRadixString(16).padLeft(8, '0')}';
    print('Sqlite storage settings: $accentColorValue');

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
