import 'package:coffee_timer/sql_migrator/sql_migrator.dart';
import 'package:flutter_test/flutter_test.dart';

class _SilentLogger implements MigrationLogger {
  const _SilentLogger();
  @override
  void info(String message) {}
  @override
  void warning(String message) {}
  @override
  void error(String message, [Object? error, StackTrace? stackTrace]) {}
}

Migration _create(int version, String table) => Migration(
  version: version,
  description: 'create $table',
  up: (db) => db.execute('CREATE TABLE $table (id INTEGER PRIMARY KEY)'),
  down: (db) => db.execute('DROP TABLE $table'),
);

void main() {
  late InMemoryExecutor executor;

  Migrator migrator(List<Migration> migrations) => Migrator(
    executor: executor,
    migrations: migrations,
    config: const MigratorConfig(useLocking: false),
    logger: const _SilentLogger(),
  );

  setUp(() => executor = InMemoryExecutor());

  test('migrate applies all pending migrations in order', () async {
    final applied = await migrator([
      _create(1, 'a'),
      _create(2, 'b'),
    ]).migrate();

    expect(applied.map((r) => r.version), [1, 2]);
    expect(executor.hasTable('a'), isTrue);
    expect(executor.hasTable('b'), isTrue);
  });

  test('migrate is idempotent — a second run applies nothing', () async {
    final migrations = [_create(1, 'a'), _create(2, 'b')];
    await migrator(migrations).migrate();

    final second = await migrator(migrations).migrate();
    expect(second, isEmpty);
    expect(await migrator(migrations).currentVersion(), 2);
  });

  test('rejects duplicate versions at construction', () {
    expect(
      () => migrator([_create(1, 'a'), _create(1, 'b')]),
      throwsA(isA<MigrationValidationException>()),
    );
  });

  test('migrateTo(1) stops before later migrations', () async {
    await migrator([
      _create(1, 'a'),
      _create(2, 'b'),
    ]).migrateTo(1);

    expect(executor.hasTable('a'), isTrue);
    expect(executor.hasTable('b'), isFalse);
  });

  test('rollback marks the last migration rolled back', () async {
    final migrations = [_create(1, 'a'), _create(2, 'b')];
    await migrator(migrations).migrate();

    final rolled = await migrator(migrations).rollback();
    expect(rolled.single.version, 2);
    expect(await migrator(migrations).currentVersion(), 1);
  });
}
