import 'package:coffee_timer/sql_migrator/domain/domain.dart';

/// Every schema change for the app database, in version order. This is the
/// single append-only log the `Migrator` replays — see its docs for the
/// authoring rules (append new versions, never edit a shipped one).
const List<Migration> appMigrations = [
  Migration(
    version: 1,
    description: 'Create recipe and step tables',
    up: _v1Up,
    down: _v1Down,
  ),
  Migration(
    version: 2,
    description: 'Seed example recipes',
    up: _v2Up,
    down: _v2Down,
  ),
  Migration(
    version: 3,
    description: 'Create settings key-value table',
    up: _v3Up,
    down: _v3Down,
  ),
];

Future<void> _v1Up(MigrationExecutor db) async {
  await db.execute('''
    CREATE TABLE recipe (
      id TEXT PRIMARY KEY,
      name TEXT,
      description TEXT
    )
  ''');
  await db.execute('''
    CREATE TABLE step (
      id TEXT PRIMARY KEY,
      recipe_id TEXT,
      name TEXT,
      duration_seconds INTEGER,
      FOREIGN KEY (recipe_id) REFERENCES recipe (id) ON DELETE CASCADE
    )
  ''');
}

Future<void> _v1Down(MigrationExecutor db) async {
  await db.execute('DROP TABLE IF EXISTS step');
  await db.execute('DROP TABLE IF EXISTS recipe');
}

Future<void> _v2Up(MigrationExecutor db) async {
  const recipes = [
    ['espresso-uuid', 'Espresso', 'Strong coffee'],
    ['pourover-uuid', 'Pour Over', 'Light and clean'],
  ];
  for (final r in recipes) {
    await db.execute(
      'INSERT INTO recipe (id, name, description) VALUES (?, ?, ?)',
      r,
    );
  }

  const steps = [
    ['step1', 'espresso-uuid', 'Grind', 30],
    ['step2', 'espresso-uuid', 'Brew', 25],
    ['step3', 'pourover-uuid', 'Grind', 40],
    ['step4', 'pourover-uuid', 'Bloom', 30],
    ['step5', 'pourover-uuid', 'Pour', 120],
  ];
  for (final s in steps) {
    await db.execute(
      'INSERT INTO step (id, recipe_id, name, duration_seconds) '
      'VALUES (?, ?, ?, ?)',
      s,
    );
  }
}

Future<void> _v2Down(MigrationExecutor db) async {
  const ids = ['espresso-uuid', 'pourover-uuid'];
  for (final id in ids) {
    await db.execute('DELETE FROM step WHERE recipe_id = ?', [id]);
    await db.execute('DELETE FROM recipe WHERE id = ?', [id]);
  }
}

Future<void> _v3Up(MigrationExecutor db) async {
  await db.execute('''
    CREATE TABLE settings (
      key TEXT PRIMARY KEY,
      value TEXT NOT NULL
    )
  ''');
}

Future<void> _v3Down(MigrationExecutor db) async {
  await db.execute('DROP TABLE IF EXISTS settings');
}
