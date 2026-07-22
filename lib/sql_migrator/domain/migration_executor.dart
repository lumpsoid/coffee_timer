/// Abstraction over the database executor so the migrator is not coupled
/// to any specific SQLite package (sqlite_async, sqflite, drift, sqlite3, …).
abstract class MigrationExecutor {
  /// Execute a raw SQL statement with optional positional arguments.
  Future<void> execute(String sql, [List<Object?>? arguments]);

  /// Execute a raw SQL query and return the result rows as maps.
  Future<List<Map<String, Object?>>> query(
    String sql, [
    List<Object?>? arguments,
  ]);

  /// Run a batch of operations inside a single transaction.
  /// The callback receives the same executor, but all operations are atomic.
  Future<T> transaction<T>(Future<T> Function(MigrationExecutor tx) action);
}
