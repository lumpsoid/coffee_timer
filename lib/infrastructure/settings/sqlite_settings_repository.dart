import 'package:coffee_timer/application/settings/settings_ports.dart';
import 'package:coffee_timer/domain/settings/accent_color.dart';
import 'package:coffee_timer/domain/settings/settings.dart';
import 'package:coffee_timer/domain/settings/theme_selection.dart';
import 'package:sqlite_async/sqlite_async.dart';

/// Key-value SQLite storage for [Settings]. Wire-compatible with the previous
/// `settings` table (a `theme` row and an `accent_color` row as `#AARRGGBB`).
///
/// This is the only place raw rows and hex strings appear: the smart
/// constructors ([ThemeSelection.fromStorage], [AccentColor.tryFromHex]) run
/// here at the boundary, so everything above holds valid domain values.
class SqliteSettingsRepository
    implements SettingsReadRepository, SettingsWriteRepository {
  SqliteSettingsRepository(this._db);

  static const _table = 'settings';
  static const _themeKey = 'theme';
  static const _accentKey = 'accent_color';

  final SqliteDatabase _db;

  /// Creates the table if it does not exist. Call once before first use.
  Future<void> initialize() async {
    await _db.execute('''
      CREATE TABLE IF NOT EXISTS $_table (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
      )
    ''');
  }

  @override
  Future<Settings> load() async {
    final rows = await _db.getAll('SELECT key, value FROM $_table');
    final stored = <String, String>{
      for (final row in rows) row['key'] as String: row['value'] as String,
    };
    return Settings(
      theme: ThemeSelection.fromStorage(stored[_themeKey]),
      accent:
          AccentColor.tryFromHex(stored[_accentKey] ?? '') ??
          AccentColor.fallback,
    );
  }

  @override
  Future<void> save(Settings settings) async {
    await _db.writeTransaction((tx) async {
      await tx.execute(
        'INSERT OR REPLACE INTO $_table (key, value) VALUES (?, ?)',
        [_themeKey, settings.theme.storageValue],
      );
      await tx.execute(
        'INSERT OR REPLACE INTO $_table (key, value) VALUES (?, ?)',
        [_accentKey, settings.accent.toHex()],
      );
    });
  }
}
