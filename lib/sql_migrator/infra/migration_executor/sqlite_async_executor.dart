import 'package:coffee_timer/sql_migrator/domain/migration_executor.dart';
import 'package:sqlite_async/sqlite_async.dart';

/// A [MigrationExecutor] backed by a `sqlite_async` context.
///
/// Wraps either a [SqliteDatabase] (which can open transactions) or a
/// [SqliteWriteContext] handed to it inside one. [transaction] opens a real
/// `writeTransaction` when it can, and otherwise runs inline — our migrations
/// are applied one-at-a-time and never actually nest.
class SqliteAsyncExecutor implements MigrationExecutor {
  SqliteAsyncExecutor(this._db);

  final SqliteWriteContext _db;

  @override
  Future<void> execute(String sql, [List<Object?>? arguments]) async {
    await _db.execute(sql, arguments ?? const []);
  }

  @override
  Future<List<Map<String, Object?>>> query(
    String sql, [
    List<Object?>? arguments,
  ]) async {
    final result = await _db.getAll(sql, arguments ?? const []);
    return result.map(Map<String, Object?>.from).toList();
  }

  @override
  Future<T> transaction<T>(
    Future<T> Function(MigrationExecutor tx) action,
  ) async {
    final db = _db;
    if (db is SqliteConnection) {
      return db.writeTransaction((tx) => action(SqliteAsyncExecutor(tx)));
    }
    return action(this);
  }
}
