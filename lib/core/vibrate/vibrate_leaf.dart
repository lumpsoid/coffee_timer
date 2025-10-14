import 'dart:async';

import 'package:coffee_timer/core/events/events.dart';
import 'package:vibration/vibration.dart';

abstract class VibrateFlutterLeaf {
  static Future<void> handle(Event event) async {
    switch (event) {
      case Vibrate _:
        if (await Vibration.hasVibrator()) {
          await Vibration.vibrate(duration: 100);
        }
      default:
        return;
    }
  }
}
