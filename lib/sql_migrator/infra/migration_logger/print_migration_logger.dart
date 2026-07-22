import 'package:coffee_timer/sql_migrator/domain/migration_logger.dart';

/// Default logger that writes to stdout/stderr via [print].
class PrintMigrationLogger implements MigrationLogger {
  const PrintMigrationLogger({void Function(String) printer = print})
    : _printer = printer;

  final void Function(String) _printer;

  @override
  void info(String message) => _printer('[MIGRATION] $message');

  @override
  void warning(String message) => _printer('[MIGRATION ⚠] $message');

  @override
  void error(String message, [Object? error, StackTrace? stackTrace]) {
    _printer('[MIGRATION ✗] $message');
    if (error != null) _printer('  Error: $error');
    if (stackTrace != null) _printer('  $stackTrace');
  }
}
