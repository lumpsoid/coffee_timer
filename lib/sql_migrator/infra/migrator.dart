import 'package:coffee_timer/sql_migrator/domain/migration_executor.dart';
import 'package:coffee_timer/sql_migrator/domain/migration_logger.dart';
import 'package:coffee_timer/sql_migrator/domain/models/exceptions.dart';
import 'package:coffee_timer/sql_migrator/domain/models/migration.dart';
import 'package:coffee_timer/sql_migrator/domain/models/migration_callback.dart';
import 'package:coffee_timer/sql_migrator/domain/models/migration_record.dart';
import 'package:coffee_timer/sql_migrator/domain/models/migration_status.dart';
import 'package:coffee_timer/sql_migrator/domain/models/migrator_config.dart';
import 'package:coffee_timer/sql_migrator/infra/migration_logger/print_migration_logger.dart';

/// The main migration engine.
///
/// Manages schema versioning for a SQLite database by applying and rolling
/// back [Migration] objects in order. All migrations run inside transactions
/// for atomicity.
///
/// ## How migrations must be authored
///
/// The migration list is a **replayable, append-only log**:
///
///   - **CREATE migrations are static.** Once a `CREATE TABLE` migration has
///     shipped, its body is frozen forever — never edit it to add/rename/drop
///     a column.
///   - **All schema changes are appended.** A new column, index, or rename is
///     a *new* migration with the next (higher) version, added at the end of
///     the list. Never modify a migration that has shipped.
///
/// Why this is not optional: [migrate] selects work purely by version number
/// — a migration whose version is already in the ledger is never re-run. So
/// the list has two audiences with different replay depth:
///
///   - **Existing installs** replay only the *tail* (versions past their
///     ledger). They never see an edit to an older migration.
///   - **Fresh installs** (new users, reinstalls, cleared data) replay the
///     *whole* list from scratch, running the current body of every migration.
///
/// Editing a shipped migration therefore does not "break existing users" —
/// they never re-run it — but it silently **forks the schema between old and
/// new installs** under the same version number, which nothing detects. Keep
/// every migration idempotency-free and single-run — append, never edit.
///
/// ## Usage
/// ```dart
/// final migrator = Migrator(
///   executor: SqliteAsyncExecutor(database),
///   migrations: allMigrations,
/// );
///
/// await migrator.migrate();       // apply all pending migrations
/// await migrator.rollback();      // roll back the last migration
/// await migrator.migrateTo(3);    // migrate to a specific version
/// ```
class Migrator {
  Migrator({
    required MigrationExecutor executor,
    required List<Migration> migrations,
    this.config = const MigratorConfig(),
    this.logger = const PrintMigrationLogger(),
    this.onMigrationComplete,
  }) : _executor = executor,
       _migrations = List.unmodifiable(
         List<Migration>.from(migrations)
           ..sort((a, b) => a.version.compareTo(b.version)),
       ) {
    _validateMigrations();
  }
  final MigrationExecutor _executor;
  final List<Migration> _migrations;
  final MigratorConfig config;
  final MigrationLogger logger;
  final MigrationCallback? onMigrationComplete;

  // ---------------------------------------------------------------------------
  // Public API
  // ---------------------------------------------------------------------------

  /// Apply all pending migrations up to the latest version.
  ///
  /// Returns the list of migrations that were applied.
  Future<List<MigrationRecord>> migrate() async {
    return _withLock(() async {
      await _ensureMigrationTable();
      final applied = await _getAppliedVersions();
      final pending = _migrations.where((m) => !applied.contains(m.version));

      if (pending.isEmpty) {
        logger.info(
          'Database is up to date (v${_currentVersionFrom(applied)}).',
        );
        return [];
      }

      logger.info('Applying ${pending.length} pending migration(s)...');

      final records = <MigrationRecord>[];
      for (final migration in pending) {
        records.add(await _applyUp(migration));
      }

      logger.info(
        'Migrations complete. Database is now at v${records.last.version}.',
      );
      return records;
    });
  }

  /// Migrate to a specific target version.
  ///
  /// If [targetVersion] is higher than the current version, pending migrations
  /// up to (and including) [targetVersion] are applied. If lower, applied
  /// migrations above it are rolled back in reverse order. Equal is a no-op.
  ///
  /// Pass `0` to roll back all migrations.
  Future<List<MigrationRecord>> migrateTo(int targetVersion) async {
    if (targetVersion < 0) {
      throw const MigrationValidationException('Target version must be >= 0.');
    }

    return _withLock(() async {
      await _ensureMigrationTable();
      final applied = await _getAppliedVersions();
      final currentVersion = _currentVersionFrom(applied);

      if (targetVersion == currentVersion) {
        logger.info('Already at v$currentVersion. Nothing to do.');
        return [];
      }

      final records = <MigrationRecord>[];

      if (targetVersion > currentVersion) {
        final pending = _migrations.where(
          (m) => m.version > currentVersion && m.version <= targetVersion,
        );
        for (final migration in pending) {
          records.add(await _applyUp(migration));
        }
      } else {
        final toRollback = _migrations.reversed.where(
          (m) => applied.contains(m.version) && m.version > targetVersion,
        );
        for (final migration in toRollback) {
          records.add(await _applyDown(migration));
        }
      }

      final newVersion = targetVersion == 0 ? 0 : records.last.version;
      logger.info('Database is now at v$newVersion.');
      return records;
    });
  }

