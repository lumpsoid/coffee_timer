import 'dart:async';

import 'package:coffee_timer/core/settings/events/events.dart';

class SettingsEventBus {
  factory SettingsEventBus() => _instance;
  SettingsEventBus._internal();
  static final SettingsEventBus _instance = SettingsEventBus._internal();

  final _controller = StreamController<SettingsEvent>.broadcast();
  Stream<SettingsEvent> get stream => _controller.stream;

  void dispose() => _controller.close();
  void fire(SettingsEvent event) => _controller.add(event);
}
