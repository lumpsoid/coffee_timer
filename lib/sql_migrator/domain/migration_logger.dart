/// Logger interface for migration events.
///
/// Implement this to integrate with your app's logging infrastructure
/// (e.g. `package:logging`, `package:logger`, or Firebase Crashlytics).
abstract class MigrationLogger {
  void info(String message);
  void warning(String message);
  void error(String message, [Object? error, StackTrace? stackTrace]);
}
