import 'package:coffee_timer/sql_migrator/domain/domain.dart';

/// A simple in-memory [MigrationExecutor] for unit testing migrations
/// without a real SQLite database.
class InMemoryExecutor implements MigrationExecutor {
  final Map<String, List<Map<String, Object?>>> _tables = {};
  final List<String> executedStatements = [];

  @override
  Future<void> execute(String sql, [List<Object?>? arguments]) async {
    executedStatements.add(sql);

    final normalized = sql.trim().toUpperCase();

    // DDL
    if (normalized.startsWith('CREATE TABLE')) {
      _tables.putIfAbsent(_extractTableName(sql), () => []);
      return;
    }

    if (normalized.startsWith('DROP TABLE')) {
      _tables.remove(_extractTableName(sql));
      return;
    }

    // INSERT [OR REPLACE]
    if (normalized.startsWith('INSERT')) {
      final tableName = _extractTableName(sql);
      _tables.putIfAbsent(tableName, () => []);

      if (arguments != null && arguments.isNotEmpty) {
        final columns = _extractInsertColumns(sql);
        final row = <String, Object?>{};
        for (var i = 0; i < columns.length && i < arguments.length; i++) {
          row[columns[i]] = arguments[i];
        }

        // OR REPLACE: remove existing row whose primary-key column matches.
        // The migration table keys on "version" (with rolled_back = 0); other
        // tables key on their first column.
        if (normalized.contains('OR REPLACE')) {
          final keyColumn = columns.contains('version')
              ? 'version'
              : columns.firstOrNull;

          if (keyColumn != null && row.containsKey(keyColumn)) {
            final keyValue = row[keyColumn];
            if (keyColumn == 'version') {
              _tables[tableName]?.removeWhere(
                (r) =>
                    r[keyColumn] == keyValue &&
                    (r['rolled_back'] == 0 || r['rolled_back'] == null),
              );
            } else {
              _tables[tableName]?.removeWhere((r) => r[keyColumn] == keyValue);
            }
          }
        }

        _tables[tableName]!.add(row);
      }
      return;
    }

    if (normalized.startsWith('UPDATE')) {
      final tableName = _extractTableName(sql);
      final table = _tables[tableName];
      if (table == null || arguments == null || arguments.isEmpty) return;

      final setRolledBack = normalized.contains('SET ROLLED_BACK');
      final setExecTime = normalized.contains('SET EXECUTION_TIME_MS');

      // WHERE clause ends with "WHERE version = ? AND rolled_back = 0"; the
      // version value is always the last argument.
      final whereVersion = arguments.last;

      for (final row in table) {
        final matchesVersion = row['version'] == whereVersion;
        final notRolledBack =
            row['rolled_back'] == 0 || row['rolled_back'] == null;

        if (matchesVersion && notRolledBack) {
          if (setRolledBack) {
            row['rolled_back'] = 1;
          } else if (setExecTime) {
            row['execution_time_ms'] = arguments.first;
          }
        }
      }
      return;
    }

    // DELETE
    if (normalized.startsWith('DELETE')) {
      final tableName = _extractTableName(sql);
      if (arguments != null && arguments.isNotEmpty) {
        _tables[tableName]?.removeWhere(
          (row) => row.values.any((v) => arguments.contains(v)),
        );
      } else {
        _tables[tableName]?.clear();
      }
      return;
    }

    // Any other DDL (ALTER, CREATE INDEX, …) — just record it.
  }

  @override
  Future<List<Map<String, Object?>>> query(
    String sql, [
    List<Object?>? arguments,
  ]) async {
    executedStatements.add(sql);

    final tableName = _extractTableName(sql);
    final rows = _tables[tableName] ?? [];

    final upperSql = sql.toUpperCase();
    final filterRolledBack = upperSql.contains('ROLLED_BACK = 0');

    final whereColMatch = RegExp(
      r'WHERE\s+(\w+)\s*=\s*\?',
      caseSensitive: false,
    ).firstMatch(sql);
    final whereColumn = whereColMatch?.group(1)?.toLowerCase();

    return rows.where((row) {
      if (filterRolledBack) {
        final rb = row['rolled_back'];
        if (rb != 0) return false;
      }

      if (arguments != null && arguments.isNotEmpty && whereColumn != null) {
        if (row[whereColumn] != arguments.first) return false;
      }

      return true;
    }).toList();
  }

  @override
  Future<T> transaction<T>(
    Future<T> Function(MigrationExecutor tx) action,
  ) async {
    return action(this);
  }

  // Helpers

  /// Check if a table exists in the in-memory store.
  bool hasTable(String name) => _tables.containsKey(name);

  /// Get all rows for a table.
  List<Map<String, Object?>> getRows(String tableName) =>
      List.unmodifiable(_tables[tableName] ?? []);

  /// Reset all state.
  void reset() {
    _tables.clear();
    executedStatements.clear();
  }

  String _extractTableName(String sql) {
    final normalized = sql.replaceAll('\n', ' ').replaceAll(RegExp(r'\s+'), ' ');
    final pattern = RegExp(
      r'(?:FROM|INTO|UPDATE|TABLE(?:\s+IF(?:\s+NOT)?\s+EXISTS)?)\s+(\w+)',
      caseSensitive: false,
    );
    final match = pattern.firstMatch(normalized);
    return match?.group(1) ?? '_unknown';
  }

  List<String> _extractInsertColumns(String sql) {
    final match = RegExp(
      r'\(([^)]+)\)\s*VALUES',
      caseSensitive: false,
    ).firstMatch(sql);
    if (match == null) return [];
    return match.group(1)!.split(',').map((c) => c.trim()).toList();
  }
}
