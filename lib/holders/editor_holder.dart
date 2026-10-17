import 'dart:async';

import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter/foundation.dart';
import 'package:rinf/rinf.dart';

/// The editor page as Rust last sent it. Lives for the app's lifetime.
///
/// Listeners are notified once per update, even when it equals the last
/// one: an `Open` whose draft carries a notice is a snackbar each time it
/// arrives, and a `Closed` is the page's cue to pop.
class EditorHolder extends ChangeNotifier {
  StreamSubscription<RustSignalPack<EditorUpdate>>? _subscription;

  /// Null until Rust answers [EditorReady].
  EditorUpdate? get state => _state;
  EditorUpdate? _state;

  /// Subscribes to Rust's updates, then asks for the current state.
  void connect() {
    _subscription = EditorUpdate.rustSignalStream.listen(
      (pack) => apply(pack.message),
    );
    const EditorReady().sendSignalToRust();
  }

  /// Replaces the state.
  void apply(EditorUpdate update) {
    _state = update;
    notifyListeners();
  }

  @override
  void dispose() {
    unawaited(_subscription?.cancel());
    super.dispose();
  }
}
