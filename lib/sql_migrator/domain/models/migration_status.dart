import 'package:coffee_timer/sql_migrator/domain/models/migration.dart';
import 'package:coffee_timer/sql_migrator/domain/models/migration_record.dart';

/// A snapshot of migration status.
class MigrationStatus {
  const MigrationStatus({
    required this.currentVersion,
    required this.latestVersion,
    required this.appliedCount,
    required this.pendingCount,
    required this.appliedMigrations,
    required this.pendingMigrations,
  });
  final int currentVersion;
  final int latestVersion;
  final int appliedCount;
  final int pendingCount;
  final List<MigrationRecord> appliedMigrations;
  final List<Migration> pendingMigrations;

  bool get isUpToDate => pendingCount == 0;

  @override
  String toString() {
    final buf = StringBuffer()
      ..writeln('=== Migration Status ===')
      ..writeln('Current version: $currentVersion')
      ..writeln('Latest version:  $latestVersion')
      ..writeln('Applied: $appliedCount | Pending: $pendingCount');

    if (appliedMigrations.isNotEmpty) {
      buf.writeln('\nApplied:');
      for (final r in appliedMigrations) {
        buf.writeln(
          '  ✓ v${r.version}: ${r.description} '
          '(${r.executionTimeMs}ms, ${r.appliedAt.toIso8601String()})',
        );
      }
    }

    if (pendingMigrations.isNotEmpty) {
      buf.writeln('\nPending:');
      for (final m in pendingMigrations) {
        buf.writeln('  ○ v${m.version}: ${m.description}');
      }
    }

    return buf.toString();
  }
}
