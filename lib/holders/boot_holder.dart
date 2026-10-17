import 'dart:async';

import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter/foundation.dart';
import 'package:path_provider/path_provider.dart';
import 'package:rinf/rinf.dart';

/// Whether Rust failed to open the store. A successful boot sends nothing:
/// the app waits for the first recipe-list snapshot instead.
class BootHolder extends ChangeNotifier {
  StreamSubscription<RustSignalPack<BootFailed>>? _subscription;

  /// The last boot failure, until the next attempt.
  String? get failure => _failure;
  String? _failure;

  /// Subscribes to boot failures, then sends the first [Boot].
  Future<void> connect() async {
    _subscription = BootFailed.rustSignalStream.listen(
      (pack) => fail(pack.message.message),
    );
    await boot();
  }

  /// Sends [Boot] with the app's documents directory. Also the retry after
  /// a failure; once Rust runs, a repeated [Boot] is a no-op.
  Future<void> boot() async {
    _failure = null;
    notifyListeners();
    try {
      final dir = await getApplicationDocumentsDirectory();
      Boot(dbDir: dir.path).sendSignalToRust();
    } on Exception catch (error) {
      fail('$error');
    }
  }

  /// Shows [message] instead of the spinner.
  void fail(String message) {
    _failure = message;
    notifyListeners();
  }

  @override
  void dispose() {
    unawaited(_subscription?.cancel());
    super.dispose();
  }
}
