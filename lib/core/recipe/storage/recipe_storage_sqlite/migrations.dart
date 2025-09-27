import 'package:coffee_timer/core/recipe/storage/recipe_storage_sqlite/recipe_storage_sqlite.dart';

const List<Migration> allMigrations = [
  Migration(
    version: 1,
    statements: [
      '''
      CREATE TABLE IF NOT EXISTS recipe (
        id TEXT PRIMARY KEY,
        name TEXT,
        description TEXT
      );
      ''',
      '''
      CREATE TABLE IF NOT EXISTS step (
        id TEXT PRIMARY KEY,
        recipe_id TEXT,
        name TEXT,
        duration_seconds INTEGER,
        FOREIGN KEY (recipe_id) REFERENCES recipe (id) ON DELETE CASCADE
      );
      ''',
    ],
  ),
  Migration(
    version: 2,
    statements: [
      // Seeding example recipes as SQL inserts
      '''
      INSERT INTO recipe (id, name, description)
      VALUES 
        ('espresso-uuid', 'Espresso', 'Strong coffee'),
        ('pourover-uuid', 'Pour Over', 'Light and clean');
      ''',
      '''
      INSERT INTO step (id, recipe_id, name, duration_seconds)
      VALUES 
        ('step1', 'espresso-uuid', 'Grind', 30),
        ('step2', 'espresso-uuid', 'Brew', 25),
        ('step3', 'pourover-uuid', 'Grind', 40),
        ('step4', 'pourover-uuid', 'Bloom', 30),
        ('step5', 'pourover-uuid', 'Pour', 120);
      ''',
    ],
  ),
];
