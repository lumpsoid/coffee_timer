import 'package:coffee_timer/sql_migrator/domain/migration_executor.dart';

/// A single database migration with an up and down operation.
///
/// Each migration must have a unique, monotonically increasing [version]
/// number. Migrations are executed in version order during [up] and reverse
/// order during [down].
///
/// Example:
/// ```dart
/// final migration = Migration(
///   version: 1,
///   description: 'Create users table',
///   up: (db) async {
///     await db.execute('''
///       CREATE TABLE users (
///         id INTEGER PRIMARY KEY AUTOINCREMENT,
///         email TEXT NOT NULL UNIQUE
///       )
///     ''');
///   },
///   down: (db) async {
///     await db.execute('DROP TABLE IF EXISTS users');
///   },
/// );
/// ```
class Migration {
  const Migration({
    required this.version,
    required this.description,
    required this.up,
    required this.down,
  });

  /// Unique version identifier. Must be a positive integer.
  final int version;

  /// Human-readable description of what this migration does.
  final String description;

  /// The forward migration function. Receives a transaction-wrapped executor.
  final Future<void> Function(MigrationExecutor db) up;

  /// The rollback migration function. Receives a transaction-wrapped executor.
  final Future<void> Function(MigrationExecutor db) down;

  @override
  String toString() => 'Migration(v$version: $description)';
}