  /// Roll back the most recently applied migration.
  ///
  /// Pass [steps] to roll back multiple migrations (default: 1).
  Future<List<MigrationRecord>> rollback({int steps = 1}) async {
    if (steps < 1) {
      throw const MigrationValidationException('Rollback steps must be >= 1.');
    }

    return _withLock(() async {
      await _ensureMigrationTable();
      final applied = await _getAppliedVersions();

      if (applied.isEmpty) {
        throw const MigrationRollbackException('No migrations to roll back.');
      }

      final sortedApplied = applied.toList()..sort((a, b) => b.compareTo(a));
      final toRollback = sortedApplied.take(steps);

      final records = <MigrationRecord>[];
      for (final version in toRollback) {
        final migration = _migrations.firstWhere(
          (m) => m.version == version,
          orElse: () => throw MigrationRollbackException(
            'Cannot find migration definition for v$version. '
            'Was it removed from the migration list?',
          ),
        );
        records.add(await _applyDown(migration));
      }

      final remaining = applied.difference(toRollback.toSet());
      final newVersion = remaining.isEmpty
          ? 0
          : remaining.reduce((a, b) => a > b ? a : b);
      logger.info(
        'Rolled back ${records.length} migration(s). '
        'Database is now at v$newVersion.',
      );
      return records;
    });
  }

  /// Returns the current schema version, or 0 if no migrations applied.
  Future<int> currentVersion() async {
    await _ensureMigrationTable();
    final applied = await _getAppliedVersions();
    return _currentVersionFrom(applied);
  }

  /// Returns records for all currently applied (non-rolled-back) migrations.
  Future<List<MigrationRecord>> appliedMigrations() async {
    await _ensureMigrationTable();
    final rows = await _executor.query(
      'SELECT * FROM ${config.tableName} '
      'WHERE rolled_back = 0 '
      'ORDER BY version ASC',
    );
    return rows.map(MigrationRecord.fromRow).toList();
  }

  /// Returns the list of migrations that have not yet been applied.
  Future<List<Migration>> pendingMigrations() async {
    await _ensureMigrationTable();
    final applied = await _getAppliedVersions();
    return _migrations.where((m) => !applied.contains(m.version)).toList();
  }

  /// Produces a human-readable status report.
  Future<MigrationStatus> status() async {
    await _ensureMigrationTable();
    final applied = await appliedMigrations();
    final pending = await pendingMigrations();
    final currentVer = applied.isEmpty ? 0 : applied.last.version;
    final latestVer = _migrations.isEmpty ? 0 : _migrations.last.version;

    return MigrationStatus(
      currentVersion: currentVer,
      latestVersion: latestVer,
      appliedCount: applied.length,
      pendingCount: pending.length,
      appliedMigrations: applied,
      pendingMigrations: pending,
    );
  }

  void _validateMigrations() {
    if (_migrations.isEmpty) return;

    final versions = <int>{};
    for (final m in _migrations) {
      if (m.version <= 0) {
        throw MigrationValidationException(
          'Migration version must be positive, got ${m.version}.',
        );
      }
      if (!versions.add(m.version)) {
        throw MigrationValidationException(
          'Duplicate migration version: ${m.version}.',
        );
      }
    }
  }

  Future<void> _ensureMigrationTable() async {
    await _executor.execute('''
      CREATE TABLE IF NOT EXISTS ${config.tableName} (
        version       INTEGER NOT NULL,
        description   TEXT    NOT NULL,
        applied_at    TEXT    NOT NULL,
        execution_time_ms INTEGER NOT NULL DEFAULT 0,
        rolled_back   INTEGER NOT NULL DEFAULT 0,
        PRIMARY KEY (version, rolled_back)
      )
    ''');
  }

  Future<Set<int>> _getAppliedVersions() async {
    final rows = await _executor.query(
      'SELECT version FROM ${config.tableName} WHERE rolled_back = 0',
    );
    return rows.map((r) => r['version'] as int? ?? 0).toSet();
  }

  int _currentVersionFrom(Set<int> applied) {
    if (applied.isEmpty) return 0;
    return applied.reduce((a, b) => a > b ? a : b);
  }

