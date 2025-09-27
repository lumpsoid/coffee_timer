import 'package:coffee_timer/core/recipe/storage/recipe_storage_sqlite/recipe_storage_sqlite.dart';
import 'package:sqlite_async/sqlite_async.dart';

abstract class MigrationRunner {
  static Future<void> run({
    required SqliteDatabase db,
    required List<Migration> migrations,
  }) async {
    final currentVersionRow = await db.get('PRAGMA user_version');
    final currentVersion = currentVersionRow['user_version'] as int? ?? 0;

    final pendingMigrations =
        allMigrations.where((m) => m.version > currentVersion).toList()
          ..sort((a, b) => a.version.compareTo(b.version));

    await db.writeTransaction((tx) async {
      for (final migration in pendingMigrations) {
        for (final statement in migration.statements) {
          await tx.execute(statement);
        }
        await tx.execute('PRAGMA user_version = ${migration.version}');
      }
    });
  }
}
