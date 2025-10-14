import 'dart:async';

import 'package:coffee_timer/core/events/events.dart';
import 'package:wakelock_plus/wakelock_plus.dart';

class WakelockLeaf {
  static void handle(Event event) {
    switch (event) {
      case WakelockEnable _:
        unawaited(WakelockPlus.enable());

      case WakelockDisable _:
        unawaited(WakelockPlus.disable());

      default:
        return;
    }
  }
}
