class MigratorConfig {
  const MigratorConfig({
    this.tableName = '_schema_migrations',
    this.useLocking = true,
    this.lockTimeout = const Duration(seconds: 30),
  });

  /// Name of the table used to track applied migrations.
  final String tableName;

  /// If true, acquires an advisory lock before running migrations to
  /// prevent concurrent execution (useful in multi-isolate scenarios).
  final bool useLocking;

  /// Timeout for acquiring the migration lock.
  final Duration lockTimeout;
}
