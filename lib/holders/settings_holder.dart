import 'dart:async';

import 'package:coffee_timer/src/bindings/bindings.dart';
import 'package:flutter/foundation.dart';
import 'package:rinf/rinf.dart';

/// The settings as Rust last sent them; the app theme follows them. Lives
/// for the app's lifetime.
class SettingsHolder extends ChangeNotifier {
  StreamSubscription<RustSignalPack<SettingsUpdate>>? _subscription;

  /// Null until Rust answers [SettingsReady].
  SettingsUpdate? get state => _state;
  SettingsUpdate? _state;

  /// Subscribes to Rust's updates, then asks for the current settings.
  void connect() {
    _subscription = SettingsUpdate.rustSignalStream.listen(
      (pack) => apply(pack.message),
    );
    const SettingsReady().sendSignalToRust();
  }

  /// Replaces the state.
  void apply(SettingsUpdate update) {
    _state = update;
    notifyListeners();
  }

  @override
  void dispose() {
    unawaited(_subscription?.cancel());
    super.dispose();
  }
}
