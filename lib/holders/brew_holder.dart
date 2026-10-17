import 'dart:async';

import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter/foundation.dart';
import 'package:rinf/rinf.dart';

/// The brew page as Rust last sent it. Lives for the app's lifetime.
class BrewHolder extends ChangeNotifier {
  StreamSubscription<RustSignalPack<BrewUpdate>>? _subscription;

  /// Null until Rust answers [BrewReady].
  BrewUpdate? get state => _state;
  BrewUpdate? _state;

  /// Subscribes to Rust's updates, then asks for the current state.
  void connect() {
    _subscription = BrewUpdate.rustSignalStream.listen(
      (pack) => apply(pack.message),
    );
    const BrewReady().sendSignalToRust();
  }

  /// Replaces the state.
  void apply(BrewUpdate update) {
    _state = update;
    notifyListeners();
  }

  @override
  void dispose() {
    unawaited(_subscription?.cancel());
    super.dispose();
  }
}
