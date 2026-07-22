/// A record of a migration that has been applied to the database.
class MigrationRecord {
  const MigrationRecord({
    required this.version,
    required this.description,
    required this.appliedAt,
    required this.executionTimeMs,
    this.rolledBack = false,
  });

  factory MigrationRecord.fromRow(Map<String, Object?> row) {
    return MigrationRecord(
      version: row['version']! as int,
      description: row['description']! as String,
      appliedAt: DateTime.parse(row['applied_at']! as String),
      executionTimeMs: row['execution_time_ms']! as int,
      rolledBack: (row['rolled_back']! as int) == 1,
    );
  }

  /// The migration version that was applied.
  final int version;

  /// Description stored at the time of application.
  final String description;

  /// ISO-8601 timestamp of when the migration was applied.
  final DateTime appliedAt;

  /// Duration in milliseconds the migration took to execute.
  final int executionTimeMs;

  /// Whether this migration was a rollback (down) operation.
  final bool rolledBack;

  Map<String, Object?> toRow() => {
    'version': version,
    'description': description,
    'applied_at': appliedAt.toUtc().toIso8601String(),
    'execution_time_ms': executionTimeMs,
    'rolled_back': rolledBack ? 1 : 0,
  };

  @override
  String toString() =>
      'MigrationRecord(v$version, applied=${appliedAt.toIso8601String()}, '
      '${rolledBack ? "ROLLED BACK" : "active"})';
}
