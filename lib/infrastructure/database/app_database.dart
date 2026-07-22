import 'package:coffee_timer/infrastructure/database/app_migrations.dart';
import 'package:coffee_timer/sql_migrator/sql_migrator.dart';
import 'package:sqlite_async/sqlite_async.dart';

/// Opens the app's SQLite database and brings its schema up to date *before*
/// any repository is allowed to touch it. This is the single owner of
/// migrations for the whole app: recipes, steps and settings all live in one
/// database and one migration ledger.
final class AppDatabase {
  const AppDatabase({required this.path});

  final String path;

  /// Opens the database and applies all pending [appMigrations]. The returned
  /// [SqliteDatabase] is fully migrated — a query can never race table
  /// creation. Locking is off: this is a single-isolate app.
  Future<SqliteDatabase> open() async {
    final db = SqliteDatabase(path: path);
    await Migrator(
      executor: SqliteAsyncExecutor(db),
      migrations: appMigrations,
      config: const MigratorConfig(useLocking: false),
    ).migrate();
    return db;
  }
}
