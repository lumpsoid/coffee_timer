import 'package:coffee_timer/sql_migrator/domain/models/exceptions.dart';
import 'package:coffee_timer/sql_migrator/domain/models/migration.dart';

/// Callback invoked after each migration completes successfully.
typedef MigrationCallback =
    Future<void> Function(
      Migration migration,
      MigrationDirection direction,
      int elapsedMs,
    );