  Future<MigrationRecord> _applyUp(Migration migration) async {
    logger.info('↑ Applying v${migration.version}: ${migration.description}');

    final stopwatch = Stopwatch()..start();

    try {
      await _executor.transaction((tx) async {
        await migration.up(tx);

        await tx.execute(
          'INSERT OR REPLACE INTO ${config.tableName} '
          '(version, description, applied_at, execution_time_ms, rolled_back) '
          'VALUES (?, ?, ?, ?, ?)',
          [
            migration.version,
            migration.description,
            DateTime.now().toUtc().toIso8601String(),
            0,
            0,
          ],
        );
      });
    } catch (e, st) {
      stopwatch.stop();
      logger.error('Failed applying v${migration.version}', e, st);
      throw MigrationExecutionException(
        version: migration.version,
        direction: MigrationDirection.up,
        message: e.toString(),
        cause: e,
      );
    }

    stopwatch.stop();
    final elapsedMs = stopwatch.elapsedMilliseconds;

    // Update execution time outside the migration transaction.
    await _executor.execute(
      'UPDATE ${config.tableName} SET execution_time_ms = ? '
      'WHERE version = ? AND rolled_back = 0',
      [elapsedMs, migration.version],
    );

    logger.info('  ✓ v${migration.version} applied in ${elapsedMs}ms');

    if (onMigrationComplete != null) {
      await onMigrationComplete!(migration, MigrationDirection.up, elapsedMs);
    }

    return MigrationRecord(
      version: migration.version,
      description: migration.description,
      appliedAt: DateTime.now().toUtc(),
      executionTimeMs: elapsedMs,
    );
  }

  Future<MigrationRecord> _applyDown(Migration migration) async {
    logger.info(
      '↓ Rolling back v${migration.version}: ${migration.description}',
    );

    final stopwatch = Stopwatch()..start();

    try {
      await _executor.transaction((tx) async {
        await migration.down(tx);

        // Mark as rolled back rather than deleting — keeps an audit trail.
        await tx.execute(
          'UPDATE ${config.tableName} SET rolled_back = 1 '
          'WHERE version = ? AND rolled_back = 0',
          [migration.version],
        );
      });
    } catch (e, st) {
      stopwatch.stop();
      logger.error('Failed rolling back v${migration.version}', e, st);
      throw MigrationExecutionException(
        version: migration.version,
        direction: MigrationDirection.down,
        message: e.toString(),
        cause: e,
      );
    }

    stopwatch.stop();
    final elapsedMs = stopwatch.elapsedMilliseconds;

    logger.info('  ✓ v${migration.version} rolled back in ${elapsedMs}ms');

    if (onMigrationComplete != null) {
      await onMigrationComplete!(migration, MigrationDirection.down, elapsedMs);
    }

    return MigrationRecord(
      version: migration.version,
      description: migration.description,
      appliedAt: DateTime.now().toUtc(),
      executionTimeMs: elapsedMs,
      rolledBack: true,
    );
  }

  Future<T> _withLock<T>(Future<T> Function() action) async {
    if (!config.useLocking) return action();

    // SQLite advisory lock via a dedicated lock table.
    await _executor.execute('''
      CREATE TABLE IF NOT EXISTS _migration_lock (
        id          INTEGER PRIMARY KEY CHECK (id = 1),
        locked_at   TEXT NOT NULL,
        locked_by   TEXT NOT NULL
      )
    ''');

    final lockId = 'migrator_${DateTime.now().microsecondsSinceEpoch}';
    final deadline = DateTime.now().add(config.lockTimeout);

    while (true) {
      try {
        await _executor.transaction((tx) async {
          final existing = await tx.query(
            'SELECT locked_at FROM _migration_lock WHERE id = 1',
          );

          if (existing.isNotEmpty) {
            final lockedAt = DateTime.parse(
              existing.first['locked_at'] as String? ?? '',
            );
            final age = DateTime.now().toUtc().difference(lockedAt);
            if (age > config.lockTimeout) {
              // Stale lock — forcefully release.
              await tx.execute('DELETE FROM _migration_lock WHERE id = 1');
            } else {
              throw const MigrationLockException(
                'Lock is held by another process.',
              );
            }
          }

          await tx.execute(
            'INSERT INTO _migration_lock (id, locked_at, locked_by)'
            ' VALUES (1, ?, ?)',
            [DateTime.now().toUtc().toIso8601String(), lockId],
          );
        });

        break; // Lock acquired.
      } on MigrationLockException {
        if (DateTime.now().isAfter(deadline)) {
          throw MigrationLockException(
            'Could not acquire migration lock within ${config.lockTimeout}.',
          );
        }
        await Future<void>.delayed(const Duration(milliseconds: 100));
      }
    }

    try {
      return await action();
    } finally {
      await _executor.execute(
        'DELETE FROM _migration_lock WHERE id = 1 AND locked_by = ?',
        [lockId],
      );
    }
  }
}
